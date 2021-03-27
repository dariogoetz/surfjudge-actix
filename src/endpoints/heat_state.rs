use crate::database::Pool;
use crate::logging::LOG;
use crate::models::{heat::Heat, heat_state::{HeatState, HeatStateType}};
use crate::notifier::{Channel, Notifier};
use crate::{authorization::AuthorizedUser};

use actix_web::{error, web, Result};
use chrono::Utc;
use serde::Serialize;
use serde_json::json;
use slog::info;

#[derive(Debug, Serialize, Clone)]
pub struct ResultHeatState {
    pub state: HeatStateType,
    pub remaining_time_s: f64,
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
    

    let state = match &result {
        Some(heat_state) => heat_state.state.clone(),
        None => HeatStateType::Inactive,
    };
    let remaining_time_s = match &result {
        // if the heat is not active, there is no heat state row -> take duration from heat data
        None => Heat::find_by_id(db.get_ref(), heat_id, false)
            .await
            .map_err(|e| {
                error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
            })?
            .map(|h| h.duration * 60.0)
            .unwrap_or(0.0),
        Some(heat_state) => match heat_state.state {
            HeatStateType::Paused => heat_state.remaining_time_s.unwrap_or(0.0).max(0.0),
            HeatStateType::Active => {
                let now = Utc::now().naive_utc();
                let diff = (heat_state.end_datetime - now).num_milliseconds() as f64 / 1000.0;
                diff.max(0.0)
            }
            _ => 0.0,
        },
    };
    Ok(web::Json(ResultHeatState {
        state,
        remaining_time_s,
    }))
}

pub async fn start_heat(
    web::Path(heat_id): web::Path<u32>,
    db: web::Data<Pool>,
    notifier: web::Data<Notifier>,
    user: AuthorizedUser,
) -> Result<&'static str> {
    HeatState::set_heat_started(&db, heat_id)
        .await
        .map_err(|e| {
            // TODO: not found error in case it was not found
            error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
        })?;

    info!(LOG, "Start heat {} by {:?}", heat_id, user);
    notifier
        .send(
            Channel::ActiveHeats,
            json!({
                "heat_id": heat_id,
                "msg": "start_heat"
            }),
        )
        .unwrap();
    Ok("Started heat!")
}

pub async fn stop_heat(
    web::Path(heat_id): web::Path<u32>,
    db: web::Data<Pool>,
    notifier: web::Data<Notifier>,
    user: AuthorizedUser,
) -> Result<&'static str> {
    HeatState::set_heat_stopped(&db, heat_id)
        .await
        .map_err(|e| {
            error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
        })?;

    info!(LOG, "Stop heat {} by {:?}", heat_id, user);
    notifier
        .send(
            Channel::ActiveHeats,
            json!({
                "heat_id": heat_id,
                "msg": "stop_heat"
            }),
        )
        .unwrap();
    Ok("Stopped heat!")
}

pub async fn toggle_heat_pause(
    web::Path(heat_id): web::Path<u32>,
    db: web::Data<Pool>,
    notifier: web::Data<Notifier>,
    user: AuthorizedUser,
) -> Result<&'static str> {
    HeatState::toggle_heat_pause(&db, heat_id)
        .await
        .map_err(|e| {
            error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
        })?;

    info!(LOG, "Toggle pause for heat {} by {:?}", heat_id, user);
    notifier
        .send(
            Channel::ActiveHeats,
            json!({
                "heat_id": heat_id,
                "msg": "toggle_heat_pause"
            }),
        )
        .unwrap();
    Ok("Toggled heat pause!")
}

pub async fn reset_heat_time(
    web::Path(heat_id): web::Path<u32>,
    db: web::Data<Pool>,
    notifier: web::Data<Notifier>,
    user: AuthorizedUser,
) -> Result<&'static str> {
    HeatState::reset_heat_time(&db, heat_id)
        .await
        .map_err(|e| {
            error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
        })?;

    info!(LOG, "Reset heat time for heat {} by {:?}", heat_id, user);
    notifier
        .send(
            Channel::ActiveHeats,
            json!({
                "heat_id": heat_id,
                "msg": "reset_heat_time"
            }),
        )
        .unwrap();
    Ok("Reset heat time!")
}
