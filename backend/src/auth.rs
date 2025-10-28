use crate::models::{ApiResponse, User};
use crate::perms::ApiResult;
use crate::{get_access_time, get_refresh_time};
use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rocket::http::private::cookie::Expiration;
use rocket::http::{Cookie, CookieJar, SameSite};
use rocket::request::{FromRequest, Outcome};
use rocket::{Request, State, http::Status, post, serde::json::Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::str::FromStr;
use uuid::Uuid;

fn create_access_cookie(cookies: &CookieJar<'_>, auth: &AuthUser) -> ApiResult<()> {
    let jwt_secret = std::env::var("JWT_SECRET").map_err(|e| {
        ApiResponse::fail(
            Status::InternalServerError,
            "internal server error",
            Some(&e),
        )
    })?;
    let token = encode::<AuthUser>(
        &Header::default(),
        auth,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|e| {
        ApiResponse::fail(
            Status::InternalServerError,
            "internal server error",
            Some(&e),
        )
    })?;

    cookies.add_private(
        Cookie::build(("access_token", token))
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Lax)
            .expires(Expiration::Session),
    );
    Ok(())
}

async fn login_cookie(
    pool: &State<PgPool>,
    cookies: &CookieJar<'_>,
    user: impl Into<AuthUser>,
) -> ApiResult<()> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;
    let auth = &user.into();
    let refresh_token = sqlx::query_scalar!(
        "INSERT INTO user_tokens (user_id, expires_at) VALUES ($1, $2) RETURNING refresh_token",
        auth.user_id,
        get_refresh_time().naive_utc()
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;
    create_access_cookie(cookies, auth)?;
    cookies.add_private(
        Cookie::build(("refresh_token", refresh_token.to_string()))
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Lax)
            .expires(Expiration::Session),
    );
    tx.commit()
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub login: String,
    pub username: String,
    pub admin: bool,
    pub exp: usize,
}

impl AuthUser {
    fn renew(mut self, new_time: i64) -> Self {
        self.exp = new_time as usize;
        self
    }
}

impl From<User> for AuthUser {
    fn from(user: User) -> Self {
        AuthUser {
            user_id: user.id,
            login: user.login,
            username: user.username,
            admin: user.admin,
            exp: get_access_time().timestamp() as usize,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct LoginData {
    pub login: String,
    pub password: String,
}

#[post("/login", format = "json", data = "<data>")]
pub async fn login(
    data: Json<LoginData>,
    pool: &State<PgPool>,
    cookies: &CookieJar<'_>,
    auth: Result<AuthUser, (Status, Json<ApiResponse>)>,
) -> ApiResult {
    if auth.is_ok() {
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
        login_cookie(pool, cookies, user).await?;
        Ok((Status::Ok, ApiResponse::success()))
    } else {
        Err(ApiResponse::fail(
            Status::BadRequest,
            "wrong login or password",
            None,
        ))
    }
}

// create new user only available for admin
#[post("/register/new", format = "json", data = "<data>")]
pub async fn register(
    data: Json<LoginData>,
    pool: &State<PgPool>,
    cookies: &CookieJar<'_>,
    auth: Result<AuthUser, (Status, Json<ApiResponse>)>,
) -> ApiResult {
    let auth = auth?;

    if !auth.admin {
        return Err(ApiResponse::fail(
            Status::Forbidden,
            "you need to be admin to create new user",
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

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

    let hashed_password = hash(&data.password, DEFAULT_COST).map_err(|e| {
        ApiResponse::fail(
            Status::InternalServerError,
            "internal server error",
            Some(&e),
        )
    })?;

    let user = sqlx::query_as!(
        User,
        "INSERT INTO users (login, password) VALUES ($1, $2) RETURNING *",
        data.login,
        hashed_password
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;
    sqlx::query!("INSERT INTO permissions (user_id) VALUES ($1)", user.id)
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;
    tx.commit()
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;
    Ok((Status::Ok, ApiResponse::success()))
}

#[post("/logout")]
pub fn logout(cookies: &CookieJar<'_>) -> Json<ApiResponse> {
    cookies.remove_private(Cookie::from("access_token"));
    cookies.remove_private(Cookie::from("refresh_token"));
    ApiResponse::success()
}

#[derive(Serialize, Deserialize)]
pub struct UserData {
    user_id: Uuid,
    login: String,
    username: String,
    admin: bool,
}

#[get("/user")]
pub async fn get_user(
    auth: Result<AuthUser, (Status, Json<ApiResponse>)>,
) -> ApiResult<Json<UserData>> {
    let auth = auth?;
    Ok(Json(UserData {
        user_id: auth.user_id,
        login: auth.login,
        username: auth.username,
        admin: auth.admin,
    }))
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
    auth: Result<AuthUser, (Status, Json<ApiResponse>)>,
) -> ApiResult {
    let auth = auth?;

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

    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", auth.user_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

    if !verify(&data.current_password, &user.password).map_err(|e| {
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
        auth.user_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

    tx.commit()
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

    Ok((Status::Ok, ApiResponse::success()))
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = (Status, Json<ApiResponse>);

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let pool = match req.guard::<&State<PgPool>>().await {
            Outcome::Success(pool) => pool,
            Outcome::Error(_) | Outcome::Forward(_) => {
                return Outcome::Error((
                    Status::InternalServerError,
                    ApiResponse::fail(Status::InternalServerError, "internal server error", None),
                ));
            }
        };

        let cookies = req.cookies();
        let token = cookies
            .get_private("access_token")
            .map(|cookie| cookie.value().to_string());
        let Ok(refresh_token) = cookies
            .get_private("refresh_token")
            .map(|cookie| Uuid::from_str(cookie.value()))
            .transpose()
        else {
            return Outcome::Error((
                Status::InternalServerError,
                ApiResponse::fail(Status::InternalServerError, "internal server error", None),
            ));
        };

        let (token, refresh_token) = match (token, refresh_token) {
            (Some(t), Some(u)) => (t, u),
            _ => {
                cookies.remove_private(Cookie::from("access_token"));
                cookies.remove_private(Cookie::from("refresh_token"));
                return Outcome::Error((
                    Status::Unauthorized,
                    ApiResponse::fail(Status::Unauthorized, "you are not authenticated", None),
                ));
            }
        };

        let Ok(jwt_secret) = std::env::var("JWT_SECRET") else {
            return Outcome::Error((
                Status::InternalServerError,
                ApiResponse::fail(Status::InternalServerError, "internal server error", None),
            ));
        };
        let key = DecodingKey::from_secret(jwt_secret.as_bytes());
        match decode::<AuthUser>(&token, &key, &Validation::default()) {
            Ok(token_data) => {
                let now = Utc::now().timestamp() as usize;
                if token_data.claims.exp > now {
                    Outcome::Success(token_data.claims)
                } else {
                    let expires_at = match sqlx::query_scalar!(
                        "SELECT expires_at FROM user_tokens WHERE user_id = $1 AND refresh_token = $2 ORDER BY created_at DESC LIMIT 1",
                        token_data.claims.user_id,
                        refresh_token
                    )
                    .fetch_one(pool.inner())
                    .await
                    {
                        Ok(v) => v,
                        Err(e) => {
                            return Outcome::Error((
                                Status::Unauthorized,
                                ApiResponse::fail(
                                    Status::InternalServerError,
                                    "database error",
                                    Some(&e),
                                ),
                            ));
                        }
                    };

                    if expires_at > Utc::now().naive_utc() {
                        let auth = token_data.claims.renew(get_access_time().timestamp());
                        match create_access_cookie(cookies, &auth) {
                            Ok(_) => Outcome::Success(auth),
                            Err(e) => Outcome::Error((Status::InternalServerError, e)),
                        }
                    } else {
                        Outcome::Error((
                            Status::Unauthorized,
                            ApiResponse::fail(
                                Status::Unauthorized,
                                "authentication token expired",
                                None,
                            ),
                        ))
                    }
                }
            }
            Err(e) if e.kind() == &jsonwebtoken::errors::ErrorKind::InvalidToken => {
                Outcome::Error((
                    Status::Unauthorized,
                    ApiResponse::fail(Status::Unauthorized, "you are not authenticated", None),
                ))
            }
            Err(e) => Outcome::Error((
                Status::Unauthorized,
                ApiResponse::fail(Status::Unauthorized, "authentication error", Some(&e)),
            )),
        }
    }
}
