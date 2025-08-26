use crate::FILES_DIR;
use crate::auth::AuthUser;
use crate::models::{ApiResponse, File, Folder};
use crate::perms::{ApiResult, PermissionKind, check_permission};
use rocket::form::Form;
use rocket::{State, fs::TempFile, http::Status, post, serde::json::Json};
use serde::{Deserialize, Serialize};
use sqlx::{Acquire, PgConnection, PgPool};
use std::path::Path;
use std::{fs, path::PathBuf};
use uuid::Uuid;

async fn get_folder_path(tx: &mut PgConnection, id: Option<Uuid>) -> ApiResult<String> {
    Ok(sqlx::query_scalar!("SELECT * FROM get_folder_path($1)", id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?
        .unwrap_or_default())
}

fn remove_last_path(s: &str) -> String {
    Path::new(s)
        .parent()
        .unwrap_or(Path::new(""))
        .to_string_lossy()
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
        let invalid_string = invalid_chars
            .iter()
            .map(|c| format!("'{}'", c))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(ApiResponse::fail(
            Status::Forbidden,
            format!(
                "You used illegal character in name.\nList of illegal chars: {}",
                invalid_string
            ),
            None,
        ));
    }
    let reserved_names = [
        "api", "login", "profile", "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4",
        "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6",
        "LPT7", "LPT8", "LPT9",
    ];
    if reserved_names
        .iter()
        .any(|&res| name.eq_ignore_ascii_case(res))
    {
        let reserved_string = reserved_names.join("', '");
        return Err(ApiResponse::fail(
            Status::Forbidden,
            format!(
                "You used illegal name.\nList of illegal names: '{}'",
                reserved_string
            ),
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

fn get_ord(order: Option<String>) -> &'static str {
    match order.as_deref() {
        Some("name_asc") => "f.name ASC",
        Some("name_desc") => "f.name DESC",
        Some("created_asc") => "f.created_at ASC",
        Some("created_desc") => "f.created_at DESC",
        Some("updated_asc") => "f.updated_at ASC",
        Some("updated_desc") => "f.updated_at DESC",
        None | _ => "f.name ASC",
    }
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

    check_permission(&mut tx, &auth, data.parent, PermissionKind::Edit).await?;
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

    let mut base = PathBuf::from(FILES_DIR.get().unwrap());
    base.push(&get_folder_path(&mut tx, Some(folder_id)).await?);

    fs::create_dir(&base).map_err(|e| {
        dbg!(&e);
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

    check_permission(&mut tx, &auth, Some(data.id), PermissionKind::Modify).await?;

    let mut base = PathBuf::from(FILES_DIR.get().unwrap());
    base.push(&get_folder_path(&mut tx, Some(data.id)).await?);

    let res = sqlx::query!("DELETE FROM folders WHERE id = $1", data.id,)
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;
    if res.rows_affected() == 0 {
        return Err(ApiResponse::fail(
            Status::NotFound,
            "folder not found",
            None,
        ));
    }

    fs::remove_dir_all(base).map_err(|e| {
        ApiResponse::fail(
            Status::InternalServerError,
            "error while deleting folder",
            Some(&e),
        )
    })?;

    tx.commit()
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

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

    check_permission(&mut tx, &auth, Some(data.id), PermissionKind::Modify).await?;
    check_name(&data.name)?;
    let old_path = &get_folder_path(&mut tx, Some(data.id)).await?;

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
        JOIN permissions p ON p.folder_id IS NOT DISTINCT FROM f.id
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

#[get("/folders?<parent>&<order>")]
pub async fn get_folders(
    parent: Option<Uuid>,
    order: Option<String>,
    pool: &State<PgPool>,
    auth: Result<AuthUser, (Status, Json<ApiResponse>)>,
) -> ApiResult<Json<Vec<Folder>>> {
    let auth = auth?;
    let order_sql = get_ord(order);

    let result = if auth.admin {
        sqlx::query_as::<_, Folder>(&format!(
            r#"
            SELECT f.id, f.parent_id, f.name, f.owner_id, f.created_at, f.updated_at
            FROM folders f
            WHERE f.parent_id IS NOT DISTINCT FROM $1
            ORDER BY {}
        "#,
            order_sql
        ))
        .bind(parent)
        .fetch_all(pool.inner())
        .await
    } else {
        sqlx::query_as::<_, Folder>(&format!(
            r#"
            SELECT f.id, f.parent_id, f.name, f.owner_id, f.created_at, f.updated_at
            FROM folders f
            JOIN permissions p ON p.folder_id IS NOT DISTINCT FROM f.id
            WHERE p.user_id = $1
              AND p.read = TRUE
              AND f.parent_id IS NOT DISTINCT FROM $2
            ORDER BY {}
        "#,
            order_sql
        ))
        .bind(auth.user_id)
        .bind(parent)
        .fetch_all(pool.inner())
        .await
    }
    .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

    Ok(Json(result))
}

#[derive(Serialize, sqlx::FromRow)]
pub struct UuidPath {
    id: Option<Uuid>,
    name: Option<String>,
}
#[get("/folder/path?<id>")]
pub async fn get_folders_path(
    id: Option<Uuid>,
    pool: &State<PgPool>,
    auth: Result<AuthUser, (Status, Json<ApiResponse>)>,
) -> ApiResult<Json<Vec<UuidPath>>> {
    let auth = auth?;

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

    if !auth.admin {
        check_permission(&mut tx, &auth, id, PermissionKind::Read).await?;
    }
    let result = sqlx::query_as!(UuidPath, "SELECT * FROM get_folder_uuid_path($1)", id)
        .fetch_all(&mut *tx)
        .await
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

    check_permission(&mut tx, &auth, data.folder, PermissionKind::Edit).await?;

    let name = data
        .file
        .raw_name()
        .unwrap()
        .dangerous_unsafe_unsanitized_raw();
    let name = data.name.clone().unwrap_or(name.to_string());
    check_name(&name)?;

    let overwrite = data.overwrite.unwrap_or(false);
    let exist = sqlx::query_scalar!(
        "SELECT EXISTS (SELECT 1 FROM files WHERE name = $1 AND folder_id IS NOT DISTINCT FROM $2)",
        name,
        data.folder
    )
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
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o644);
        std::fs::set_permissions(&base, perms).map_err(|e| {
            ApiResponse::fail(
                Status::InternalServerError,
                "failed to change file permissions",
                Some(&e),
            )
        })?;
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
            JOIN permissions p ON p.folder_id IS NOT DISTINCT FROM f.folder_id
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

#[get("/files?<parent>&<order>")]
pub async fn get_files(
    parent: Option<Uuid>,
    order: Option<String>,
    pool: &State<PgPool>,
    auth: Result<AuthUser, (Status, Json<ApiResponse>)>,
) -> ApiResult<Json<Vec<File>>> {
    let auth = auth?;
    let order_sql = get_ord(order);

    let result = if auth.admin {
        sqlx::query_as::<_, File>(&format!(
            r#"
            SELECT f.id, f.folder_id, f.owner_id, f.name, f.size, f.created_at, f.updated_at
            FROM files f
            WHERE f.folder_id IS NOT DISTINCT FROM $1
            ORDER BY {}
        "#,
            order_sql
        ))
        .bind(parent)
        .fetch_all(pool.inner())
        .await
    } else {
        sqlx::query_as::<_, File>(&format!(
            r#"
            SELECT f.id, f.folder_id, f.owner_id, f.name, f.size, f.created_at, f.updated_at
            FROM files f
            JOIN permissions p ON p.folder_id IS NOT DISTINCT FROM f.folder_id
            WHERE p.user_id = $1
              AND p.read = TRUE
              AND f.folder_id IS NOT DISTINCT FROM $2
            ORDER BY {}
        "#,
            order_sql
        ))
        .bind(auth.user_id)
        .bind(parent)
        .fetch_all(pool.inner())
        .await
    }
    .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

    Ok(Json(result))
}

#[derive(Serialize, Deserialize)]
pub struct RemoveFile {
    pub id: Uuid,
}

#[delete("/file", format = "json", data = "<data>")]
pub async fn delete_file(
    data: Json<RemoveFile>,
    pool: &State<PgPool>,
    auth: Result<AuthUser, (Status, Json<ApiResponse>)>,
) -> ApiResult {
    let auth = auth?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

    let file = sqlx::query!("SELECT folder_id, name FROM files WHERE id = $1", data.id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

    check_permission(&mut tx, &auth, file.folder_id, PermissionKind::Edit).await?;

    let mut base = PathBuf::from(FILES_DIR.get().unwrap());
    base.push(&get_folder_path(&mut tx, file.folder_id).await?);
    base.push(&file.name);

    let res = sqlx::query!("DELETE FROM files WHERE id = $1", data.id)
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;
    if res.rows_affected() == 0 {
        return Err(ApiResponse::fail(Status::NotFound, "file not found", None));
    }

    tx.commit()
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

    fs::remove_file(base).map_err(|e| {
        ApiResponse::fail(
            Status::InternalServerError,
            "error while deleting file",
            Some(&e),
        )
    })?;

    Ok((Status::NoContent, ApiResponse::success_with("deleted file")))
}

#[derive(Serialize, Deserialize)]
pub struct EditFile {
    pub id: Uuid,
    pub name: String,
}

#[patch("/file", format = "json", data = "<data>")]
pub async fn edit_file(
    data: Json<EditFile>,
    pool: &State<PgPool>,
    auth: Result<AuthUser, (Status, Json<ApiResponse>)>,
) -> ApiResult {
    let auth = auth?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

    let file = sqlx::query!("SELECT folder_id, name FROM files WHERE id = $1", data.id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

    check_permission(&mut tx, &auth, file.folder_id, PermissionKind::Edit).await?;
    check_name(&data.name)?;

    let result = sqlx::query!(
        "UPDATE files SET name = $1 WHERE id = $2",
        data.name,
        data.id,
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(db_err) = &e
            && db_err.is_unique_violation()
        {
            ApiResponse::fail(Status::Conflict, "file with this name already exists", None)
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

    let mut old_path = PathBuf::from(FILES_DIR.get().unwrap());
    old_path.push(&get_folder_path(&mut tx, file.folder_id).await?);
    old_path.push(&file.name);

    let mut new_path = PathBuf::from(FILES_DIR.get().unwrap());
    new_path.push(&get_folder_path(&mut tx, file.folder_id).await?);
    new_path.push(&data.name);

    fs::rename(&old_path, &new_path).map_err(|e| {
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
