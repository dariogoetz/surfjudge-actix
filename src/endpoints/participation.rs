use crate::database::Pool;
use crate::models::participation::Participation;

use actix_web::{error, web};

pub async fn get_all(db: web::Data<Pool>) -> actix_web::Result<web::Json<Vec<Participation>>> {
    let participation = Participation::find_all(db.get_ref()).await.map_err(|e| {
        error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
    })?;
    Ok(web::Json(participation))
}

pub async fn get_by_heat_id(web::Path(heat_id): web::Path<u32>, db: web::Data<Pool>) -> actix_web::Result<web::Json<Vec<Participation>>> {
    let participation = Participation::find_by_heat_id(db.get_ref(), heat_id).await.map_err(|e| {
        error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
    })?;
    Ok(web::Json(participation))
}

