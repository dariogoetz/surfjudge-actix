use crate::database::Pool;
use crate::models::heat::Heat;

use serde::Deserialize;
use actix_web::{error, web, Result};

#[derive(Debug, Deserialize)]
pub struct HeatQuery {
    category_id: Option<i32>,
}

pub async fn get_all(query_params: web::Query<HeatQuery>, db: web::Data<Pool>) -> Result<web::Json<Vec<Heat>>> {
    let result = match query_params.into_inner() {
        HeatQuery {category_id: Some(x)} => Heat::find_by_category_id(db.get_ref(), x as u32, true).await,
        _ => Heat::find_all(db.get_ref(), true).await
    }
    .map_err(|e| {
        error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
    })?;
    Ok(web::Json(result))
}

pub async fn get_by_id(web::Path(heat_id): web::Path<u32>, db: web::Data<Pool>) -> Result<web::Json<Option<Heat>>> {
    let result = Heat::find_by_id(db.get_ref(), heat_id, true).await.map_err(|e| {
        error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
    })?;
    Ok(web::Json(result))
}


pub async fn get_active_heats_by_tournament_id(web::Path(tournament_id): web::Path<u32>, db: web::Data<Pool>) -> Result<web::Json<Vec<Heat>>> {
    let result = Heat::find_active_heats_by_tournament_id(db.get_ref(), tournament_id, true).await.map_err(|e| {
        error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
    })?;
    Ok(web::Json(result))
}
