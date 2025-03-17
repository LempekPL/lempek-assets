use serde::{Serialize, Deserialize};
use sqlx::{FromRow};
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub login: String,
    pub password_hash: String,
    pub allow_upload: bool,
}

#[derive(FromRow)]
pub struct Folder {
    pub id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
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
}