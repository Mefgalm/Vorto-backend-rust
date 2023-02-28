use std::env;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn pg_sqlx_conect() -> PgPool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool")
}
