use crate::database::Pool;
use crate::models::lycra_color::LycraColor;

use actix_web::{error, web, Result};

pub async fn get_all(db: web::Data<Pool>) -> Result<web::Json<Vec<LycraColor>>> {
    let result = LycraColor::find_all(db.get_ref()).await.map_err(|e| {
        error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
    })?;
    Ok(web::Json(result))
}

pub async fn get_by_id(
    web::Path(lycra_color_id): web::Path<u32>,
    db: web::Data<Pool>,
) -> Result<web::Json<Option<LycraColor>>> {
    let result = LycraColor::find_by_id(db.get_ref(), lycra_color_id)
        .await
        .map_err(|e| {
            error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
        })?;
    Ok(web::Json(result))
}
