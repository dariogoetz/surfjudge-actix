use crate::database::Pool;
use crate::models::permission::PermissionType;
use crate::models::user::User;

use actix_web::{error, web, Result};

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
