use crate::FILES_DIR;
use crate::auth::AuthUser;
use crate::models::{ApiResponse, File, Folder};
use rocket::form::Form;
use rocket::{State, fs::TempFile, http::Status, post, serde::json::Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::path::Path;
use std::{fs, path::PathBuf};
use uuid::Uuid;

type ApiResult<T = (Status, Json<ApiResponse>)> = Result<T, (Status, Json<ApiResponse>)>;

fn remove_last_path(s: &str) -> String {
    Path::new(s)
        .parent()
        .and_then(|p| p.to_str())
        .unwrap_or("")
        .trim_end_matches('/')
        .to_string()
}

fn check_name(name: &str) -> ApiResult<()> {
    let invalid_chars = [
        '<', '>', ':', '"', '/', '\\', '|', '?', '*', ',', ';', '=', '(', ')', '&', '#', '\'',
    ];
    if name
        .chars()
        .any(|c| invalid_chars.contains(&c) || c.is_control())
    {
        return Err(ApiResponse::fail(
            Status::Forbidden,
            "You used illegal character in name\nList of illegal chars: '<', '>', ':', '\"', '/', '\\', '|', '?', '*', ',', ';', '=', '(', ')', '&', '#', '\''",
            None,
        ));
    }
    let reserved_names = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    if reserved_names
        .iter()
        .any(|&res| name.eq_ignore_ascii_case(res))
    {
        return Err(ApiResponse::fail(
            Status::Forbidden,
            "You used illegal name\nList of illegal names: 'CON', 'PRN', 'AUX', 'NUL', 'COM1', 'COM2', 'COM3', 'COM4', 'COM5', 'COM6', 'COM7', 'COM8', 'COM9', 'LPT1', 'LPT2', 'LPT3', 'LPT4', 'LPT5', 'LPT6', 'LPT7', 'LPT8', 'LPT9'",
            None,
        ));
    }
    if name.len() > 255 {
        return Err(ApiResponse::fail(
            Status::Forbidden,
            "Name is too long",
            None,
        ));
    }
    Ok(())
}

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
) -> ApiResult {
    let auth = auth?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

    if !auth.admin {
        let edit = sqlx::query_scalar!(
            "SELECT edit FROM permissions
                WHERE user_id = $1 AND
                (($2::uuid IS NULL AND folder_id IS NULL) OR folder_id = $2)",
            auth.user_id,
            data.parent
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

        if !edit.is_some_and(|v| v) {
            return Err(ApiResponse::fail(
                Status::Forbidden,
                "no permissions to edit contents of this folder",
                None,
            ));
        }
    }

    check_name(&data.name)?;

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
                None,
            ));
        }
        Err(e) => {
            return Err(ApiResponse::fail(
                Status::InternalServerError,
                "database error",
                Some(&e),
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
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

    let path = sqlx::query_scalar!("SELECT * FROM get_folder_path($1)", folder_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?
        .unwrap_or_default();

    let mut base = PathBuf::from(FILES_DIR.get().unwrap());
    base.push(&path);

    fs::create_dir(&base).map_err(|e| {
        ApiResponse::fail(
            Status::InternalServerError,
            "error while creating folder",
            Some(&e),
        )
    })?;

    tx.commit().await.map_err(|dbe| {
        if let Err(e) = fs::remove_dir(&base) {
            return ApiResponse::fail(
                Status::InternalServerError,
                "error while removing created folder",
                Some(&e),
            );
        }
        ApiResponse::fail(Status::InternalServerError, "database error", Some(&dbe))
    })?;

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
) -> ApiResult {
    let auth = auth?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

    if !auth.admin {
        let modify = sqlx::query_scalar!(
            "SELECT modify FROM permissions WHERE user_id = $1 AND folder_id = $2",
            auth.user_id,
            data.id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

        if !modify.is_some_and(|v| v) {
            return Err(ApiResponse::fail(
                Status::Forbidden,
                "no permissions modify this folder",
                None,
            ));
        }
    }

    let path = sqlx::query_scalar!("SELECT * FROM get_folder_path($1)", data.id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?
        .unwrap_or_default();

    let mut base = PathBuf::from(FILES_DIR.get().unwrap());
    base.push(&path);

    sqlx::query!("DELETE FROM folders WHERE id = $1", data.id,)
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

    tx.commit()
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

    fs::remove_dir_all(base).map_err(|e| {
        ApiResponse::fail(
            Status::InternalServerError,
            "error while deleting folder",
            Some(&e),
        )
    })?;

    Ok((
        Status::NoContent,
        ApiResponse::success_with("deleted folder"),
    ))
}

#[derive(Serialize, Deserialize)]
pub struct EditFolderData {
    pub id: Uuid,
    pub name: String,
}

#[patch("/folder", format = "json", data = "<data>")]
pub async fn edit_folder(
    data: Json<EditFolderData>,
    pool: &State<PgPool>,
    auth: Result<AuthUser, (Status, Json<ApiResponse>)>,
) -> ApiResult {
    let auth = auth?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

    if !auth.admin {
        let modify = sqlx::query_scalar!(
            "SELECT modify FROM permissions WHERE user_id = $1 AND folder_id = $2",
            auth.user_id,
            data.id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

        if !modify.is_some_and(|v| v) {
            return Err(ApiResponse::fail(
                Status::Forbidden,
                "no permissions modify this folder",
                None,
            ));
        }
    }

    let old_path = sqlx::query_scalar!("SELECT * FROM get_folder_path($1)", data.id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?
        .unwrap_or_default();

    let result = sqlx::query!(
        "UPDATE folders SET name = $1 WHERE id = $2",
        data.name,
        data.id,
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(db_err) = &e
            && db_err.is_unique_violation()
        {
            ApiResponse::fail(
                Status::Conflict,
                "folder with this name already exists",
                None,
            )
        } else {
            ApiResponse::fail(Status::InternalServerError, "database error", Some(&e))
        }
    })?;

    if result.rows_affected() == 0 {
        return Err(ApiResponse::fail(
            Status::NotFound,
            "folder not found",
            None,
        ));
    }

    fs::rename(
        FILES_DIR.get().unwrap().to_string() + &old_path,
        FILES_DIR.get().unwrap().to_string() + &remove_last_path(&old_path) + "/" + &data.name,
    )
    .map_err(|e| {
        ApiResponse::fail(
            Status::InternalServerError,
            "error while renaming folder",
            Some(&e),
        )
    })?;

    tx.commit()
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

    Ok((
        Status::NoContent,
        ApiResponse::success_with("renamed folder"),
    ))
}

#[get("/folders/all")]
pub async fn get_all_folders(
    pool: &State<PgPool>,
    auth: Result<AuthUser, (Status, Json<ApiResponse>)>,
) -> ApiResult<Json<Vec<Folder>>> {
    let auth = auth?;

    let result = if auth.admin {
        sqlx::query_as!(
            Folder,
            r#"
                SELECT f.id, f.parent_id, f.name, f.owner_id, f.created_at, f.updated_at
                FROM folders f
            "#
        )
        .fetch_all(pool.inner())
        .await
    } else {
        sqlx::query_as!(
            Folder,
            r#"
        SELECT f.id, f.parent_id, f.name, f.owner_id, f.created_at, f.updated_at
        FROM folders f
        JOIN permissions p ON p.folder_id = f.id
        WHERE p.user_id = $1
          AND p.read = TRUE
        "#,
            auth.user_id
        )
        .fetch_all(pool.inner())
        .await
    }
    .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

    Ok(Json(result))
}

#[get("/folders?<parent>")]
pub async fn get_folders(
    parent: Option<Uuid>,
    pool: &State<PgPool>,
    auth: Result<AuthUser, (Status, Json<ApiResponse>)>,
) -> ApiResult<Json<Vec<Folder>>> {
    let auth = auth?;

    let result = if auth.admin {
        sqlx::query_as!(
            Folder,
            r#"
            SELECT f.id, f.parent_id, f.name, f.owner_id, f.created_at, f.updated_at
            FROM folders f
            WHERE (
                ($1::uuid IS NULL AND f.parent_id IS NULL)
                OR f.parent_id = $1
            )
        "#,
            parent
        )
        .fetch_all(pool.inner())
        .await
    } else {
        sqlx::query_as!(
            Folder,
            r#"
            SELECT f.id, f.parent_id, f.name, f.owner_id, f.created_at, f.updated_at
            FROM folders f
            JOIN permissions p ON p.folder_id = f.id
            WHERE p.user_id = $1
              AND p.read = TRUE
              AND (
                ($2::uuid IS NULL AND f.parent_id IS NULL)
                OR f.parent_id = $2
              )
        "#,
            auth.user_id,
            parent
        )
        .fetch_all(pool.inner())
        .await
    }
    .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

    Ok(Json(result))
}

#[derive(Debug, FromForm)]
pub struct UploadFile<'a> {
    pub file: TempFile<'a>,
    pub folder: Option<Uuid>,
    pub name: Option<String>,
    pub overwrite: Option<bool>,
}

#[post("/upload", data = "<data>", format = "multipart/form-data")]
pub async fn upload_file<'a>(
    mut data: Form<UploadFile<'a>>,
    pool: &State<PgPool>,
    auth: Result<AuthUser, (Status, Json<ApiResponse>)>,
) -> ApiResult {
    let auth = auth?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

    if !auth.admin {
        let edit = sqlx::query_scalar!(
            "SELECT edit FROM permissions WHERE user_id = $1 AND (($2::uuid IS NULL AND folder_id IS NULL) OR folder_id = $2)",
            auth.user_id,
            data.folder
        )
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

        if !edit.is_some_and(|v| v) {
            return Err(ApiResponse::fail(
                Status::Forbidden,
                "no permissions to upload file",
                None,
            ));
        }
    }

    let name = data
        .file
        .raw_name()
        .unwrap()
        .dangerous_unsafe_unsanitized_raw();
    let name = data.name.clone().unwrap_or(name.to_string());
    check_name(&name)?;

    let overwrite = data.overwrite.unwrap_or(false);
    let exist = sqlx::query_scalar!("SELECT EXISTS (SELECT 1 FROM files WHERE name = $1 AND (($2::uuid IS NULL AND folder_id IS NULL) OR folder_id = $2))", name, data.folder)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?
        .unwrap_or(false);
    if exist && !overwrite {
        return Err(ApiResponse::fail(
            Status::Conflict,
            "file with this name already exists",
            None,
        ));
    } else if exist && overwrite {
        let mut base = PathBuf::from(FILES_DIR.get().unwrap());
        let path = sqlx::query_scalar!("SELECT * FROM get_folder_path($1)", data.folder)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| {
                ApiResponse::fail(Status::InternalServerError, "database error", Some(&e))
            })?
            .ok_or_else(|| ApiResponse::fail(Status::NotFound, "folder not found", None))?;
        base.push(path);
        base.push(&name);
        sqlx::query!("DELETE FROM files WHERE name = $1 AND (($2::uuid IS NULL AND folder_id IS NULL) OR folder_id = $2)", name, data.folder)
            .execute(&mut *tx)
            .await
            .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;
        fs::remove_file(&base).map_err(|e| {
            ApiResponse::fail(
                Status::InternalServerError,
                "error while overwriting file",
                Some(&e),
            )
        })?;
    }
    let mut base = PathBuf::from(FILES_DIR.get().unwrap());
    if let Some(folder) = data.folder {
        let path = sqlx::query_scalar!("SELECT * FROM get_folder_path($1)", folder)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| {
                ApiResponse::fail(Status::InternalServerError, "database error", Some(&e))
            })?
            .ok_or_else(|| ApiResponse::fail(Status::NotFound, "folder not found", None))?;
        base.push(path);
    };
    if !base.exists() {
        fs::create_dir_all(&base).map_err(|e| {
            ApiResponse::fail(
                Status::InternalServerError,
                "could not create folders",
                Some(&e),
            )
        })?;
    }
    base.push(&name);
    if let Err(e) = data.file.persist_to(&base).await {
        return Err(ApiResponse::fail(
            Status::InternalServerError,
            "failed to save file",
            Some(&e),
        ));
    }
    let size: i64 = match fs::metadata(&base) {
        Ok(metadata) => metadata.len() as i64,
        Err(_) => 0,
    };
    sqlx::query!(
        "INSERT INTO files (owner_id, folder_id, name, size) VALUES ($1, $2, $3, $4)",
        auth.user_id,
        data.folder,
        name,
        size,
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

    tx.commit()
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

    Ok((
        Status::Created,
        ApiResponse::success_with(format!(r#"created file named: "{}""#, name)),
    ))
}

#[get("/files/all")]
pub async fn get_all_files(
    pool: &State<PgPool>,
    auth: Result<AuthUser, (Status, Json<ApiResponse>)>,
) -> ApiResult<Json<Vec<File>>> {
    let auth = auth?;

    let result = if auth.admin {
        sqlx::query_as!(
            File,
            r#"
            SELECT f.id, f.folder_id, f.owner_id, f.name, f.size, f.created_at, f.updated_at
            FROM files f
            "#
        )
        .fetch_all(pool.inner())
        .await
    } else {
        sqlx::query_as!(
            File,
            r#"
            SELECT f.id, f.folder_id, f.owner_id, f.name, f.size, f.created_at, f.updated_at
            FROM files f
            JOIN permissions p ON p.folder_id = f.folder_id
            WHERE p.user_id = $1
              AND p.read = TRUE
        "#,
            auth.user_id
        )
        .fetch_all(pool.inner())
        .await
    }
    .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

    Ok(Json(result))
}

#[get("/files?<parent>")]
pub async fn get_files(
    parent: Option<Uuid>,
    pool: &State<PgPool>,
    auth: Result<AuthUser, (Status, Json<ApiResponse>)>,
) -> ApiResult<Json<Vec<File>>> {
    let auth = auth?;

    let result = if auth.admin {
        sqlx::query_as!(
            File,
            r#"
            SELECT f.id, f.folder_id, f.owner_id, f.name, f.size, f.created_at, f.updated_at
            FROM files f
            WHERE (
                ($1::uuid IS NULL AND f.folder_id IS NULL)
                OR f.folder_id = $1
            )
        "#,
            parent
        )
        .fetch_all(pool.inner())
        .await
    } else {
        sqlx::query_as!(
            File,
            r#"
            SELECT f.id, f.folder_id, f.owner_id, f.name, f.size, f.created_at, f.updated_at
            FROM files f
            JOIN permissions p ON p.folder_id = f.folder_id
            WHERE p.user_id = $1
              AND p.read = TRUE
              AND (
                ($2::uuid IS NULL AND f.folder_id IS NULL)
                OR f.folder_id = $2
              )
        "#,
            auth.user_id,
            parent
        )
        .fetch_all(pool.inner())
        .await
    }
    .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

    Ok(Json(result))
}