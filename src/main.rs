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

    info!(
        LOG,
        "Connecting ZMQ publisher to port {:?}", CONFIG.zmq_address
    );
    let websocket_server = websockets::WebSocketServer::new().start();
    let ws_notifier = notifier::WSNotifier::new(websocket_server.clone().recipient())?;
    let zmq_notifier = notifier::ZMQNotifier::new(&format!("tcp://{}", CONFIG.zmq_address))?;
    let notifier = Arc::new(notifier::Notifier::new()?
        .zmq(zmq_notifier)?
        .ws(ws_notifier)?);


    let private_key = rand::thread_rng().gen::<[u8; 32]>();
    let sessions = web::Data::new(Sessions::new());
    let server = HttpServer::new(move || {
        let app = App::new()
            .app_data(sessions.clone())
            .app_data(oso_state.clone())
            .data(pool.clone())
            .data(tmpl.clone())
            .data(notifier.clone())
            .data(websocket_server.clone())
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
            .configure(routes::public_api_routes);

        let app = if CONFIG.serve_private_api {
            app.configure(routes::private_api_routes)
        } else {
            app
        };

        let app = app.route("/ws", web::get().to(websockets::ws_route));

        // page routes need to come last due to the "" scope
        app.configure(routes::static_routes)
            .configure(routes::page_routes)
    })
    .bind(&CONFIG.server_address)?;

    info!(LOG, "Starting server at {:?}", CONFIG.server_address);
    info!(
        LOG,
        "Serving public API on {:?}", CONFIG.ui_settings.public_api_path
    );
    if CONFIG.serve_private_api {
        info!(
            LOG,
            "Serving private API on {:?}", CONFIG.ui_settings.private_api_path
        );
    }
    server.run().await?;

    Ok(())
}
