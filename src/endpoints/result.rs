use crate::{authorization::AuthorizedUser, database::Pool};
use crate::models::result::Result;
use crate::models::preliminary_result::PreliminaryResult;
use actix_web::{error, web};

pub async fn get_all(db: web::Data<Pool>) -> actix_web::Result<web::Json<Vec<Result>>> {
    let result = Result::find_all(db.get_ref(), true).await.map_err(|e| {
        error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
    })?;
    Ok(web::Json(result))
}

pub async fn get_by_heat_id(
    path: web::Path<u32>,
    db: web::Data<Pool>,
) -> actix_web::Result<web::Json<Vec<Result>>> {
    let heat_id = path.into_inner();
    let results = Result::find_by_heat_id(db.get_ref(), heat_id, true)
        .await
        .map_err(|e| {
            error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
        })?;
    Ok(web::Json(results))
}

pub async fn get_by_category_id(
    path: web::Path<u32>,
    db: web::Data<Pool>,
) -> actix_web::Result<web::Json<Vec<Result>>> {
    let category_id = path.into_inner();
    let result = Result::find_by_category_id(db.get_ref(), category_id, true)
        .await
        .map_err(|e| {
            error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
        })?;
    Ok(web::Json(result))
}

pub async fn get_preliminary_by_heat_id(
    path: web::Path<u32>,
    db: web::Data<Pool>,
    _user: AuthorizedUser,

) -> actix_web::Result<web::Json<Vec<Result>>> {
    let heat_id = path.into_inner();
    let results = PreliminaryResult::by_heat_id(db.get_ref(), heat_id)
        .await
        .map_err(|e| {
            error::ErrorInternalServerError(format!("Error computing preliminary results: {:?}", e))
        })?;
    Ok(web::Json(results))
}
