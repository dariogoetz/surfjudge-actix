use crate::database::Pool;
use crate::models::participation::Participation;

use actix_web::{error, web, Result};

pub async fn get_all(db: web::Data<Pool>) -> Result<web::Json<Vec<Participation>>> {
    let participation = Participation::find_all(db.get_ref(), true)
        .await
        .map_err(|e| {
            error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
        })?;
    Ok(web::Json(participation))
}

pub async fn get_by_heat_id(
    path: web::Path<u32>,
    db: web::Data<Pool>,
) -> Result<web::Json<Vec<Participation>>> {
    let heat_id = path.into_inner();
    let participation = Participation::find_by_heat_id(db.get_ref(), heat_id, true)
        .await
        .map_err(|e| {
            error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
        })?;
    Ok(web::Json(participation))
}

pub async fn get_by_category_id(
    path: web::Path<u32>,
    db: web::Data<Pool>,
) -> Result<web::Json<Vec<Participation>>> {
    let category_id = path.into_inner();
    let participation = Participation::find_by_category_id(db.get_ref(), category_id, true)
        .await
        .map_err(|e| {
            error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
        })?;
    Ok(web::Json(participation))
}
