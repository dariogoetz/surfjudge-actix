use crate::database::Pool;
use crate::models::permission::PermissionType;
use crate::models::user::User;
use crate::models::judge::JudgingRequest;
use crate::notifier::{Channel, Notifier};

use actix_web::{error, web, Result};
use serde::Deserialize;
use serde_json::json;

pub async fn get_all(db: web::Data<Pool>) -> Result<web::Json<Vec<User>>> {
    let result = User::find_by_permission(db.get_ref(), PermissionType::Judge, false)
        .await
        .map_err(|e| {
            error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
        })?;
    Ok(web::Json(result))
}

pub async fn get_assigned(
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


pub async fn get_requests(db: web::Data<Pool>,
) -> Result<web::Json<Vec<JudgingRequest>>> {
    let result = JudgingRequest::find_all(db.get_ref(), true)
        .await
        .map_err(|e| {
            error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
        })?;
    Ok(web::Json(result))
}


#[derive(Debug, Deserialize)]
pub struct JudgingRequestBody {
    pub judge_id: u32,
    pub expire_s: Option<u32>,
}
pub async fn add_request(
    req: web::Json<JudgingRequestBody>,
    db: web::Data<Pool>,
    notifier: web::Data<Notifier>,
) -> Result<web::Json<&'static str>> {
    let expire_s = req.expire_s.unwrap_or(20);
    JudgingRequest::add(db.get_ref(), req.judge_id, expire_s)
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
