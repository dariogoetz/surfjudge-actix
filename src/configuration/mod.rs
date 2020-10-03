use config;
use dotenv::dotenv;
use std::env;
use once_cell::sync::Lazy;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Settings {
    pub debug: String,
    pub server_address: String,
}


pub static CONFIG: Lazy<Settings> = Lazy::new(|| {
    dotenv().ok();

    let run_mode = env::var("RUN_MODE").unwrap_or("development".into());

    let mut settings = config::Config::default();
    settings
    // Add in `./conf/settings.toml`
        .merge(config::File::with_name("config/settings")).unwrap()
    // Add in `./conf/development.toml`
        .merge(config::File::with_name(&format!("conf/{}", run_mode)).required(false)).unwrap()
    // Add in settings from the environment (with a prefix of APP)
    // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        .merge(config::Environment::with_prefix("APP")).unwrap();

    match settings.try_into() {
        Ok(s) => s,
        Err(e) => panic!("Error parsing config files: {}", e),
    }

});
