use config::{Config, File, Environment};
use dotenv::dotenv;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::env;

// not sure exactly, but I thing because we use CONFIG as a global object, all the settings fields need to be pub
#[derive(Deserialize, Debug)]
pub struct Database {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Deserialize, Debug)]
pub struct Settings {
    pub debug: String,
    pub server_address: String,
    pub database: Database,
    pub template_dir: String,
    pub websocket_url: String,
}

pub static CONFIG: Lazy<Settings> = Lazy::new(|| {
    dotenv().ok();

    let run_mode = env::var("RUN_MODE")
        .unwrap_or_else(|_| {"development".into()});

    let mut settings = Config::default();
    settings
        // Add in `./conf/settings.toml`
        .merge(File::with_name("config/settings"))
        .unwrap()
        // Add in `./conf/development.toml` or `./conf/production.toml`, depending on RUN_MODE
        .merge(File::with_name(&format!("conf/{}", run_mode)).required(false))
        .unwrap()
        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        .merge(Environment::with_prefix("APP"))
        .unwrap();

    match settings.try_into() {
        Ok(s) => s,
        Err(e) => panic!("Error parsing config files: {}", e),
    }
});
