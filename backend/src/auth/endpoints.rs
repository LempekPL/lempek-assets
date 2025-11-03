use crate::auth::*;
use crate::models::{ApiResponse, User, UserToken};
use crate::perms::ApiResult;
use crate::REFRESH_TOKEN_TIME;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{DateTime, NaiveDateTime, Utc};
use rocket::http::private::cookie::Expiration;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::net::IpAddr;

#[derive(Serialize, Deserialize)]
pub struct LoginData {
    pub login: String,
    pub password: String,
}

#[derive(Deserialize, Default)]
struct IpApiResponse {
    city: Option<String>,
    #[serde(rename = "regionName")]
    region: Option<String>,
    country: Option<String>,
}

// TODO: make cookies live as long as they need to
async fn login_cookie(
    uaip: UserAgentIp,
    pool: &State<PgPool>,
    cookies: &CookieJar<'_>,
    user: impl Into<UserData>,
    stay_logged_in: bool,
) -> ApiResult<()> {
    let user = user.into();
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

    let user_data = if let Some(user_ip) = uaip.client_ip {
        let url = format!(
            "http://ip-api.com/json/{}?fields=city,regionName,country",
            user_ip
        );
        let resp = reqwest::Client::new().get(&url).send().await.map_err(|e| {
            ApiResponse::fail(Status::InternalServerError, "request error", Some(&e))
        })?;
        resp.json().await.map_err(|e| {
            ApiResponse::fail(Status::InternalServerError, "request error", Some(&e))
        })?
    } else {
        IpApiResponse::default()
    };

    let refresh_token = sqlx::query_scalar!(
        "INSERT INTO user_tokens (user_id, expires_at, user_agent, city, region, country) VALUES ($1, $2, $3, $4, $5, $6) RETURNING refresh_token",
        user.user_id,
        (Utc::now() + REFRESH_TOKEN_TIME),
        uaip.user_agent,
        user_data.city,
        user_data.region,
        user_data.country,
    )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

    refresh_refresh_cookie(
        cookies,
        &RefreshTokenData {
            user_id: user.user_id,
            refresh_token,
            stay_logged_in,
        },
        Expiration::Session,
    )
    .await?;
    refresh_access_cookie(cookies, &user, Expiration::Session).await?;

    tx.commit()
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;
    Ok(())
}

#[post("/login", format = "json", data = "<data>")]
pub async fn login(
    data: Json<LoginData>,
    uaip: UserAgentIp,
    pool: &State<PgPool>,
    cookies: &CookieJar<'_>,
    user: AuthUser,
) -> ApiResult {
    if user.is_ok() {
        return Err(ApiResponse::fail(
            Status::Conflict,
            "you are already logged in",
            None,
        ));
    }

    if data.login.trim().is_empty() || data.password.len() < 8 {
        return Err(ApiResponse::fail(
            Status::BadRequest,
            "invalid credentials format",
            None,
        ));
    }

    let user = match sqlx::query_as!(User, "SELECT * FROM users WHERE login = $1", data.login)
        .fetch_one(pool.inner())
        .await
    {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => {
            return Err(ApiResponse::fail(
                Status::BadRequest,
                "wrong login or password",
                None,
            ));
        }
        Err(e) => {
            return Err(ApiResponse::fail(
                Status::InternalServerError,
                "database error",
                Some(&e),
            ));
        }
    };

    if verify(&data.password, &user.password).map_err(|e| {
        ApiResponse::fail(
            Status::InternalServerError,
            "internal server error",
            Some(&e),
        )
    })? {
        login_cookie(uaip, pool, cookies, user, true).await?;
        Ok((Status::Ok, ApiResponse::success()))
    } else {
        Err(ApiResponse::fail(
            Status::BadRequest,
            "wrong login or password",
            None,
        ))
    }
}

async fn remove_current_refresh_token(cookies: &CookieJar<'_>, pool: &State<PgPool>) -> ApiResult<()> {
    let refresh_token = cookies
        .get_private("refresh_token")
        .map(|cookie| cookie.value().to_string());
    if let Some(refresh_token) = refresh_token {
        let jwt_secret = std::env::var("JWT_SECRET").map_err(|e| {
            ApiResponse::fail(Status::InternalServerError, "internal server error", None)
        })?;
        let key = DecodingKey::from_secret(jwt_secret.as_bytes());
        let mut valid = Validation::default();
        valid.validate_exp = false;
        if let Ok(refresh_data) = decode::<JWTData<RefreshTokenData>>(&refresh_token, &key, &valid)
        {
            let _ = sqlx::query!(
                "DELETE FROM user_tokens WHERE user_id = $1 AND (refresh_token = $2 OR expires_at < NOW());",
                refresh_data.claims.data.user_id,
                refresh_data.claims.data.refresh_token
            )
                .execute(pool.inner())
                .await;
        }
    }
    Ok(())
}

#[post("/logout")]
pub async fn logout(cookies: &CookieJar<'_>, pool: &State<PgPool>) -> Json<ApiResponse> {
    let _ = remove_current_refresh_token(cookies, pool).await;
    cookies.remove_private(Cookie::from("access_token"));
    cookies.remove_private(Cookie::from("refresh_token"));
    ApiResponse::success()
}

#[get("/user")]
pub async fn get_user(user: AuthUser) -> ApiResult<Json<UserData>> {
    let user = user?;
    Ok(Json(UserData {
        user_id: user.user_id,
        login: user.login,
        username: user.username,
        admin: user.admin,
    }))
}

#[derive(Serialize)]
pub struct UserWithoutPassword {
    pub id: Uuid,
    pub login: String,
    pub username: String,
    pub admin: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[get("/user/all", rank = 2)]
pub async fn get_user_all(
    user: AuthUser,
    pool: &State<PgPool>,
) -> ApiResult<Json<UserWithoutPassword>> {
    let user = user?;
    let user_data = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user.user_id)
        .fetch_one(pool.inner())
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

    Ok(Json(UserWithoutPassword {
        id: user_data.id,
        login: user_data.login,
        username: user_data.username,
        admin: user_data.admin,
        created_at: user_data.created_at,
        updated_at: user_data.updated_at,
    }))
}

#[get("/user/all?<id>")]
pub async fn get_user_all_admin(
    id: Uuid,
    user: AuthAdminUser,
    pool: &State<PgPool>,
) -> ApiResult<Json<UserWithoutPassword>> {
    let _user = user?;
    let user_data = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
        .fetch_one(pool.inner())
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

    Ok(Json(UserWithoutPassword {
        id: user_data.id,
        login: user_data.login,
        username: user_data.username,
        admin: user_data.admin,
        created_at: user_data.created_at,
        updated_at: user_data.updated_at,
    }))
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct UserTokenWithoutTheToken {
    id: Uuid,
    user_agent: Option<String>,
    country: Option<String>,
    region: Option<String>,
    city: Option<String>,
    expires_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

#[get("/user/tokens")]
pub async fn get_user_tokens(
    user: AuthUser,
    pool: &State<PgPool>,
) -> ApiResult<Json<Vec<UserTokenWithoutTheToken>>> {
    let user = user?;
    let user_tokens = sqlx::query_as!(
        UserTokenWithoutTheToken,
        "SELECT id, user_agent, country, region, city, expires_at, created_at FROM user_tokens WHERE user_id = $1 ORDER BY created_at",
        user.user_id
    )
        .fetch_all(pool.inner())
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;
    Ok(Json(user_tokens))
}

#[derive(Serialize, Deserialize)]
pub struct RemoveTokenData {
    pub id: Uuid,
}

#[delete("/user/tokens", format = "json", data = "<data>")]
pub async fn remove_user_token(
    data: Json<RemoveTokenData>,
    user: AuthUser,
    pool: &State<PgPool>,
) -> ApiResult {
    let user = user?;
    sqlx::query_as!(
        UserToken,
        "DELETE FROM user_tokens WHERE user_id = $1 AND id = $2",
        user.user_id,
        data.id
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;
    // TODO: get amount and give it (if needed)
    Ok((Status::Ok, ApiResponse::success()))
}

#[derive(Deserialize)]
pub struct ChangePasswordData {
    pub current_password: String,
    pub new_password: String,
}

#[post("/user/change_password", format = "json", data = "<data>")]
pub async fn change_password(
    data: Json<ChangePasswordData>,
    pool: &State<PgPool>,
    user: AuthUser,
) -> ApiResult {
    let user = user?;

    if data.current_password.is_empty() || data.new_password.len() < 8 {
        return Err(ApiResponse::fail(
            Status::BadRequest,
            "password must have at least 8 characters",
            None,
        ));
    }

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

    let password = sqlx::query_scalar!("SELECT password FROM users WHERE id = $1", user.user_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

    if !verify(&data.current_password, &password).map_err(|e| {
        ApiResponse::fail(
            Status::InternalServerError,
            "internal server error",
            Some(&e),
        )
    })? {
        return Err(ApiResponse::fail(
            Status::BadRequest,
            "wrong current password",
            None,
        ));
    }

    let hashed = hash(&data.new_password, DEFAULT_COST).map_err(|e| {
        ApiResponse::fail(
            Status::InternalServerError,
            "internal server error",
            Some(&e),
        )
    })?;

    sqlx::query!(
        "UPDATE users SET password = $1 WHERE id = $2",
        hashed,
        user.user_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

    tx.commit()
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

    Ok((Status::Ok, ApiResponse::success()))
}

#[derive(Debug)]
pub struct UserAgentIp {
    user_agent: Option<String>,
    client_ip: Option<IpAddr>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserAgentIp {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let user_agent = request
            .headers()
            .get_one("User-Agent")
            .map(|s| s.to_string());
        let client_ip = request.client_ip();
        Outcome::Success(UserAgentIp {
            user_agent,
            client_ip,
        })
    }
}
