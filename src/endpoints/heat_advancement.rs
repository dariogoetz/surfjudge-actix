use crate::database::Pool;
use crate::models::heat_advancement::HeatAdvancement;

use actix_web::{error, web, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct HeatAdvancementQuery {
    pub to_heat_id: Option<i32>,
    pub from_heat_id: Option<i32>,
}

pub async fn get_all(
    query_params: web::Query<HeatAdvancementQuery>,
    db: web::Data<Pool>,
) -> Result<web::Json<Vec<HeatAdvancement>>> {
    let result = match query_params.into_inner() {
        HeatAdvancementQuery {
            to_heat_id: None,
            from_heat_id: None,
        } => HeatAdvancement::find_all(db.get_ref(), true).await,
        HeatAdvancementQuery {
            to_heat_id: Some(x),
            from_heat_id: None,
        } => HeatAdvancement::find_by_to_heat_id(db.get_ref(), x as u32, true).await,
        HeatAdvancementQuery {
            to_heat_id: None,
            from_heat_id: Some(x),
        } => HeatAdvancement::find_by_from_heat_id(db.get_ref(), x as u32, true).await,
        _ => {
            return Err(error::ErrorInternalServerError(format!(
                "Query for both to_heat_id and from_heat_id unsupported"
            )))
        }
    }
    .map_err(|e| {
        error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
    })?;
    Ok(web::Json(result))
}
pub async fn get_by_category_id(
    web::Path(category_id): web::Path<u32>,
    db: web::Data<Pool>,
) -> Result<web::Json<Vec<HeatAdvancement>>> {
    let result = HeatAdvancement::find_by_category_id(db.get_ref(), category_id, true)
        .await
        .map_err(|e| {
            error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
        })?;
    Ok(web::Json(result))
}

// TODO: advancing surfers
// TODO: query parameters Query<Params> with struct Params {to_heat_id: Option<i32>, from_heat_id: Option<i32>, ...}
