use crate::authorization::AuthorizedUser;
use crate::database::Pool;
use crate::models::heat::Heat;
use crate::models::judge::{JudgingRequest, JudgingAssignment};
use crate::models::permission::PermissionType;
use crate::models::user::User;
use crate::notifier::{Channel, Notifier};

use actix_web::{error, web, Result};
use serde_json::json;

pub async fn get_all(db: web::Data<Pool>, _: AuthorizedUser) -> Result<web::Json<Vec<User>>> {
    let result = User::find_by_permission(db.get_ref(), PermissionType::Judge, false)
        .await
        .map_err(|e| {
            error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
        })?;
    Ok(web::Json(result))
}

pub async fn get_assigned_judges_for_heat(
    db: web::Data<Pool>,
    path: web::Path<u32>,
    _: AuthorizedUser,
) -> Result<web::Json<Vec<User>>> {
    let heat_id = path.into_inner();
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
        return Err(error::ErrorForbidden(format!(
            "User '{}' not allowed to post judging requests",
            user.0.username
        )));
    }
    let result = Heat::find_active_heats_by_judge_id(db.get_ref(), user.0.id, false)
        .await
        .map_err(|e| {
            error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
        })?;
    Ok(web::Json(result))
}


pub async fn add_assignment(
    path: web::Path<(u32, u32)>,
    db: web::Data<Pool>,
    notifier: web::Data<Notifier>,
    _: AuthorizedUser,
) -> Result<web::Json<&'static str>> {
    let (heat_id, judge_id) = path.into_inner();
    JudgingAssignment::add(db.get_ref(), heat_id, judge_id)
        .await
        .map_err(|e|{
            error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
        })?;
    notifier
        .send(Channel::JudgingAssignments, json!("changed"))
        .unwrap();
    Ok(web::Json("Judging assignment added!"))
}

pub async fn delete_assignment(
    path: web::Path<(u32, u32)>,
    db: web::Data<Pool>,
    notifier: web::Data<Notifier>,
    _: AuthorizedUser,
) -> Result<web::Json<&'static str>> {
    let (heat_id, judge_id) = path.into_inner();
    JudgingAssignment::delete(db.get_ref(), heat_id, judge_id)
        .await
        .map_err(|e|{
            error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
        })?;
    notifier
        .send(Channel::JudgingAssignments, json!("changed"))
        .unwrap();
    Ok(web::Json("Judging assignment deleted!"))
}

pub async fn get_requests(
    db: web::Data<Pool>,
    _: AuthorizedUser,
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
        return Err(error::ErrorForbidden(format!(
            "User '{}' not allowed to post judging requests",
            user.0.username
        )));
    }

    JudgingRequest::add(db.get_ref(), user.0.id, 20)
        .await
        .map_err(|e| {
            error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
        })?;

    notifier
        .send(Channel::JudgingRequests, json!("changed"))
        .unwrap();
    Ok(web::Json("Judging request received!"))
}
