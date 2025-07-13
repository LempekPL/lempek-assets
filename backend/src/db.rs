use sqlx::PgPool;
use std::env;

pub async fn connect_db() -> PgPool {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&db_url)
        .await
        .expect("failed to connect to DB");

    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    pool
}
