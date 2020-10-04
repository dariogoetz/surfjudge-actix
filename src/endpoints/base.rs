use crate::database::Pool;
use crate::models::heat::Heat;

use actix_web::{web, error, Result};


pub async fn test_endpoint(db: web::Data<Pool>) -> Result<web::Json<Vec<Heat>>> {
    let result = Heat::find_all(db.get_ref())
        .await
        .map_err(|e| error::ErrorInternalServerError(e))?;
    Ok(web::Json(result))
}
