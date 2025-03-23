use crate::models::User;
use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rocket::http::{Cookie, CookieJar, SameSite};
use rocket::request::{FromRequest, Outcome};
use rocket::{Request, State, http::Status, post, serde::json::Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

fn set_cookie(cookies: &CookieJar<'_>, user: impl Into<AuthUser>) -> Result<(), Status> {
    let jwt_secret = std::env::var("JWT_SECRET").map_err(|_| Status::InternalServerError)?;
    let auth = &user.into();
    let token = encode::<AuthUser>(
        &Header::default(),
        auth,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|_| Status::InternalServerError)?;

    cookies.add_private(
        Cookie::build(("jwt_token", token))
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Lax)
            .expires(rocket::time::OffsetDateTime::from_unix_timestamp(auth.exp as i64).unwrap()),
    );
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct Success {
    success: bool,
}

#[derive(Serialize, Deserialize)]
pub struct AuthResponse {
    pub success: bool,
    pub message: Option<String>,
}

impl AuthResponse {
    fn success() -> Self {
        Self {
            success: true,
            message: None,
        }
    }
    fn no_success(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: Some(message.into()),
        }
    }
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
) -> Result<Json<AuthResponse>, Status> {
    let user = match sqlx::query_as!(User, "SELECT * FROM users WHERE login = $1", data.login)
        .fetch_one(pool.inner())
        .await
    {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => {
            return Ok(Json(AuthResponse::no_success("Wrong login or password")));
        }
        Err(_) => return Err(Status::InternalServerError),
    };

    if verify(&data.password, &user.password_hash).map_err(|_| Status::InternalServerError)? {
        set_cookie(cookies, user)?;
        Ok(Json(AuthResponse::success()))
    } else {
        Ok(Json(AuthResponse::no_success("Wrong login or password")))
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
    cookies: &CookieJar<'_>,
) -> Result<Json<Success>, Status> {
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
    set_cookie(cookies, user)?;
    Ok(Json(Success { success: true }))
}

#[post("/logout")]
pub async fn logout(_auth: AuthUser, cookies: &CookieJar<'_>) -> Result<Json<Success>, Status> {
    cookies.remove_private(Cookie::from("jwt_token"));
    Ok(Json(Success { success: true }))
}

#[derive(Serialize, Deserialize)]
struct UserData {
    pub user_id: Uuid,
    pub login: String,
    pub allow_upload: bool,
}

#[get("/user")]
pub async fn get_user(auth: AuthUser) -> Result<Json<UserData>, Status> {
    Ok(Json(UserData {
        user_id: auth.user_id,
        login: auth.login,
        allow_upload: auth.allow_upload,
    }))
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = ();

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
            None => return Outcome::Error((Status::Unauthorized, ())),
        };

        let Ok(jwt_secret) = std::env::var("JWT_SECRET") else {
            return Outcome::Error((Status::InternalServerError, ()));
        };
        let key = DecodingKey::from_secret(jwt_secret.as_bytes());
        match decode::<AuthUser>(&token, &key, &Validation::default()) {
            Ok(token_data) => {
                let now = Utc::now().timestamp() as usize;
                if token_data.claims.exp > now {
                    Outcome::Success(token_data.claims)
                } else {
                    Outcome::Error((Status::Unauthorized, ()))
                }
            }
            Err(_) => Outcome::Error((Status::Unauthorized, ())),
        }
    }
}
