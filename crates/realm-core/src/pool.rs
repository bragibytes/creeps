use std::sync::{Arc, OnceLock};

use anyhow::Context;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

static POOL: OnceLock<Arc<PgPool>> = OnceLock::new();

pub fn pool() -> Arc<PgPool> {
    POOL.get().expect("database pool not initialized").clone()
}

pub async fn connect_and_migrate(database_url: &str) -> anyhow::Result<Arc<PgPool>> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .context("connect to postgres")?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("run database migrations")?;

    let pool = Arc::new(pool);
    POOL.set(pool.clone()).ok();
    Ok(pool)
}

pub fn block_on<F: std::future::Future>(future: F) -> F::Output {
    tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(future))
}