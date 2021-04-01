use crate::database::Pool;
use crate::models::score::Score;
use crate::authorization::AuthorizedUser;

use actix_web::{error, web, Result};

pub async fn get_by_heat_id_and_judge_id(
    web::Path((heat_id, judge_id)): web::Path<(u32, u32)>,
    db: web::Data<Pool>,
    _user: AuthorizedUser,
) -> Result<web::Json<Vec<Score>>> {
    let result = Score::find_by_heat_and_judge(db.get_ref(), heat_id, judge_id)
        .await.map_err(|e| {
            error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
    })?;

    Ok(web::Json(result))
}

pub async fn get_by_heat_id(
    web::Path(heat_id): web::Path<u32>,
    db: web::Data<Pool>,
    _user: AuthorizedUser,
) -> Result<web::Json<Vec<Score>>> {
    let result = Score::find_by_heat(db.get_ref(), heat_id)
        .await.map_err(|e| {
            error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
    })?;

    Ok(web::Json(result))
}