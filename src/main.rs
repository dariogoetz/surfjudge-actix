use slog::info;
use sloggers::terminal::TerminalLoggerBuilder;
use sloggers::Build;

use anyhow::Result;

use actix_web::{middleware::Logger, App, HttpServer};

mod configuration;
mod database;
mod endpoints;
mod models;
mod routes;

#[actix_web::main]
async fn main() -> Result<()> {
    use configuration::CONFIG;

    let builder = TerminalLoggerBuilder::new();
    let logger = builder.build().unwrap();

    // generate a database connection pool
    info!(logger, "Connecting to database: {:?}", CONFIG.database.url);
    let pool = database::get_pool().await?;

    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .configure(routes::routes)
            .data(pool.clone())
    })
    .bind(&CONFIG.server_address)?;

    info!(logger, "Starting server at {:?}", CONFIG.server_address);
    server.run().await?;

    Ok(())
}
