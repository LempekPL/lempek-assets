use crate::models::User;
use bcrypt::{DEFAULT_COST, hash, verify};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rocket::request::{FromRequest, Outcome};
use rocket::{Request, State, http::Status, post, serde::json::Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::Duration;
use chrono::Utc;
use uuid::Uuid;

fn encode_auth(user: impl Into<AuthUser>) -> Option<String> {
    let jwt_secret = std::env::var("JWT_SECRET").ok()?;
    encode::<AuthUser>(
        &Header::default(),
        &user.into(),
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .ok()
}

#[derive(Serialize, Deserialize)]
pub struct TokenResponse {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub login: String,
    pub allow_upload: bool,
    pub exp: usize,
}

impl Into<AuthUser> for User {
    fn into(self) -> AuthUser {
        AuthUser {
            user_id: self.id,
            login: self.login,
            allow_upload: self.allow_upload,
            exp: (Utc::now() + Duration::from_secs(60 * 60 * 24)).timestamp() as usize,
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
) -> Result<Json<TokenResponse>, Status> {
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE login = $1", data.login)
        .fetch_one(pool.inner())
        .await
        .map_err(|_| Status::Unauthorized)?;

    if verify(&data.password, &user.password_hash).map_err(|_| Status::InternalServerError)? {
        let token = encode_auth(user).ok_or(Status::InternalServerError)?;
        Ok(Json(TokenResponse { token }))
    } else {
        Err(Status::Unauthorized)
    }
}

#[derive(Serialize, Deserialize)]
pub struct RegisterData {
    pub login: String,
    pub password: String,
}

#[post("/register", format = "json", data = "<data>")]
pub async fn register(
    data: Json<RegisterData>,
    pool: &State<PgPool>,
) -> Result<Json<TokenResponse>, Status> {
    let existing_user = sqlx::query!("SELECT id FROM users WHERE login = $1", data.login)
        .fetch_optional(pool.inner())
        .await
        .map_err(|_| Status::InternalServerError)?;

    if existing_user.is_some() {
        return Err(Status::Conflict);
    }

    let hashed_password =
        hash(&data.password, DEFAULT_COST).map_err(|_| Status::InternalServerError)?;

    sqlx::query!(
        "INSERT INTO users (login, password_hash) VALUES ($1, $2)",
        data.login,
        hashed_password
    )
    .execute(pool.inner())
    .await
    .map_err(|_| Status::InternalServerError)?;

    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE login = $1", data.login)
        .fetch_one(pool.inner())
        .await
        .map_err(|_| Status::Unauthorized)?;
    let token = encode_auth(user).ok_or(Status::InternalServerError)?;
    Ok(Json(TokenResponse { token }))
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let token = req
            .headers()
            .get_one("Authorization")
            .map(|t| t.replace("Bearer ", ""));
        let Ok(jwt_secret) = std::env::var("JWT_SECRET") else {
            return Outcome::Error((Status::InternalServerError, ()));
        };

        if let Some(token) = token {
            let key = DecodingKey::from_secret(jwt_secret.as_bytes());
            let result = decode::<AuthUser>(&token, &key, &Validation::default());

            if let Ok(decoded) = result {
                if decoded.claims.exp > Utc::now().timestamp() as usize {
                    return Outcome::Success(decoded.claims);
                }
            }
        }

        Outcome::Error((Status::Unauthorized, ()))
    }
}
