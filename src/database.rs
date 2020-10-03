use crate::configuration::CONFIG;

use actix_web::web;

use anyhow::Result;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub type PoolOptions = PgPoolOptions;
pub type Pool = PgPool;


pub async fn get_pool() -> anyhow::Result<Pool> {
    // Create a connection pool
    let pool = PoolOptions::new()
        .max_connections(CONFIG.database.max_connections)
        .connect(&CONFIG.database.url)
        .await?;

    Ok(pool)
}
