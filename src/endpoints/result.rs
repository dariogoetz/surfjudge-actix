use crate::database::Pool;
use crate::models::result::Result;

use actix_web::{error, web};

pub async fn get_all(db: web::Data<Pool>) -> actix_web::Result<web::Json<Vec<Result>>> {
    let result = Result::find_all(db.get_ref(), true).await.map_err(|e| {
        error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
    })?;
    Ok(web::Json(result))
}

pub async fn get_by_heat_id(web::Path(heat_id): web::Path<u32>, db: web::Data<Pool>) -> actix_web::Result<web::Json<Vec<Result>>> {
    let result = Result::find_by_heat_id(db.get_ref(), heat_id, true).await.map_err(|e| {
        error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
    })?;
    Ok(web::Json(result))
}

pub async fn get_by_category_id(web::Path(category_id): web::Path<u32>, db: web::Data<Pool>) -> actix_web::Result<web::Json<Vec<Result>>> {
    let result = Result::find_by_category_id(db.get_ref(), category_id, true).await.map_err(|e| {
        error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
    })?;
    Ok(web::Json(result))
}
