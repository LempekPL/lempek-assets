use crate::FILES_DIR;
use crate::auth::AuthUser;
use crate::models::{ApiResponse, Perms, User};
use chrono::DateTime;
use chrono::Utc;
use rocket::form::Form;
use rocket::{State, fs::TempFile, http::Status, post, serde::json::Json};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::path::Path;
use std::{fs, path::PathBuf};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct NewFolderData {
    pub name: String,
    pub parent: Option<Uuid>,
}

#[post("/folder", format = "json", data = "<data>")]
pub async fn create_folder(
    data: Json<NewFolderData>,
    pool: &State<PgPool>,
    auth: Result<AuthUser, (Status, Json<ApiResponse>)>,
) -> Result<(Status, Json<ApiResponse>), (Status, Json<ApiResponse>)> {
    let auth = auth?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|_| ApiResponse::fail(Status::InternalServerError, "database error"))?;

    if !auth.admin {
        let perms = if let Some(parent) = data.parent {
            sqlx::query_as!(
                Perms,
                "SELECT read, modify, edit FROM permissions WHERE user_id = $1 AND folder_id = $2",
                auth.user_id,
                parent
            )
            .fetch_optional(&mut *tx)
            .await
            .map_err(|_| ApiResponse::fail(Status::InternalServerError, "database error"))?
        } else {
            sqlx::query_as!(Perms,
            "SELECT read, modify, edit FROM permissions WHERE user_id = $1 AND folder_id IS NULL",
            auth.user_id,
            )
                .fetch_optional(&mut *tx)
                .await
                .map_err(|_| ApiResponse::fail(Status::InternalServerError, "database error"))?
        };

        if perms.is_none() || !perms.as_ref().unwrap().edit {
            return Err(ApiResponse::fail(
                Status::Forbidden,
                "no permissions to edit contents of this folder",
            ));
        }
    }

    let folder_id = sqlx::query_scalar!(
        "INSERT INTO folders (name, parent_id, owner_id) VALUES ($1, $2, $3) RETURNING id",
        data.name,
        data.parent,
        auth.user_id
    )
    .fetch_one(&mut *tx)
    .await;

    let folder_id = match folder_id {
        Ok(id) => id,
        Err(sqlx::Error::Database(v)) if v.is_unique_violation() => {
            return Err(ApiResponse::fail(
                Status::Conflict,
                "folder with this name already exists",
            ));
        }
        Err(v) => {
            return Err(ApiResponse::fail(
                Status::InternalServerError,
                "database error",
            ));
        }
    };

    sqlx::query!(
            "INSERT INTO permissions (user_id, folder_id, read, modify, edit) VALUES ($1, $2, TRUE, TRUE, TRUE)",
            auth.user_id,
            folder_id,
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| ApiResponse::fail(Status::InternalServerError, "database error"))?;

    tx.commit()
        .await
        .map_err(|_| ApiResponse::fail(Status::InternalServerError, "database error"))?;

    Ok((
        Status::Created,
        ApiResponse::success_with(format!(r#"created folder named: "{}""#, data.name)),
    ))
}

#[derive(Serialize, Deserialize)]
pub struct RemoveFolderData {
    pub id: Uuid,
}

#[delete("/folder", format = "json", data = "<data>")]
pub async fn delete_folder(
    data: Json<RemoveFolderData>,
    pool: &State<PgPool>,
    auth: Result<AuthUser, (Status, Json<ApiResponse>)>,
) -> Result<(Status, Json<ApiResponse>), (Status, Json<ApiResponse>)> {
    let auth = auth?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|_| ApiResponse::fail(Status::InternalServerError, "database error"))?;

    if !auth.admin {
        let perms = sqlx::query_as!(
            Perms,
            "SELECT read, modify, edit FROM permissions WHERE user_id = $1 AND folder_id = $2",
            auth.user_id,
            data.id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| ApiResponse::fail(Status::InternalServerError, "database error"))?;

        if perms.is_none() || !perms.as_ref().unwrap().modify {
            return Err(ApiResponse::fail(
                Status::Forbidden,
                "no permissions modify this folder",
            ));
        }
    }

    sqlx::query!(
            "DELETE FROM folders WHERE id = $1",
            data.id,
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| ApiResponse::fail(Status::InternalServerError, "database error"))?;

    tx.commit()
        .await
        .map_err(|_| ApiResponse::fail(Status::InternalServerError, "database error"))?;

    Ok((
        Status::NoContent,
        ApiResponse::success_with("deleted folder"),
    ))
}

//
// #[derive(Debug, FromForm)]
// pub struct UploadRequest<'a> {
//     pub file: TempFile<'a>,
//     pub folder: Option<String>, // UUID
//     pub name: Option<String>,
//     pub overwrite: Option<bool>,
// }
//
// #[post("/upload", data = "<data>", format = "multipart/form-data")]
// pub async fn upload_file<'a>(
//     auth_user: AuthUser,
//     mut data: Form<UploadRequest<'a>>,
//     pool: &'a State<PgPool>,
// ) -> Result<String, (Status, Option<&'a str>)> {
//     let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", auth_user.user_id)
//         .fetch_one(pool.inner())
//         .await
//         .map_err(|_| (Status::InternalServerError, None))?;
//     if !user.allow_upload {
//         return Err((Status::Unauthorized, Some("No upload permission")));
//     }
//     let mut upload_dir = FILES_DIR.get().unwrap().to_string();
//
//     let folder_uuid = match data.folder.as_ref() {
//         None => None,
//         Some(v) => Some(Uuid::parse_str(v).map_err(|_| (Status::BadRequest, None))?),
//     };
//     let folder_root_uuid = if let Some(folder_uuid) = folder_uuid {
//         sqlx::query_scalar!("SELECT get_folder_root($1)", folder_uuid)
//             .fetch_one(pool.inner())
//             .await
//             .map_err(|_| (Status::InternalServerError, None))?
//     } else {
//         None
//     };
//
//     let is_public = ADMIN_UUID.get().unwrap() == &user.id
//         && folder_root_uuid.is_some_and(|v| &v == PUBLIC_DIR_UUID.get().unwrap());
//     if !is_public {
//         upload_dir += &user.login;
//         upload_dir += "/";
//     };
//     if let Some(folder_uuid) = folder_uuid {
//         let str_path = sqlx::query_scalar!("SELECT path FROM folders WHERE id = $1;", folder_uuid)
//             .fetch_optional(pool.inner())
//             .await
//             .map_err(|_| (Status::InternalServerError, None))?
//             .ok_or((Status::BadRequest, None))?;
//         upload_dir += (str_path + "/").as_str();
//     }
//
//     if !data.overwrite.unwrap_or(false) {
//         // let file = sqlx::query_as!(
//         //     File,
//         //     "SELECT
//         // );
//     }
//
//     if !PathBuf::from(&upload_dir).exists() {
//         fs::create_dir_all(&upload_dir).map_err(|_| (Status::InternalServerError, None))?;
//     }
//     let invalid_chars = [
//         '<', '>', ':', '"', '/', '\\', '|', '?', '*', ',', ';', '=', '(', ')', '&', '#', '\'',
//     ];
//     let reserved_names = [
//         "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
//         "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
//     ];
//     let name = match &data.name {
//         None => Ok(data
//             .file
//             .raw_name()
//             .unwrap()
//             .dangerous_unsafe_unsanitized_raw()
//             .as_str()),
//         Some(name) => {
//             if name
//                 .chars()
//                 .any(|c| invalid_chars.contains(&c) || c.is_control())
//             {
//                 return Err((
//                     Status::BadRequest,
//                     Some(
//                         "You used illegal character in name\nList of illegal chars: '<', '>', ':', '\"', '/', '\\', '|', '?', '*', ',', ';', '=', '(', ')', '&', '#', '\''",
//                     ),
//                 ));
//             }
//             Ok(name.as_str())
//         }
//     }?;
//     let file_name = Path::new(name)
//         .file_name()
//         .to_owned()
//         .ok_or((Status::InternalServerError, None))?
//         .to_string_lossy()
//         .to_string();
//
//     if file_name
//         .chars()
//         .any(|c| invalid_chars.contains(&c) || c.is_control())
//     {
//         return Err((
//             Status::BadRequest,
//             Some(
//                 "You used illegal character in name\nList of illegal chars: '<', '>', ':', '\"', '/', '\\', '|', '?', '*', ',', ';', '=', '(', ')', '&', '#', '\''",
//             ),
//         ));
//     }
//     if reserved_names
//         .iter()
//         .any(|&name| file_name.eq_ignore_ascii_case(name))
//     {
//         return Err((
//             Status::BadRequest,
//             Some(
//                 "You used illegal name\nList of illegal names: 'CON', 'PRN', 'AUX', 'NUL', 'COM1', 'COM2', 'COM3', 'COM4', 'COM5', 'COM6', 'COM7', 'COM8', 'COM9', 'LPT1', 'LPT2', 'LPT3', 'LPT4', 'LPT5', 'LPT6', 'LPT7', 'LPT8', 'LPT9'",
//             ),
//         ));
//     }
//     let file_dir = upload_dir + &file_name;
//     if let Err(e) = data.file.persist_to(&file_dir).await {
//         eprintln!("Failed to save file: {}", e);
//         dbg!(&file_dir);
//         return Err((Status::InternalServerError, Some("Failed to save file")));
//     }
//
//     let size: i64 = match fs::metadata(&file_dir) {
//         Ok(metadata) => metadata.len() as i64,
//         Err(_) => 0,
//     };
//     let insert_result = sqlx::query!(
//         "INSERT INTO files (user_id, folder_id, name, path, size) VALUES ($1, $2, $3, $4, $5)",
//         user.id,
//         folder_uuid,
//         file_name,
//         file_dir.trim_start_matches("../files"),
//         size
//     )
//     .execute(pool.inner())
//     .await
//     .map_err(|_| (Status::InternalServerError, None))?;
//     Ok(format!(
//         "Created file: {}",
//         file_dir.trim_start_matches("../files")
//     ))
// }
//
// #[derive(Debug, Deserialize, Serialize, FromRow)]
// pub struct ShowFile {
//     pub id: Uuid,
//     pub folder_id: Option<Uuid>,
//     pub name: String,
//     pub path: String,
//     pub size: Option<i64>,
//     pub created_at: DateTime<Utc>,
// }
//
// #[get("/files")]
// pub async fn get_files<'a>(
//     auth_user: AuthUser,
//     pool: &'a State<PgPool>,
// ) -> Result<Json<Vec<ShowFile>>, (Status, Option<&'a str>)> {
//     let a = sqlx::query_as!(
//         ShowFile,
//         "SELECT id, folder_id, name, path, size, created_at FROM files WHERE user_id = $1",
//         auth_user.user_id
//     )
//     .fetch_all(pool.inner())
//     .await
//     .map_err(|_| (Status::InternalServerError, None))?;
//     Ok(Json(a))
// }
//
// // #[derive(Debug, FromForm)]
// // pub struct UploadRequest<'a> {
// //     pub file: TempFile<'a>,
// //     pub folder: Option<String>, // UUID
// //     pub name: Option<String>,
// //     pub overwrite: Option<bool>,
// // }
// //
// // #[post("/files")]
// // pub async fn upload_file<'a>(
// //     auth_user: AuthUser,
// //     mut data: Form<UploadRequest<'a>>,
// //     pool: &'a State<PgPool>,
// // ) -> Result<String, (Status, Option<&'a str>)> {
//
// // #[derive(Debug, FromForm)]
// // pub struct UploadRequest<'a> {
// //     pub file: TempFile<'a>,
// //     pub folder: Option<String>, // UUID
// //     pub name: Option<String>,
// //     pub overwrite: Option<bool>,
// // }
// //
// // #[post("/upload", data = "<data>", format = "multipart/form-data")]
// // pub async fn upload_file<'a>(
// //     auth_user: AuthUser,
// //     mut data: Form<UploadRequest<'a>>,
// //     pool: &'a State<PgPool>,
// // ) -> Result<String, (Status, Option<&'a str>)> {
// //
// // }
//
// #[derive(Debug, Deserialize, Serialize, FromRow)]
// pub struct ShowFolder {
//     pub id: Uuid,
//     pub parent_id: Option<Uuid>,
//     pub name: String,
//     pub path: String,
//     pub created_at: DateTime<Utc>,
// }
//
// #[get("/folders")]
// pub async fn get_folders<'a>(
//     auth_user: AuthUser,
//     pool: &'a State<PgPool>,
// ) -> Result<Json<Vec<ShowFolder>>, (Status, Option<&'a str>)> {
//     sqlx::query_as!(
//         ShowFolder,
//         "SELECT id, parent_id, name, path, created_at
//         FROM folders
//         WHERE user_id = $1",
//         auth_user.user_id
//     )
//     .fetch_all(pool.inner())
//     .await
//     .and_then(|v| Ok(Json(v)))
//     .map_err(|_| (Status::InternalServerError, None))
// }
//
// #[get("/folder?<id>")]
// pub async fn get_folder<'a>(
//     auth_user: AuthUser,
//     id: String,
//     pool: &'a State<PgPool>,
// ) -> Result<Json<ShowFolder>, (Status, Option<&'a str>)> {
//     let uuid = Uuid::parse_str(id.as_str())
//         .map_err(|_| (Status::BadRequest, Some("Invalid folder UUID")))?;
//     sqlx::query_as!(
//         ShowFolder,
//         "SELECT id, parent_id, name, path, created_at
//         FROM folders
//         WHERE user_id = $1 AND id = $2",
//         auth_user.user_id, uuid
//     )
//         .fetch_one(pool.inner())
//         .await
//         .and_then(|v| Ok(Json(v)))
//         .map_err(|_| (Status::InternalServerError, None))
// }
//
// #[derive(Debug, Deserialize, Serialize)]
// pub struct CreateFolderRequest {
//     pub parent_folder: Option<String>, // UUID
//     pub name: String,
// }
//
// #[post("/folder", data = "<data>")]
// pub async fn create_folder<'a>(
//     auth_user: AuthUser,
//     data: Json<CreateFolderRequest>,
//     pool: &'a State<PgPool>,
// ) -> Result<String, (Status, Option<&'a str>)> {
//     if data.name.is_empty() {
//         return Err((Status::BadRequest, Some("Empty name")));
//     }
//     let parent_name;
//     let parent_uuid = if let Some(ref parent_str) = data.parent_folder {
//         let uuid = Uuid::parse_str(parent_str)
//             .map_err(|_| (Status::BadRequest, Some("Invalid parent_folder UUID")))?;
//         let parent_exists = sqlx::query_scalar!(
//             "SELECT name FROM folders WHERE user_id = $1 AND id = $2",
//             auth_user.user_id,
//             uuid
//         )
//         .fetch_optional(pool.inner())
//         .await
//         .map_err(|_| (Status::InternalServerError, None))?;
//         if parent_exists.is_none() {
//             return Err((Status::BadRequest, Some("Parent folder does not exist")));
//         }
//         parent_name = parent_exists;
//         Some(uuid)
//     } else {
//         parent_name = None;
//         None
//     };
//
//     let invalid_chars = [
//         '<', '>', ':', '"', '/', '\\', '|', '?', '*', ',', ';', '=', '(', ')', '&', '#', '\'',
//     ];
//     if data
//         .name
//         .chars()
//         .any(|c| invalid_chars.contains(&c) || c.is_control())
//     {
//         return Err((
//             Status::BadRequest,
//             Some(
//                 "You used illegal character in name\nList of illegal chars: '<', '>', ':', '\"', '/', '\\', '|', '?', '*', ',', ';', '=', '(', ')', '&', '#', '\''",
//             ),
//         ));
//     }
//
//     let existing = sqlx::query_scalar!(
//         "
//         SELECT id FROM folders
//         WHERE name = $1
//           AND parent_id IS NOT DISTINCT FROM $2
//           AND user_id = $3
//         ",
//         data.name,
//         parent_uuid,
//         auth_user.user_id
//     )
//     .fetch_optional(pool.inner())
//     .await
//     .map_err(|_| (Status::InternalServerError, None))?;
//
//     if let Some(existing_id) = existing {
//         return Ok(format!("Folder already exists: {}", existing_id));
//     }
//
//     let inserted_id = sqlx::query_scalar!(
//         "
//         INSERT INTO folders (name, parent_id, user_id)
//         VALUES ($1, $2, $3)
//         RETURNING id
//         ",
//         data.name,
//         parent_uuid,
//         auth_user.user_id
//     )
//     .fetch_one(pool.inner())
//     .await
//     .map_err(|_| (Status::InternalServerError, None))?;
//
//     let dir_path = sqlx::query_scalar!("SELECT path FROM folders WHERE id = $1", inserted_id,)
//         .fetch_one(pool.inner())
//         .await
//         .map_err(|_| (Status::InternalServerError, None))?;
//
//     let new_dir = if parent_name.is_some_and(|v| &v == "public")
//         && ADMIN_UUID.get().is_some_and(|v| v == &auth_user.user_id)
//     {
//         FILES_DIR.get().unwrap().to_string() + &dir_path
//     } else {
//         FILES_DIR.get().unwrap().to_string() + &auth_user.login + "/" + &dir_path
//     };
//
//     if !PathBuf::from(&new_dir).exists() {
//         fs::create_dir_all(&new_dir).map_err(|_| (Status::InternalServerError, None))?;
//     }
//
//     Ok(format!("Created folder: {}", inserted_id))
// }
//
// #[derive(Debug, Deserialize, Serialize)]
// pub struct DeleteFolderRequest {
//     pub id: String, // UUID
// }
//
// struct DeleteFolderDB {
//     name: String,
//     path: String,
// }
//
// #[delete("/folder", data = "<data>")]
// pub async fn delete_folder<'a>(
//     auth_user: AuthUser,
//     data: Json<DeleteFolderRequest>,
//     pool: &'a State<PgPool>,
// ) -> Result<String, (Status, Option<&'a str>)> {
//     let uuid = Uuid::parse_str(data.id.as_str())
//         .map_err(|_| (Status::BadRequest, Some("Invalid folder UUID")))?;
//
//     let f_path = sqlx::query_as!(
//         DeleteFolderDB,
//         r#"SELECT path, name FROM folders WHERE id = $1 AND user_id = $2"#,
//         uuid,
//         auth_user.user_id
//     )
//     .fetch_optional(pool.inner())
//     .await
//     .map_err(|_| (Status::InternalServerError, None))?;
//
//     if f_path.is_none() {
//         return Err((Status::BadRequest, Some("Folder does not exist")));
//     }
//     let f_path = f_path.unwrap();
//
//     let old_dir =
//         if f_path.name == "public" && ADMIN_UUID.get().is_some_and(|v| v == &auth_user.user_id) {
//             FILES_DIR.get().unwrap().to_string() + &f_path.path
//         } else {
//             FILES_DIR.get().unwrap().to_string() + &auth_user.login + "/" + &f_path.path
//         };
//
//     let result = sqlx::query!(
//         "DELETE FROM folders WHERE id = $1 AND user_id = $2",
//         uuid,
//         auth_user.user_id
//     )
//     .execute(pool.inner())
//     .await
//     .map_err(|_| (Status::InternalServerError, None))?;
//
//     if PathBuf::from(&old_dir).exists() {
//         fs::remove_dir_all(&old_dir).map_err(|_| (Status::InternalServerError, None))?;
//     }
//
//     if result.rows_affected() > 0 {
//         Ok(format!("Folder removed: {}", uuid))
//     } else {
//         Ok("Folder does not exist".to_string())
//     }
// }
//
// #[derive(Debug, Serialize, Deserialize)]
// pub struct FolderItem {
//     pub id: Uuid,
//     pub folder_id: Option<Uuid>,
//     pub r#type: String,
//     pub name: String,
//     pub path: String,
//     pub size: Option<i64>,
//     pub created_at: DateTime<Utc>,
// }
//
// #[derive(Debug, Deserialize, Serialize)]
// pub struct GetItemsRequest {
//     pub id: String, // UUID
// }
//
// #[get("/items?<at>")]
// pub async fn get_items<'a>(
//     auth_user: AuthUser,
//     at: Option<String>,
//     // data: Json<GetItemsRequest>,
//     pool: &'a State<PgPool>,
// ) -> Result<Json<Vec<FolderItem>>, (Status, Option<&'a str>)> {
//     if at.is_none() {
//         return sqlx::query_as!(FolderItem,r#"
//         SELECT id as "id!", folder_id, name as "name!", path as "path!", 'file'::text AS "type!", created_at as "created_at!", size
//         FROM files WHERE user_id = $1
//         UNION ALL
//         SELECT id as "id!", parent_id AS folder_id, name as "name!", path as "path!", 'folder'::text AS "type!", created_at as "created_at!", NULL as size
//         FROM folders WHERE user_id = $1;
//         "#, auth_user.user_id)
//             .fetch_all(pool.inner())
//             .await
//             .and_then(|v| Ok(Json(v)))
//             .map_err(|_| (Status::InternalServerError, None));
//     }
//     let uuid = if at.as_ref().is_some_and(|v| v == "null") {
//         None
//     } else {
//         at.map(|v| v.parse::<Uuid>().ok()).ok_or((Status::BadRequest, Some("Invalid uuid")))?
//     };
//
//     sqlx::query_as!(FolderItem,r#"
//         SELECT id as "id!", folder_id, name as "name!", path as "path!", 'file'::text AS "type!", created_at as "created_at!", size
//         FROM files WHERE user_id = $1 AND folder_id IS NOT DISTINCT FROM $2
//         UNION ALL
//         SELECT id as "id!", parent_id AS folder_id, name as "name!", path as "path!", 'folder'::text AS "type!", created_at as "created_at!", NULL as size
//         FROM folders WHERE user_id = $1 AND parent_id IS NOT DISTINCT FROM $2;
//         "#, auth_user.user_id, uuid)
//         .fetch_all(pool.inner())
//         .await
//         .and_then(|v| Ok(Json(v)))
//         .map_err(|_| (Status::InternalServerError, None))
// }
