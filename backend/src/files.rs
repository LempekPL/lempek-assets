use crate::auth::AuthUser;
use rocket::form::Form;
use rocket::http::uncased::UncasedStr;
use rocket::{State, fs::TempFile, http::Status, post, response::status, serde::json::Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::types::chrono::Utc;
use std::path::Path;
use std::{env, fs, path::PathBuf};
use uuid::Uuid;

#[derive(Debug, FromForm)]
pub struct UploadRequest<'a> {
    pub project_name: Option<String>,
    pub name: Option<String>,
    pub file: TempFile<'a>,
}

// fn generate_name() -> String {
//     let file_ext = data
//         .file
//         .name()
//         .and_then(|name| PathBuf::from(name).extension().map(|ext| ext.to_string_lossy().to_string()))
//         .unwrap_or_default();
//
//     let now = Utc::now().format("%Y-%m-%d_%H%M%S").to_string();
//     let new_file_name = format!("{}.{}", now, file_ext);
//     return new_file_name;
// }

#[post("/upload", data = "<data>", format = "multipart/form-data")]
pub async fn upload_file(
    auth_user: AuthUser,
    mut data: Form<UploadRequest<'_>>,
    pool: &State<PgPool>,
) -> Result<status::Accepted<String>, Status> {
    let upload_dir = if auth_user.login == "Lempek"
        && data
            .project_name
            .as_ref()
            .is_some_and(|name| name == "public")
    {
        String::from("../files/public")
    } else {
        let project = sqlx::query!(
            "SELECT id FROM asset_space WHERE owner_id = $1 AND name = $2",
            auth_user.user_id,
            data.project_name
        )
        .fetch_optional(pool.inner())
        .await
        .map_err(|_| Status::InternalServerError)?;
        if project.is_none() {
            return Err(Status::Forbidden);
        }
        if let Some(project_name) = data.project_name.as_ref() {
            format!("../files/users/{}/{}", auth_user.login, project_name)
        } else {
            format!("../files/users/{}", auth_user.login)
        }
    };

    if !PathBuf::from(&upload_dir).exists() {
        fs::create_dir_all(&upload_dir).map_err(|_| Status::InternalServerError)?;
    }

    let ext = &data
        .file
        .raw_name()
        .and_then(|v| {
            Some(
                Path::new(&v.dangerous_unsafe_unsanitized_raw().to_string())
                    .extension()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
            )
        })
        .unwrap_or_default();
    let file_name = data
        .name
        .clone()
        .unwrap_or(data.file.name().unwrap_or("unknown").to_string());
    let mut file_path = format!("{}/{}.{}", upload_dir, file_name, ext);
    let mut attempts = 0;

    while Path::new(&file_path).exists() && attempts < 100 {
        attempts += 1;
        let ext = PathBuf::from(file_path)
            .extension()
            .map(|ext| ext.to_string_lossy().to_string())
            .unwrap_or_default();
        file_path = format!("{}/{}_({}).{}", upload_dir, file_name, attempts, ext);
    }

    if attempts >= 100 {
        return Err(Status::InternalServerError);
    }

    data.file
        .move_copy_to(&file_path)
        .await
        .map_err(|e| Status::InternalServerError)?;

    Ok(status::Accepted("File uploaded".to_string()))
}
