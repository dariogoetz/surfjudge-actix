use crate::database::Pool;
use crate::models::permission::PermissionType;
use crate::models::user::User;

use actix_web::{error, web, Result};
use serde::Deserialize;

pub async fn get_all(db: web::Data<Pool>) -> Result<web::Json<Vec<User>>> {
    let result = User::find_by_permission(db.get_ref(), PermissionType::Judge, false)
        .await
        .map_err(|e| {
            error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
        })?;
    Ok(web::Json(result))
}
