use slog::info;
use sloggers::Build;
use sloggers::terminal::{TerminalLoggerBuilder};

use actix_web::{middleware::Logger, App, HttpServer};

mod routes;
mod configuration;
mod endpoints;
mod database;

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    use configuration::CONFIG;

    let builder = TerminalLoggerBuilder::new();
    let logger = builder.build().unwrap();

    info!(logger, "Starting server at {:?}", CONFIG.server_address);

    let pool = database::get_pool().await;

    let mut server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .configure(routes::routes)
            .data(pool.clone())
    })
        .bind(&CONFIG.server_address)?
        .run()
        .await;

    server
}
