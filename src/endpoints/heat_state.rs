use crate::database::Pool;
use crate::models::heat_state::{HeatState, HeatStateType};

use actix_web::{error, web, Result};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ResultHeatState {
    pub state: HeatStateType,
}

pub async fn get_by_heat_id(
    web::Path(heat_id): web::Path<u32>,
    db: web::Data<Pool>,
) -> Result<web::Json<ResultHeatState>> {
    let result = HeatState::find_by_heat_id(db.get_ref(), heat_id)
        .await
        .map_err(|e| {
            error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
        })?;

    let result = match result {
        Some(heat_state) => heat_state.state,
        None => HeatStateType::Inactive,
    };
    Ok(web::Json(ResultHeatState { state: result }))
}
