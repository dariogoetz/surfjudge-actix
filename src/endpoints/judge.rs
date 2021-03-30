use crate::database::Pool;
use crate::models::permission::PermissionType;
use crate::models::user::User;
use crate::models::judge::JudgingRequest;
use crate::models::heat::Heat;
use crate::notifier::{Channel, Notifier};
use crate::authorization::AuthorizedUser;

use actix_web::{error, web, Result};
use serde_json::json;

pub async fn get_all(db: web::Data<Pool>) -> Result<web::Json<Vec<User>>> {
    let result = User::find_by_permission(db.get_ref(), PermissionType::Judge, false)
        .await
        .map_err(|e| {
            error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
        })?;
    Ok(web::Json(result))
}

pub async fn get_assigned_judges_for_heat(
    db: web::Data<Pool>,
    web::Path(heat_id): web::Path<u32>,
) -> Result<web::Json<Vec<User>>> {
    let result = User::find_by_judge_assignments(db.get_ref(), heat_id, false)
        .await
        .map_err(|e| {
            error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
        })?;
    Ok(web::Json(result))
}

pub async fn get_assigned_active_heats_for_judge(
    db: web::Data<Pool>,
    user: AuthorizedUser,
) -> Result<web::Json<Vec<Heat>>> {
    if !user.0.is_judge() {
        return Err(error::ErrorForbidden(format!("User '{}' not allowed to post judging requests", user.0.username)));
    }
    let result = Heat::find_active_heats_by_judge_id(db.get_ref(), user.0.id, false)
        .await
        .map_err(|e| {
            error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
        })?;
    Ok(web::Json(result))
}


pub async fn get_requests(db: web::Data<Pool>,
) -> Result<web::Json<Vec<JudgingRequest>>> {
    let result = JudgingRequest::find_all(db.get_ref(), true)
        .await
        .map_err(|e| {
            error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
        })?;
    Ok(web::Json(result))
}


pub async fn add_request(
    db: web::Data<Pool>,
    notifier: web::Data<Notifier>,
    user: AuthorizedUser,
) -> Result<web::Json<&'static str>> {

    if !user.0.is_judge() {
        return Err(error::ErrorForbidden(format!("User '{}' not allowed to post judging requests", user.0.username)));
    }

    JudgingRequest::add(db.get_ref(), user.0.id, 20)
        .await
        .map_err(|e| {
            error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
        })?;

    notifier
        .send(
            Channel::JudgingRequests,
            json!("changed"),
        )
        .unwrap();
    Ok(web::Json("Judging request received!"))
}
