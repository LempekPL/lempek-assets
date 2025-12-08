use crate::auth::UserData;
use crate::models::ApiResponse;
use rocket::http::Status;
use sqlx::PgConnection;
use uuid::Uuid;
use crate::ApiResult;

pub enum PermissionKind {
    Read,
    Modify,
    Edit,
}

impl PermissionKind {
    fn as_str(&self) -> &'static str {
        match self {
            PermissionKind::Read => "read",
            PermissionKind::Modify => "modify",
            PermissionKind::Edit => "edit",
        }
    }
}

pub async fn check_permission<'a>(
    tx: &mut PgConnection,
    user: &UserData,
    folder_id: Option<Uuid>,
    permission: PermissionKind,
) -> ApiResult<()> {
    if user.admin {
        return Ok(());
    }

    let query = format!(
        "SELECT {} FROM permissions WHERE user_id = $1 AND folder_id IS NOT DISTINCT FROM $2",
        permission.as_str()
    );

    let value: Option<bool> = sqlx::query_scalar(&query)
        .bind(user.user_id)
        .bind(folder_id)
        .fetch_optional(tx)
        .await
        .map_err(|e| ApiResponse::fail(Status::InternalServerError, "database error", Some(&e)))?;

    if value.is_some_and(|v| v) {
        Ok(())
    } else {
        Err(ApiResponse::fail(
            Status::Forbidden,
            &format!("no permissions to {}", permission.as_str()),
            None,
        ))
    }
}
