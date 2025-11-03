use rocket::http::Status;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::types::chrono::{DateTime, Utc};
use std::error::Error;
use std::panic::Location;
use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiResponse {
    pub success: bool,
    pub err_id: Option<String>,
    pub detail: Option<String>,
}

impl ApiResponse {
    fn print_err(loc: &Location, msg: &str, error: Option<&dyn Error>) -> String {
        let err_id = Uuid::now_v7().to_string();
        if let Some(err) = error {
            log::error!(
                "{}\n[{}:{}:{}] {}\n{}",
                err_id,
                loc.file(),
                loc.line(),
                loc.column(),
                msg,
                err
            );
        }
        // else {
        //     log::error!(
        //         "{}\n[{}:{}:{}] {}",
        //         err_id,
        //         loc.file(),
        //         loc.line(),
        //         loc.column(),
        //         msg,
        //     );
        // }
        err_id
    }

    pub fn success() -> Json<Self> {
        Json(Self {
            success: true,
            err_id: None,
            detail: None,
        })
    }
    pub fn success_with(message: impl Into<String>) -> Json<Self> {
        Json(Self {
            success: true,
            err_id: None,
            detail: Some(message.into()),
        })
    }

    #[track_caller]
    pub fn no_success(message: impl Into<String>, error: Option<&(dyn Error)>) -> Json<Self> {
        let msg = message.into();
        let err_id = Self::print_err(Location::caller(), &msg, error);
        Json(Self {
            success: false,
            err_id: Some(err_id),
            detail: Some(msg),
        })
    }

    #[track_caller]
    pub fn fail(
        status: Status,
        message: impl Into<String>,
        error: Option<&(dyn Error)>,
    ) -> (Status, Json<Self>) {
        let msg = message.into();
        let err_id = Self::print_err(Location::caller(), &msg, error);
        (
            status,
            Json(Self {
                success: false,
                err_id: Some(err_id),
                detail: Some(msg),
            }),
        )
    }
}

#[derive(Serialize, Deserialize, FromRow, Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub login: String,
    pub username: String,
    pub password: String,
    pub admin: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, FromRow, Debug, Clone)]
pub struct UserToken {
    pub id: Uuid,
    // TODO: ugh option
    pub user_id: Option<Uuid>,
    pub refresh_token: Uuid,
    // TODO: change to DateTime<Utc>, ugh and the option bruh
    pub expires_at: NaiveDateTime,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(FromRow, Serialize, Debug, Clone)]
pub struct Folder {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub owner_id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

 #[derive(FromRow, Serialize, Debug, Clone)]
pub struct File {
    pub id: Uuid,
    pub folder_id: Option<Uuid>,
    pub owner_id: Uuid,
    pub name: String,
    pub size: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(FromRow, Debug, Clone)]
pub struct Permission {
    pub id: Uuid,
    pub folder_id: Option<Uuid>,
    pub user_id: Uuid,
    pub read: bool,
    pub modify: bool,
    pub edit: bool,
}
