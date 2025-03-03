use crate::auth::AuthUser;
use rocket::{State, fs::TempFile, http::Status, post, response::status, serde::json::Json};
use sqlx::PgPool;
use std::{env, fs, path::PathBuf};
use rocket::form::Form;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, FromForm)]
pub struct UploadRequest<'a> {
    pub project_name: String,
    pub file: TempFile<'a>,
}

#[post("/upload", data = "<data>", format = "multipart/form-data")]
pub async fn upload_file(
    auth_user: AuthUser,
    mut data: Form<UploadRequest<'_>>,
    pool: &State<PgPool>,
) -> Result<status::Accepted<String>, Status> {
    println!("1a");
    let upload_dir =
        if auth_user.login.to_ascii_lowercase() == "lempek" && data.project_name == "public" {
            String::from("../files/public")
        } else {
            println!("2a");
            let project = sqlx::query!(
                "SELECT id FROM projects WHERE owner_id = $1 AND name = $2",
                auth_user.user_id,
                data.project_name
            )
            .fetch_optional(pool.inner())
            .await
            .map_err(|_| Status::InternalServerError)?;

            println!("2b");
            if project.is_none() {
                return Err(Status::Forbidden);
            }
            println!("2c");
            format!("../files/users/{}/{}", auth_user.login, data.project_name)
        };

    println!("1b");
    if !PathBuf::from(&upload_dir).exists() {
        fs::create_dir_all(&upload_dir).map_err(|_| Status::InternalServerError)?;
    }
    println!("1c");

    let file_name = data.file.name().unwrap_or("unknown").to_string();
    let file_path = format!("{}/{}", upload_dir, file_name);

    println!("1d");
    data.file.persist_to(&file_path)
        .await
        .map_err(|e| { dbg!(e); Status::InternalServerError })?;

    println!("1e");
    Ok(status::Accepted(format!("File uploaded: {}", file_path)))
}
