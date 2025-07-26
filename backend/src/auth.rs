use crate::models::{ApiResponse, User};
use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rocket::http::{Cookie, CookieJar, SameSite};
use rocket::request::{FromRequest, Outcome};
use rocket::{Request, State, http::Status, post, serde::json::Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

fn set_cookie(
    cookies: &CookieJar<'_>,
    user: impl Into<AuthUser>,
) -> Result<(), (Status, Json<ApiResponse>)> {
    let jwt_secret = std::env::var("JWT_SECRET").map_err(|e| {
        ApiResponse::fail(
            Status::InternalServerError,
            "internal server error",
            Some(&e),
        )
    })?;
    let auth = &user.into();
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
        Cookie::build(("jwt_token", token))
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Lax)
            .expires(rocket::time::OffsetDateTime::from_unix_timestamp(auth.exp as i64).unwrap()),
    );
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub login: String,
    pub admin: bool,
    pub exp: usize,
}

impl From<User> for AuthUser {
    fn from(user: User) -> Self {
        AuthUser {
            user_id: user.id,
            login: user.login,
            admin: user.admin,
            exp: (Utc::now() + Duration::hours(5)).timestamp() as usize,
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
) -> Result<Json<ApiResponse>, (Status, Json<ApiResponse>)> {
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
        set_cookie(cookies, user)?;
        Ok(ApiResponse::success())
    } else {
        Err(ApiResponse::fail(
            Status::BadRequest,
            "wrong login or password",
            None,
        ))
    }
}

#[post("/register", format = "json", data = "<data>")]
pub async fn register(
    data: Json<LoginData>,
    pool: &State<PgPool>,
    cookies: &CookieJar<'_>,
    auth: Result<AuthUser, (Status, Json<ApiResponse>)>,
) -> Result<Json<ApiResponse>, (Status, Json<ApiResponse>)> {
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

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

    let existing_user = sqlx::query_scalar!(
        "SELECT EXISTS (SELECT 1 FROM users WHERE login = $1)",
        data.login
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?
    .unwrap_or(false);

    if existing_user {
        return Err(ApiResponse::fail(
            Status::Conflict,
            "user already registered",
            None,
        ));
    }

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
    set_cookie(cookies, user)?;
    tx.commit()
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;
    Ok(ApiResponse::success())
}

#[post("/logout")]
pub fn logout(cookies: &CookieJar<'_>) -> Json<ApiResponse> {
    cookies.remove_private(Cookie::from("jwt_token"));
    ApiResponse::success()
}

#[derive(Serialize, Deserialize)]
pub struct UserData {
    user_id: Uuid,
    login: String,
    admin: bool,
}

#[get("/user")]
pub async fn get_user(
    auth: Result<AuthUser, (Status, Json<ApiResponse>)>,
) -> Result<Json<UserData>, (Status, Json<ApiResponse>)> {
    let auth = auth?;
    Ok(Json(UserData {
        user_id: auth.user_id,
        login: auth.login,
        admin: auth.admin,
    }))
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = (Status, Json<ApiResponse>);

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let cookies = req.cookies();
        let token = cookies
            .get_private("jwt_token")
            .map(|cookie| cookie.value().to_string())
            .or_else(|| {
                cookies
                    .get("jwt_token")
                    .map(|cookie| cookie.value().to_string())
            });

        let token = match token {
            Some(t) => t,
            None => {
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
            Err(e) if e.kind() == &jsonwebtoken::errors::ErrorKind::InvalidToken => Outcome::Error((
                Status::Unauthorized,
                ApiResponse::fail(Status::Unauthorized, "you are not authenticated", None),
            )),
            Err(e) => Outcome::Error((
                Status::Unauthorized,
                ApiResponse::fail(Status::Unauthorized, "authentication error", Some(&e)),
            )),
        }
    }
}
