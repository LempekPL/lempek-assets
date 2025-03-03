use crate::models::User;
use bcrypt::{DEFAULT_COST, hash, verify};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, TokenData, Validation, decode, encode};
use rocket::request::{FromRequest, Outcome};
use rocket::{Request, State, http::Status, post, response::status, serde::json::Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::types::chrono;
use sqlx::types::chrono::Utc;
use std::time::Duration;
use uuid::{Timestamp, Uuid};

#[derive(Serialize, Deserialize)]
pub struct LoginData {
    pub login: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct TokenResponse {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub login: String,
    pub exp: usize,
}

impl Into<AuthUser> for User {
    fn into(self) -> AuthUser {
        AuthUser {
            user_id: self.id,
            login: self.login,
            exp: (Utc::now() + Duration::from_secs(60 * 60 * 24)).timestamp() as usize,
        }
    }
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

    let jwt_secret = std::env::var("JWT_SECRET").map_err(|_| Status::InternalServerError)?;
    if verify(&data.password, &user.password_hash).map_err(|_| Status::InternalServerError)? {
        let token = encode::<AuthUser>(
            &Header::default(),
            &user.into(),
            &EncodingKey::from_secret(jwt_secret.as_bytes()),
        )
        .map_err(|_| Status::InternalServerError)?;

        Ok(Json(TokenResponse { token }))
    } else {
        Err(Status::Unauthorized)
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let token = req.headers().get_one("Authorization");
        let jwt_secret = std::env::var("JWT_SECRET").unwrap();
        if let Some(token) = token {
            let key = DecodingKey::from_secret(jwt_secret.as_bytes());
            let result =
                decode::<AuthUser>(&token.replace("Bearer ", ""), &key, &Validation::default());
            if let Ok(decoded) = result {
                return Outcome::Success(decoded.claims);
            }
        }
        Outcome::Error((Status::Unauthorized, ()))
    }
}
