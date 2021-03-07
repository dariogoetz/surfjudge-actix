use crate::database::Pool;
use crate::models::heat_state::{HeatState, HeatStateType};

use actix_web::{error, web, Result};
use chrono::Utc;
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

pub async fn get_remaining_heat_time(
    web::Path(heat_id): web::Path<u32>,
    db: web::Data<Pool>,
) -> Result<web::Json<Option<f64>>> {
    let result = HeatState::find_by_heat_id(db.get_ref(), heat_id)
        .await
        .ok()
        .unwrap_or(None);

    let result = match result {
        None => None,
        Some(heat_state) => match heat_state.state {
            HeatStateType::Paused => heat_state.remaining_time_s.map(|t| t.max(0.0)),
            HeatStateType::Active => {
                let now = Utc::now().naive_utc();
                let diff = (heat_state.end_datetime - now).num_milliseconds() as f64 / 1000.0;
                Some(diff.max(0.0))
            }
            _ => None,
        },
    };

    Ok(web::Json(result))
}
