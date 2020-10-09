use crate::database::Pool;
use crate::models::heat_advancement::HeatAdvancement;

use actix_web::{error, web, Result};

pub async fn get_all(db: web::Data<Pool>) -> Result<web::Json<Vec<HeatAdvancement>>> {
    let result = HeatAdvancement::find_all(db.get_ref()).await.map_err(|e| {
        error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
    })?;
    Ok(web::Json(result))
}
pub async fn get_by_category_id(web::Path(category_id): web::Path<u32>, db: web::Data<Pool>) -> Result<web::Json<Vec<HeatAdvancement>>> {
    let result = HeatAdvancement::find_by_category_id(db.get_ref(), category_id).await.map_err(|e| {
        error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
    })?;
    Ok(web::Json(result))
}

// TODO: advancing surfers
// TODO: query parameters Query<Params> with struct Params {to_heat_id: Option<i32>, from_heat_id: Option<i32>, ...} 
