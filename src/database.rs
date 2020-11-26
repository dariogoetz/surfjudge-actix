use crate::configuration::CONFIG;

use anyhow::Result;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub type PoolOptions = PgPoolOptions;
pub type Pool = PgPool;

pub async fn get_pool() -> Result<Pool> {
    // Create a connection pool
    let pool = PoolOptions::new()
        .max_connections(CONFIG.database.maxconnections)
        .connect(&CONFIG.database.url)
        .await?;

    Ok(pool)
}
