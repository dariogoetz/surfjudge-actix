use slog::info;
use sloggers::terminal::TerminalLoggerBuilder;
use sloggers::Build;

use anyhow::Result;

use actix_web::{middleware::Logger, middleware::Compress, App, HttpServer};
use actix_cors::Cors;

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
            .wrap(Cors::new()  // enable cors for frontend development with webpack dev server
                  .finish()
            )
            .wrap(Compress::default())
            .configure(routes::routes)
            .data(pool.clone())
            .data(tmpl.clone())
    })
    .bind(&CONFIG.server_address)?;

    info!(logger, "Starting server at {:?}", CONFIG.server_address);
    info!(logger, "Serving api on {:?}", CONFIG.ui_settings.api_path);
    server.run().await?;

    Ok(())
}
