use crate::models::{ApiResponse, User};
use crate::perms::ApiResult;
use crate::{ACCESS_TOKEN_TIME, REFRESH_TOKEN_TIME};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rocket::http::private::cookie::Expiration;
use rocket::http::{Cookie, CookieJar, SameSite, Status};
use rocket::outcome::try_outcome;
use rocket::request::{FromRequest, Outcome};
use rocket::serde::json::Json;
use rocket::{Request, State};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::borrow::Cow;
use uuid::Uuid;

pub mod endpoints;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JWTData<T> {
    pub data: T,
    pub exp: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenData {
    pub user_id: Uuid,
    pub refresh_token: Uuid,
    pub stay_logged_in: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserData {
    pub user_id: Uuid,
    pub login: String,
    pub username: String,
    pub admin: bool,
}

impl From<AdminData> for UserData {
    fn from(value: AdminData) -> Self {
        Self {
            user_id: value.user_id,
            login: value.login,
            username: value.username,
            admin: true,
        }
    }
}

impl From<User> for UserData {
    fn from(value: User) -> Self {
        Self {
            user_id: value.id,
            login: value.login,
            username: value.username,
            admin: value.admin,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminData {
    pub user_id: Uuid,
    pub login: String,
    pub username: String,
}

#[derive(Debug)]
pub struct UserAdminError;

impl std::fmt::Display for UserAdminError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "User does not have admin rights")
    }
}

impl std::error::Error for UserAdminError {}

impl TryFrom<UserData> for AdminData {
    type Error = UserAdminError;

    fn try_from(value: UserData) -> Result<Self, Self::Error> {
        if !value.admin {
            return Err(UserAdminError);
        }
        Ok(Self {
            user_id: value.user_id,
            login: value.login,
            username: value.username,
        })
    }
}

// when using these types to protect endpoints remember to `let user = user?;` even if you are not planing on using the user data
pub type AuthUser = ApiResult<UserData>;
pub type AuthAdminUser = ApiResult<AdminData>;

fn create_jwt_cookie<T: Serialize, Str: Into<Cow<'static, str>>>(
    cookies: &CookieJar<'_>,
    name: Str,
    data: &JWTData<T>,
    expiration: Expiration,
) -> ApiResult<()> {
    let jwt_secret = std::env::var("JWT_SECRET").map_err(|e| {
        ApiResponse::fail(
            Status::InternalServerError,
            "internal server error",
            Some(&e),
        )
    })?;
    let token = encode(
        &Header::default(),
        data,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|e| {
        ApiResponse::fail(
            Status::InternalServerError,
            "internal server error",
            Some(&e),
        )
    })?;

    let cookie = Cookie::build((name, token))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .expires(expiration);
    cookies.add_private(cookie);
    Ok(())
}

async fn refresh_access_cookie(
    cookies: &CookieJar<'_>,
    user: &UserData,
    expiration: Expiration,
) -> ApiResult<()> {
    let user_token = JWTData {
        data: user,
        exp: (Utc::now() + ACCESS_TOKEN_TIME).timestamp() as usize,
    };
    create_jwt_cookie(cookies, "access_token", &user_token, expiration)
}

async fn refresh_refresh_cookie(
    cookies: &CookieJar<'_>,
    refresh_data: &RefreshTokenData,
    expiration: Expiration,
) -> ApiResult<()> {
    let user_token = JWTData {
        data: refresh_data,
        exp: (Utc::now() + REFRESH_TOKEN_TIME).timestamp() as usize,
    };
    create_jwt_cookie(cookies, "refresh_token", &user_token, expiration)
}

async fn from_request_user_data(
    request: &Request<'_>,
) -> Result<UserData, (Status, Json<ApiResponse>)> {
    let cookies = request.cookies();
    let access_token = cookies
        .get_private("access_token")
        .map(|cookie| cookie.value().to_string());
    let refresh_token = cookies
        .get_private("refresh_token")
        .map(|cookie| cookie.value().to_string());

    if refresh_token.is_none() {
        return Err(ApiResponse::fail(
            Status::Unauthorized,
            "you are not authenticated",
            None,
        ));
    }

    let jwt_secret = std::env::var("JWT_SECRET").map_err(|e| {
        ApiResponse::fail(Status::InternalServerError, "internal server error", None)
    })?;
    let key = DecodingKey::from_secret(jwt_secret.as_bytes());
    if let Some(access_token) = access_token {
        match decode::<JWTData<UserData>>(&access_token, &key, &Validation::default()) {
            Ok(v) => return Ok(v.claims.data),
            // no need to return anything and the code can continue, if token expired it can be refreshed because the refresh token exists
            Err(e) if e.kind() == &jsonwebtoken::errors::ErrorKind::ExpiredSignature => {}
            // the difference with these next two patterns is that second one logs errors to console
            Err(e) if e.kind() == &jsonwebtoken::errors::ErrorKind::InvalidToken => {
                return Err(ApiResponse::fail(
                    Status::Unauthorized,
                    "you are not authenticated",
                    None,
                ));
            }
            Err(e) => {
                return Err(ApiResponse::fail(
                    Status::Unauthorized,
                    "authentication error",
                    Some(&e),
                ));
            }
        }
    };

    // TODO: refresh refresh token some time (7-14 days) before expiration
    // if we are here it means the token has expired or cookie for access doesn't exist
    let refresh_data = match decode::<JWTData<RefreshTokenData>>(
        &refresh_token.unwrap(),
        &key,
        &Validation::default(),
    ) {
        Ok(v) => v.claims.data,
        // the difference with these next two patterns is that second one logs errors to console
        Err(e)
            if e.kind() == &jsonwebtoken::errors::ErrorKind::InvalidToken
                || e.kind() == &jsonwebtoken::errors::ErrorKind::InvalidToken =>
        {
            return Err(ApiResponse::fail(
                Status::Unauthorized,
                "you are not authenticated",
                None,
            ));
        }
        Err(e) => {
            return Err(ApiResponse::fail(
                Status::Unauthorized,
                "authentication error",
                Some(&e),
            ));
        }
    };

    let pool = match request.guard::<&State<PgPool>>().await {
        Outcome::Success(pool) => pool,
        Outcome::Error(_) | Outcome::Forward(_) => {
            return Err(ApiResponse::fail(
                Status::InternalServerError,
                "internal server error",
                None,
            ));
        }
    };

    let expires_at = sqlx::query_scalar!(
        "SELECT expires_at FROM user_tokens WHERE user_id = $1 AND refresh_token = $2 ORDER BY created_at DESC LIMIT 1",
        refresh_data.user_id,
        refresh_data.refresh_token
    )
        .fetch_optional(pool.inner())
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

    if expires_at.is_none() || expires_at.unwrap() < Utc::now() {
        // we can ignore error because if it doesn't remove now it can later
        let _ = sqlx::query!(
            "DELETE FROM user_tokens WHERE user_id = $1 AND expires_at < NOW();",
            refresh_data.user_id
        )
        .fetch_optional(pool.inner())
        .await;
        cookies.remove_private(Cookie::from("access_token"));
        cookies.remove_private(Cookie::from("refresh_token"));
        return Err(ApiResponse::fail(
            Status::Unauthorized,
            "you are not authenticated",
            None,
        ));
    }

    let user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE id = $1",
        refresh_data.user_id
    )
    .fetch_one(pool.inner())
    .await
    .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;
    let user_data: UserData = user.into();

    refresh_access_cookie(cookies, &user_data, Expiration::Session).await?;

    Ok(user_data)
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminData {
    type Error = (Status, Json<ApiResponse>);
    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let user = try_outcome!(request.guard::<UserData>().await);
        if user.admin {
            Outcome::Success(user.try_into().unwrap())
        } else {
            Outcome::Error((
                Status::Unauthorized,
                ApiResponse::fail(Status::Unauthorized, "you are not an admin", None),
            ))
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserData {
    type Error = (Status, Json<ApiResponse>);
    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let request_user = request
            .local_cache_async(async { from_request_user_data(request).await })
            .await;

        match request_user {
            Ok(v) => Outcome::Success(v.clone()),
            Err(e) => Outcome::Error((e.0, e.clone())),
        }
    }
}
