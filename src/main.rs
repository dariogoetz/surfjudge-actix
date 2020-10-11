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
mod templates;

#[actix_web::main]
async fn main() -> Result<()> {
    use configuration::CONFIG;

    let builder = TerminalLoggerBuilder::new();
    let logger = builder.build().unwrap();

    info!(logger, "Connecting to database: {:?}", CONFIG.database.url);
    let pool = database::get_pool().await?;

    info!(logger, "Loading templates from {:?}", CONFIG.template_dir);
    let tmpl = templates::get_templates().await?;

    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .configure(routes::routes)
            .data(pool.clone())
            .data(tmpl.clone())
    })
    .bind(&CONFIG.server_address)?;

    info!(logger, "Starting server at {:?}", CONFIG.server_address);
    server.run().await?;

    Ok(())
}
