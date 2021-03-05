use anyhow::Result;
use rand::Rng;
use slog::info;
use std::sync::Arc;

use actix_cors::Cors;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{middleware::Compress, middleware::Logger, web, App, HttpServer};

mod auth;
mod configuration;
mod database;
mod endpoints;
mod logging;
mod models;
mod routes;
mod templates;

#[actix_web::main]
async fn main() -> Result<()> {
    use auth::{OsoState, Sessions};
    use configuration::CONFIG;
    use logging::LOG;

    info!(LOG, "Connecting to database: {:?}", CONFIG.database.url);
    let pool = database::get_pool().await?;

    info!(LOG, "Loading templates from {:?}", CONFIG.template_dir);
    let tmpl = templates::get_templates().await?;

    info!(LOG, "Loading auth rules form {:?}", CONFIG.auth.rules_file);
    let oso_state = web::Data::new(Arc::new(OsoState::new(&CONFIG.auth.rules_file)?));

    let private_key = rand::thread_rng().gen::<[u8; 32]>();
    let sessions = web::Data::new(Sessions::new());
    let server = HttpServer::new(move || {
        App::new()
            .app_data(sessions.clone())
            .app_data(oso_state.clone())
            .wrap(Compress::default())
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&private_key)
                    .name("surfjudge-actix")
                    .secure(false),
            ))
            .wrap(
                Cors::new() // enable cors for frontend development with webpack dev server
                    .finish(),
            )
            .wrap(Compress::default())
            // enable logger - always register actix-web Logger middleware last
            .wrap(Logger::default())
            .configure(routes::routes)
            .data(pool.clone())
            .data(tmpl.clone())
    })
    .bind(&CONFIG.server_address)?;

    info!(LOG, "Starting server at {:?}", CONFIG.server_address);
    info!(LOG, "Serving API on {:?}", CONFIG.ui_settings.api_path);
    server.run().await?;

    Ok(())
}
