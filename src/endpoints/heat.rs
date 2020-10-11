use crate::database::Pool;
use crate::models::heat::Heat;

use actix_web::{error, web, Result};

pub async fn get_all(db: web::Data<Pool>) -> Result<web::Json<Vec<Heat>>> {
    let result = Heat::find_all(db.get_ref()).await.map_err(|e| {
        error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
    })?;
    Ok(web::Json(result))
}

pub async fn get_by_id(web::Path(heat_id): web::Path<u32>, db: web::Data<Pool>) -> Result<web::Json<Option<Heat>>> {
    let result = Heat::find_by_id(db.get_ref(), heat_id).await.map_err(|e| {
        error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
    })?;
    Ok(web::Json(result))
}


pub async fn get_active_heats_by_tournament_id(web::Path(tournament_id): web::Path<u32>, db: web::Data<Pool>) -> Result<web::Json<Vec<Heat>>> {
    let result = Heat::find_active_heats_by_tournament_id(db.get_ref(), tournament_id).await.map_err(|e| {
        error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
    })?;
    Ok(web::Json(result))
}
