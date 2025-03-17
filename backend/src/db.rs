use crate::{ADMIN_UUID, PUBLIC_DIR_UUID};
use sqlx::PgPool;
use std::env;

pub async fn connect_db() -> PgPool {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&db_url)
        .await
        .expect("Failed to connect to DB");

    if let Some(uuid) = ADMIN_UUID.get() {
        let exists_public = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM folders WHERE user_id = $1 AND name = 'public')",
            uuid,
        )
        .fetch_one(&pool)
        .await
        .unwrap()
        .unwrap_or(false);
        if !exists_public {
            sqlx::query!(
                "INSERT INTO folders (name, user_id) VALUES ('public', $1)",
                uuid
            )
            .execute(&pool)
            .await
            .unwrap();
        }

        let public_dir = sqlx::query_scalar!(
            "SELECT id FROM folders WHERE user_id = $1 AND name = 'public'",
            uuid
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        PUBLIC_DIR_UUID.set(public_dir).unwrap();
    }

    pool
}
