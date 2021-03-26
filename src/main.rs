use anyhow::Result;
use rand::Rng;
use slog::info;
use std::sync::Arc;

use actix::prelude::*;
use actix_cors::Cors;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{middleware::Compress, middleware::Logger, web, App, HttpServer};

mod authentication;
mod authorization;
mod configuration;
mod database;
mod endpoints;
mod logging;
mod models;
mod notifier;
mod routes;
mod templates;
mod websockets;

#[actix_web::main]
async fn main() -> Result<()> {
    use authentication::Sessions;
    use authorization::OsoState;
    use configuration::CONFIG;
    use logging::LOG;

    info!(LOG, "Connecting to database: {:?}", CONFIG.database.url);
    let pool = database::get_pool().await?;

    info!(LOG, "Loading templates from {:?}", CONFIG.template_dir);
    let tmpl = templates::get_templates().await?;

    info!(LOG, "Loading auth rules form {:?}", CONFIG.auth.rules_file);
    let oso_state = web::Data::new(Arc::new(OsoState::new(&CONFIG.auth.rules_file)?));


    
    let mut notifier = notifier::Notifier::new()?;
    let websocket_server = if let Some(address) = &CONFIG.notifications.websocket_server_address {
        info!(LOG, "Starting websocket server at {}", address);
        let websocket_server = websockets::WebSocketServer::new().start();
        let ws_notifier = notifier::WSNotifier::new(websocket_server.clone().recipient())?;
        notifier.register(Box::new(ws_notifier))?;
        Some(websocket_server)
    } else { None };
    if let Some(address) = &CONFIG.notifications.zmq_pub_address {
        info!(LOG, "Connecting ZMQ publisher to port {:?}", address);
        let zmq_notifier = notifier::ZMQNotifier::new(&format!("tcp://{}", address))?;
        notifier.register(Box::new(zmq_notifier))?;
    };


    let private_key = rand::thread_rng().gen::<[u8; 32]>();
    let sessions = web::Data::new(Sessions::new());
    let server = HttpServer::new(move || {
        let app = App::new()
            .app_data(sessions.clone())
            .app_data(oso_state.clone())
            .data(pool.clone())
            .data(tmpl.clone())
            .data(notifier.clone())
            .wrap(Compress::default())
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&private_key)
                    .name("surfjudge-actix")
                    .secure(false),
            ))
            .wrap(Cors::permissive())
            .wrap(Compress::default())
            // enable logger - always register actix-web Logger middleware last
            .wrap(Logger::default())
            .route("/config", web::get().to(endpoints::config::get_ui_config));
        
        let app = if let Some(_) = &CONFIG.api.public_path {
            app.configure(routes::public_api_routes)
        } else { app };

        let app = if let Some(_) = &CONFIG.api.private_path {
            app.configure(routes::private_api_routes)
        } else { app };

        let app = if let Some(address) = &CONFIG.notifications.websocket_server_address {
            app.data(websocket_server.clone().unwrap())
                .route(address, web::get().to(websockets::ws_route))
        } else { app };

        // page routes need to come last due to the "" scope
        app.configure(routes::static_routes)
            .configure(routes::page_routes)
    })
    .bind(&CONFIG.server_address)?;

    if let Some(address) = &CONFIG.api.public_path {
        info!(LOG, "Serving public API on {:?}", address);
    }
    if let Some(address) = &CONFIG.api.private_path {
        info!(LOG, "Serving private API on {:?}", address);
    }
    info!(LOG, "Starting server at {:?}", CONFIG.server_address);
    server.run().await?;

    Ok(())
}
