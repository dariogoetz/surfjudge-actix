use crate::configuration::{CONFIG, UISettings};
use actix_web::{web, Result};

pub async fn get_ui_config() -> Result<web::Json<UISettings>> {
    let result = CONFIG.uisettings.clone();
    Ok(web::Json(result))
}
