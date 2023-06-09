use config::{Config, Environment, File};
use dotenv::dotenv;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::env;

// not sure exactly why, but I think because we use CONFIG as a global object, all the settings fields need to be pub
#[derive(Deserialize, Debug)]
pub struct Database {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct API {
    pub websocket_url: String,
    pub public_path: Option<String>,
    pub auth_path: Option<String>,
    pub judging_path: Option<String>,
    pub admin_path: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Auth {
    pub rules_file: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Notifications {
    pub zmq_sender_address: Option<String>,
    pub zmq_receiver_port: Option<String>,
    pub websocket_server_address: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Settings {
    pub server_address: String,
    pub template_dir: String,
    pub cors_origins: Option<String>,
    pub database: Database,
    pub api: API,
    pub auth: Auth,
    pub notifications: Notifications,
}

pub static CONFIG: Lazy<Settings> = Lazy::new(|| {
    dotenv().ok();

    let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

    let mut settings = Config::default();
    settings
        // Add in `./conf/settings.toml`
        .merge(File::with_name("config/settings"))
        .unwrap()
        // Add in `./config/development.toml` or `./config/production.toml`, depending on RUN_MODE
        .merge(File::with_name(&format!("config/{}", run_mode)).required(false))
        .unwrap()
        // Add in settings from the environment
        // Eg.. `DEBUG=1 ./target/app` would set the `debug` key
        .merge(Environment::new().separator("__"))
        .unwrap();

    match settings.try_into() {
        Ok(s) => s,
        Err(e) => panic!("Error parsing config files: {}", e),
    }
});
