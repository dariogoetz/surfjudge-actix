use crate::database::Pool;
use crate::models::category::Category;

use actix_web::{error, web, Result};

pub async fn get_all(db: web::Data<Pool>) -> Result<web::Json<Vec<Category>>> {
    let result = Category::find_all(db.get_ref(), false).await.map_err(|e| {
        error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
    })?;
    Ok(web::Json(result))
}

pub async fn get_by_id(
    web::Path(category_id): web::Path<u32>,
    db: web::Data<Pool>,
) -> Result<web::Json<Option<Category>>> {
    let result = Category::find_by_id(db.get_ref(), category_id, false)
        .await
        .map_err(|e| {
            error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
        })?;
    Ok(web::Json(result))
}

pub async fn get_by_tournament_id(
    web::Path(tournament_id): web::Path<u32>,
    db: web::Data<Pool>,
) -> Result<web::Json<Vec<Category>>> {
    let result = Category::find_by_tournament_id(db.get_ref(), tournament_id, false)
        .await
        .map_err(|e| {
            error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
        })?;
    Ok(web::Json(result))
}
