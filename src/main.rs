use slog::info;
use sloggers::Build;
use sloggers::terminal::{TerminalLoggerBuilder};

use actix_web::{middleware::Logger, App, HttpServer};

mod routes;
mod configuration;
mod endpoints;

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    use configuration::CONFIG;

    let builder = TerminalLoggerBuilder::new();
    let logger = builder.build().unwrap();

    info!(logger, "Starting server at {:?}", CONFIG.server_address);

    let mut server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .configure(routes::routes)
    })
        .bind(&CONFIG.server_address)?
        .run()
        .await;

    server
}
