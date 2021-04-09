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
mod websockets;

#[actix_web::main]
async fn main() -> Result<()> {
    use authentication::Sessions;
    use authorization::OsoState;
    use configuration::CONFIG;
    use logging::LOG;

    info!(LOG, "Connecting to database: {:?}", CONFIG.database.url);
    let pool = database::get_pool().await?;

    info!(LOG, "Loading auth rules form {:?}", CONFIG.auth.rules_file);
    let oso_state = web::Data::new(Arc::new(OsoState::new(&CONFIG.auth.rules_file)?));

    let mut notifier = notifier::Notifier::new()?;
    let websocket_server = if let Some(address) = &CONFIG.notifications.websocket_server_address {
        info!(LOG, "Starting websocket server at {}", address);

        let websocket_server = websockets::WebSocketServer::new().start();

        let ws_notifier = notifier::WSNotifier::new(websocket_server.clone().recipient())?;
        notifier.register(Box::new(ws_notifier))?;

        Some(websocket_server)
    } else {
        None
    };

    #[cfg(feature = "zmq-notifier")]
    if let Some(address) = &CONFIG.notifications.zmq_sender_address {
        use notifier::zmq_notifier::ZMQNotifier;

        info!(LOG, "Connecting ZMQ publisher to {}", address);
        let zmq_notifier = ZMQNotifier::new(&format!("tcp://{}", address))?;
        notifier.register(Box::new(zmq_notifier))?;
    };

    #[cfg(feature = "zmq-receiver")]
    if let Some(port) = &CONFIG.notifications.zmq_receiver_port {
        use notifier::zmq_receiver::ZMQReceiver;

        info!(LOG, "Listening for ZMQ messages on port {}", port);
        let zmq_receiver = ZMQReceiver::new(&format!("tcp://0.0.0.0:{}", port), &notifier)?;
        zmq_receiver.start()?;
    };

    #[cfg(feature = "zmq-notifier-async")]
    if let Some(address) = &CONFIG.notifications.zmq_sender_address {
        use notifier::zmq_notifier::ZMQNotifier;

        info!(LOG, "Connecting ZMQ publisher to {}", address);
        let zmq_notifier = ZMQNotifier::new(&format!("tcp://{}", address)).await?;
        notifier.register(Box::new(zmq_notifier))?;
    };

    #[cfg(feature = "zmq-receiver-async")]
    if let Some(port) = &CONFIG.notifications.zmq_receiver_port {
        use notifier::zmq_receiver::ZMQReceiver;

        info!(LOG, "Listening for ZMQ messages on port {}", port);
        let zmq_receiver = ZMQReceiver::new(&format!("tcp://0.0.0.0:{}", port), &notifier)?;
        zmq_receiver.start().await?;
    };

    let private_key = rand::thread_rng().gen::<[u8; 32]>();
    let sessions = web::Data::new(Sessions::new());
    info!(LOG, "Starting server at {:?}", CONFIG.server_address);
    let server = HttpServer::new(move || {
        let app = App::new()
            .app_data(sessions.clone())
            .app_data(oso_state.clone())
            .data(pool.clone())
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

        let app = if let Some(address) = &CONFIG.notifications.websocket_server_address {
            app
                .data(websocket_server.clone().unwrap())
                .route(&format!("{}/messages", address), web::post().to(websockets::post))
                .route(address, web::get().to(websockets::ws_route))
        } else {
            app
        };
        app.configure(routes::configure_apis)
    })
    .bind(&CONFIG.server_address)?;

    if let Some(address) = &CONFIG.api.public_path {
        info!(LOG, "Serving public API at {:?}", address);
    }
    if let Some(address) = &CONFIG.api.admin_path {
        info!(LOG, "Serving admin API at {:?}", address);
    }
    if let Some(address) = &CONFIG.api.judging_path {
        info!(LOG, "Serving judging API at {:?}", address);
    }
    server.run().await?;

    Ok(())
}
