use crate::database::Pool;
use crate::models::surfer::Surfer;

use actix_web::{error, web, Result};

pub async fn get_all(db: web::Data<Pool>) -> Result<web::Json<Vec<Surfer>>> {
    let result = Surfer::find_all(db.get_ref()).await.map_err(|e| {
        error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
    })?;
    Ok(web::Json(result))
}

pub async fn get_by_id(web::Path(surfer_id): web::Path<u32>, db: web::Data<Pool>) -> Result<web::Json<Option<Surfer>>> {
    let result = Surfer::find_by_id(db.get_ref(), surfer_id).await.map_err(|e| {
        error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
    })?;
    Ok(web::Json(result))
}
