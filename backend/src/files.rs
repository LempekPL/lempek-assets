use crate::auth::AuthUser;
use crate::models::User;
use crate::{ADMIN_UUID, FILES_DIR};
use chrono::DateTime;
use chrono::Utc;
use rocket::form::Form;
use rocket::{State, fs::TempFile, http::Status, post, serde::json::Json};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::path::Path;
use std::{fs, path::PathBuf};
use uuid::Uuid;

#[derive(Debug, FromForm)]
pub struct UploadRequest<'a> {
    pub file: TempFile<'a>,
    pub folder: Option<String>, // UUID
    pub name: Option<String>,
    pub overwrite: Option<bool>,
    pub public_file: Option<bool>,
}

#[post("/upload", data = "<data>", format = "multipart/form-data")]
pub async fn upload_file<'a>(
    auth_user: AuthUser,
    mut data: Form<UploadRequest<'a>>,
    pool: &'a State<PgPool>,
) -> Result<String, (Status, Option<&'a str>)> {
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", auth_user.user_id)
        .fetch_one(pool.inner())
        .await
        .map_err(|_| (Status::InternalServerError, None))?;
    if !user.allow_upload {
        return Err((Status::Unauthorized, Some("No upload permission")));
    }
    let mut upload_dir = FILES_DIR.get().unwrap().to_string();
    let folder_uuid;
    if ADMIN_UUID.get().unwrap() == &user.id && data.public_file.unwrap_or(false) {
        if data.folder.is_none() {
            upload_dir += "public";
        }
        // let folder: Uuid = sqlx::query_scalar!(
        //     "WITH ins AS (
        //             INSERT INTO folders (name, parent_id, user_id)
        //             SELECT 'public', NULL, $1
        //             WHERE NOT EXISTS (
        //                 SELECT 1
        //                 FROM folders
        //                 WHERE name = 'public'
        //                   AND parent_id IS NULL
        //                   AND user_id = $1
        //             )
        //             RETURNING id
        //         )
        //         SELECT id FROM ins
        //         UNION ALL
        //         SELECT id FROM folders
        //         WHERE name = 'public'
        //           AND parent_id IS NULL
        //           AND user_id = $1
        //           LIMIT 1;",
        //     user.id
        // )
        // .fetch_one(pool.inner())
        // .await
        // .map_err(|_| (Status::InternalServerError, None))?
        // .ok_or((Status::InternalServerError, None))?;

        // folder_uuid = Uuid::parse_str(&folder_id).map_err(|_| (Status::InternalServerError, None))?;
        // String::from("../files/public")
    } else {
        upload_dir += &user.login;
        //     if let Some(ref folder_id) = data.folder {
        //         folder_uuid =
        //             Uuid::parse_str(&folder_id).map_err(|_| (Status::InternalServerError, None))?;
        //         let str_path = sqlx::query_as!(
        //     FolderPath,
        //     "SELECT get_folder_path($1) as path;",
        //     folder_uuid
        // )
        //             .fetch_optional(pool.inner())
        //             .await
        //             .map_err(|_| (Status::InternalServerError, None))?
        //             .ok_or((Status::BadRequest, None))?;
        //         path += str_path.path.ok_or((Status::BadRequest, None))?.as_str();
        //     };
        // path
    };
    if let Some(ref folder_id) = data.folder {
        folder_uuid =
            Uuid::parse_str(&folder_id).map_err(|_| (Status::InternalServerError, None))?;
        let str_path = sqlx::query_scalar!("SELECT get_folder_path($1) as path;", folder_uuid)
            .fetch_optional(pool.inner())
            .await
            .map_err(|_| (Status::InternalServerError, None))?
            .ok_or((Status::BadRequest, None))?;
        upload_dir += str_path.ok_or((Status::BadRequest, None))?.as_str();
    };

    if !data.overwrite.unwrap_or(false) {
        // let file = sqlx::query_as!(
        //     File,
        //     "SELECT
        // );
    }

    if !PathBuf::from(&upload_dir).exists() {
        fs::create_dir_all(&upload_dir).map_err(|_| (Status::InternalServerError, None))?;
    }

    let invalid_chars = [
        '<', '>', ':', '"', '/', '\\', '|', '?', '*', ',', ';', '=', '(', ')', '&', '#', '\'',
    ];
    let reserved_names = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    let name = match &data.name {
        None => Ok(data
            .file
            .raw_name()
            .unwrap()
            .dangerous_unsafe_unsanitized_raw()
            .as_str()),
        Some(name) => {
            if name
                .chars()
                .any(|c| invalid_chars.contains(&c) || c.is_control())
            {
                return Err((
                    Status::BadRequest,
                    Some(
                        "You used illegal character in name\nList of illegal chars: '<', '>', ':', '\"', '/', '\\', '|', '?', '*', ',', ';', '=', '(', ')', '&', '#', '\''",
                    ),
                ));
            }
            Ok(name.as_str())
        }
    }?;
    let file_name = Path::new(name)
        .file_name()
        .to_owned()
        .ok_or((Status::InternalServerError, None))?
        .to_string_lossy()
        .to_string();

    if file_name
        .chars()
        .any(|c| invalid_chars.contains(&c) || c.is_control())
    {
        return Err((
            Status::BadRequest,
            Some(
                "You used illegal character in name\nList of illegal chars: '<', '>', ':', '\"', '/', '\\', '|', '?', '*', ',', ';', '=', '(', ')', '&', '#', '\''",
            ),
        ));
    }
    if reserved_names
        .iter()
        .any(|&name| file_name.eq_ignore_ascii_case(name))
    {
        return Err((
            Status::BadRequest,
            Some(
                "You used illegal name\nList of illegal names: 'CON', 'PRN', 'AUX', 'NUL', 'COM1', 'COM2', 'COM3', 'COM4', 'COM5', 'COM6', 'COM7', 'COM8', 'COM9', 'LPT1', 'LPT2', 'LPT3', 'LPT4', 'LPT5', 'LPT6', 'LPT7', 'LPT8', 'LPT9'",
            ),
        ));
    }

    dbg!(&file_name);

    // let base_name = if let Some(ref name) = data.name {
    //     name.clone()
    // } else if let Some(raw) = data.file.raw_name() {
    //     raw.as_str().to_string()
    // } else {
    //     // Fallback: generate a unique name based on UUID and content-type extension.
    //     let ext = data
    //         .file
    //         .content_type()
    //         .and_then(|ct| ct.extension())
    //         .unwrap_or_default();
    //     format!("file_{}.{}", Uuid::new_v4(), ext)
    // };
    // let mut destination_path = format!("{}/{}", upload_dir, base_name);
    // if let Err(e) = data.file.persist_to(&destination_path).await {
    //     eprintln!("Failed to save file to {}: {}", destination_path, e);
    //     return Err(Status::InternalServerError);
    // }
    //

    // let size: i64 = match fs::metadata(&upload_dir) {
    //     Ok(metadata) => metadata.len() as i64,
    //     Err(_) => 0,
    // };
    // let insert_result = sqlx::query!(
    //     "INSERT INTO files (user_id, folder_id, filename, filepath, size) VALUES ($1, $2, $3, $4, $5)",
    //     user.id,
    //     folder_uuid,
    //     file_name,
    //     upload_dir.trim_start_matches("../files"),
    //     size
    // )
    //     .execute(pool.inner())
    //     .await
    //     .map_err(|_| Status::InternalServerError)?;
    Ok("yej".to_string())
}

#[derive(Debug, Deserialize, Serialize, FromRow)]
pub struct ShowFolder {
    pub id: Uuid,
    pub name: String,
    pub path: String,
    pub created_at: DateTime<Utc>,
}

#[get("/folders")]
pub async fn get_folders<'a>(
    auth_user: AuthUser,
    pool: &'a State<PgPool>,
) -> Result<Json<Vec<ShowFolder>>, (Status, Option<&'a str>)> {
    sqlx::query_as!(
        ShowFolder,
        r#"SELECT
            id,
            name,
            get_folder_path(id) as "path!",
            created_at
        FROM folders
        WHERE user_id = $1"#,
        auth_user.user_id
    )
    .fetch_all(pool.inner())
    .await
    .and_then(|v| Ok(Json(v)))
    .map_err(|_| (Status::InternalServerError, None))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateFolderRequest {
    pub parent_folder: Option<String>, // UUID
    pub name: String,
}

#[post("/folder", data = "<data>")]
pub async fn create_folder<'a>(
    auth_user: AuthUser,
    data: Json<CreateFolderRequest>,
    pool: &'a State<PgPool>,
) -> Result<String, (Status, Option<&'a str>)> {
    if data.name.is_empty() {
        return Err((Status::BadRequest, Some("Empty name")));
    }
    let parent_name;
    let parent_uuid = if let Some(ref parent_str) = data.parent_folder {
        let uuid = Uuid::parse_str(parent_str)
            .map_err(|_| (Status::BadRequest, Some("Invalid parent_folder UUID")))?;
        let parent_exists = sqlx::query_scalar!(
            "SELECT name FROM folders WHERE user_id = $1 AND id = $2",
            auth_user.user_id,
            uuid
        )
        .fetch_optional(pool.inner())
        .await
        .map_err(|_| (Status::InternalServerError, None))?;
        if parent_exists.is_none() {
            return Err((Status::BadRequest, Some("Parent folder does not exist")));
        }
        parent_name = parent_exists;
        Some(uuid)
    } else {
        parent_name = None;
        None
    };

    let invalid_chars = [
        '<', '>', ':', '"', '/', '\\', '|', '?', '*', ',', ';', '=', '(', ')', '&', '#', '\'',
    ];
    if data
        .name
        .chars()
        .any(|c| invalid_chars.contains(&c) || c.is_control())
    {
        return Err((
            Status::BadRequest,
            Some(
                "You used illegal character in name\nList of illegal chars: '<', '>', ':', '\"', '/', '\\', '|', '?', '*', ',', ';', '=', '(', ')', '&', '#', '\''",
            ),
        ));
    }

    let existing = sqlx::query_scalar!(
        "
        SELECT id FROM folders
        WHERE name = $1
          AND parent_id IS NOT DISTINCT FROM $2
          AND user_id = $3
        ",
        data.name,
        parent_uuid,
        auth_user.user_id
    )
    .fetch_optional(pool.inner())
    .await
    .map_err(|_| (Status::InternalServerError, None))?;

    if let Some(existing_id) = existing {
        return Ok(format!("Folder already exists: {}", existing_id));
    }

    let inserted_id = sqlx::query_scalar!(
        "
        INSERT INTO folders (name, parent_id, user_id)
        VALUES ($1, $2, $3)
        RETURNING id
        ",
        data.name,
        parent_uuid,
        auth_user.user_id
    )
    .fetch_one(pool.inner())
    .await
    .map_err(|_| (Status::InternalServerError, None))?;

    let dir_path = sqlx::query_scalar!(
        "SELECT get_folder_path(id) FROM folders WHERE id = $1",
        inserted_id,
    )
    .fetch_one(pool.inner())
    .await
    .map_err(|_| (Status::InternalServerError, None))?
    .unwrap();

    let new_dir = if parent_name.is_some_and(|v| &v == "public")
        && ADMIN_UUID.get().is_some_and(|v| v == &auth_user.user_id)
    {
        FILES_DIR.get().unwrap().to_string() + &dir_path
    } else {
        FILES_DIR.get().unwrap().to_string() + &auth_user.login + "/" + &dir_path
    };

    if !PathBuf::from(&new_dir).exists() {
        fs::create_dir_all(&new_dir).map_err(|_| (Status::InternalServerError, None))?;
    }

    Ok(format!("Created folder: {}", inserted_id))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeleteFolderRequest {
    pub id: String, // UUID
}

struct DeleteFolderDB {
    name: String,
    path: String,
}

#[delete("/folder", data = "<data>")]
pub async fn delete_folder<'a>(
    auth_user: AuthUser,
    data: Json<DeleteFolderRequest>,
    pool: &'a State<PgPool>,
) -> Result<String, (Status, Option<&'a str>)> {
    let uuid = Uuid::parse_str(data.id.as_str())
        .map_err(|_| (Status::BadRequest, Some("Invalid folder UUID")))?;

    let f_path = sqlx::query_as!(
        DeleteFolderDB,
        r#"SELECT get_folder_path(id) as "path!", name FROM folders WHERE id = $1 AND user_id = $2"#,
        uuid,
        auth_user.user_id
    )
    .fetch_optional(pool.inner())
    .await
    .map_err(|_| (Status::InternalServerError, None))?;

    if f_path.is_none() {
        return Err((Status::BadRequest, Some("Folder does not exist")));
    }
    let f_path = f_path.unwrap();

    let old_dir =
        if f_path.name == "public" && ADMIN_UUID.get().is_some_and(|v| v == &auth_user.user_id) {
            FILES_DIR.get().unwrap().to_string() + &f_path.path
        } else {
            FILES_DIR.get().unwrap().to_string() + &auth_user.login + "/" + &f_path.path
        };

    let result = sqlx::query!(
        "DELETE FROM folders WHERE id = $1 AND user_id = $2",
        uuid,
        auth_user.user_id
    )
    .execute(pool.inner())
    .await
    .map_err(|_| (Status::InternalServerError, None))?;

    if PathBuf::from(&old_dir).exists() {
        fs::remove_dir_all(&old_dir).map_err(|_| (Status::InternalServerError, None))?;
    }

    if result.rows_affected() > 0 {
        Ok(format!("Folder removed: {}", uuid))
    } else {
        Ok("Folder does not exist".to_string())
    }
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

// #[post("/upload", data = "<data>", format = "multipart/form-data")]
// pub async fn upload_file(
//     auth_user: AuthUser,
//     mut data: Form<UploadRequest<'_>>,
//     pool: &State<PgPool>,
// ) -> Result<status::Accepted<String>, Status> {
//     println!("sex");
//     dbg!(&data.file.name());
//     dbg!(&data.file.raw_name());
//     dbg!(&data.file.path());
//     dbg!(&data.file.content_type());
//     let upload_dir = String::from("../files/public");
//
//     if !PathBuf::from(&upload_dir).exists() {
//         fs::create_dir_all(&upload_dir).map_err(|_| Status::InternalServerError)?;
//     }
//
//     let ext = &data
//         .file
//         .raw_name()
//         .and_then(|v| {
//             Some(
//                 Path::new(&v.dangerous_unsafe_unsanitized_raw().to_string())
//                     .extension()
//                     .unwrap_or_default()
//                     .to_string_lossy()
//                     .to_string(),
//             )
//         })
//         .unwrap_or_default();
//     let file_name = data
//         .name
//         .clone()
//         .unwrap_or(data.file.name().unwrap_or("unknown").to_string());
//     let mut file_path = format!("{}/{}.{}", upload_dir, file_name, ext);
//     let mut attempts = 0;
//
//     while Path::new(&file_path).exists() && attempts < 100 {
//         attempts += 1;
//         let ext = PathBuf::from(file_path)
//             .extension()
//             .map(|ext| ext.to_string_lossy().to_string())
//             .unwrap_or_default();
//         file_path = format!("{}/{}_({}).{}", upload_dir, file_name, attempts, ext);
//     }
//
//     if attempts >= 100 {
//         return Err(Status::InternalServerError);
//     }
//
//     data.file
//         .move_copy_to(&file_path)
//         .await
//         .map_err(|e| Status::InternalServerError)?;
//
//     Ok(status::Accepted("File uploaded".to_string()))
// }
