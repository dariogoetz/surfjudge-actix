use crate::database::Pool;
use crate::models::tournament::Tournament;

use actix_web::{error, web, Result};

pub async fn get_all(db: web::Data<Pool>) -> Result<web::Json<Vec<Tournament>>> {
    let result = Tournament::find_all(db.get_ref()).await.map_err(|e| {
        error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
    })?;
    Ok(web::Json(result))
}

pub async fn get_by_id(
    path: web::Path<u32>,
    db: web::Data<Pool>,
) -> Result<web::Json<Option<Tournament>>> {
    let tournament_id = path.into_inner();
    let result = Tournament::find_by_id(db.get_ref(), tournament_id)
        .await
        .map_err(|e| {
            error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
        })?;
    Ok(web::Json(result))
}
