use rocket::http::Status;
use rocket::serde::json::Json;
use serde::{Serialize, Deserialize};
use sqlx::{FromRow};
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;


#[derive(Serialize, Deserialize, Debug)]
pub struct ApiResponse {
    pub success: bool,
    pub detail: Option<String>,
}

impl ApiResponse {
    pub fn success() -> Json<Self> {
        Json(Self {
            success: true,
            detail: None,
        })
    }
    pub fn success_with(message: impl Into<String>) -> Json<Self> {
        Json(Self {
            success: true,
            detail: Some(message.into()),
        })
    }
    pub fn no_success(message: impl Into<String>) -> Json<Self> {
        Json(Self {
            success: false,
            detail: Some(message.into()),
        })
    }
    pub fn fail(status: Status, message: impl Into<String>) -> (Status, Json<Self>) {
        (
            status,
            Json(Self {
                success: false,
                detail: Some(message.into()),
            }),
        )
    }
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub login: String,
    pub password: String,
    pub admin: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(FromRow)]
pub struct Folder {
    pub id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(FromRow)]
pub struct File {
    pub id: Uuid,
    pub folder_id: Option<Uuid>,
    pub user_id: Uuid,
    pub filename: String,
    pub filepath: String,
    pub size: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(FromRow)]
pub struct Permission {
    pub id: Uuid,
    pub folder_id: Option<Uuid>,
    pub user_id: Uuid,
    pub read: bool,
    pub modify: bool,
    pub edit: bool,
}

#[derive(FromRow)]
pub struct Perms {
    pub read: bool,
    pub modify: bool,
    pub edit: bool,
}