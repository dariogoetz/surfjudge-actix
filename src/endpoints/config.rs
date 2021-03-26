use crate::configuration::{API, CONFIG};
use actix_web::{web, Result};

pub async fn get_ui_config() -> Result<web::Json<API>> {
    let result = CONFIG.api.clone();
    Ok(web::Json(result))
}
