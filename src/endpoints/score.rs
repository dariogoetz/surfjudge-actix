use crate::authorization::AuthorizedUser;
use crate::database::Pool;
use crate::models::score::{Score, DeleteScore};
use crate::notifier::{Channel, Notifier};

use actix_web::{error, web, Result};
use serde_json::json;

pub async fn get_by_heat_id_and_judge_id(
    path: web::Path<(u32, u32)>,
    db: web::Data<Pool>,
    _user: AuthorizedUser,
) -> Result<web::Json<Vec<Score>>> {
    let (heat_id, judge_id) = path.into_inner();
    let result = Score::find_by_heat_and_judge(db.get_ref(), heat_id, judge_id)
        .await
        .map_err(|e| {
            error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
        })?;

    Ok(web::Json(result))
}

pub async fn get_by_heat_id(
    path: web::Path<u32>,
    db: web::Data<Pool>,
    _user: AuthorizedUser,
) -> Result<web::Json<Vec<Score>>> {
    let heat_id = path.into_inner();
    let result = Score::find_by_heat(db.get_ref(), heat_id)
        .await
        .map_err(|e| {
            error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
        })?;

    Ok(web::Json(result))
}

pub async fn put(
    web::Json(score): web::Json<Score>,
    db: web::Data<Pool>,
    notifier: web::Data<Notifier>,
    user: AuthorizedUser,
) -> Result<web::Json<Option<Score>>> {
    // compare given judge_id with session
    if (user.0.id != score.judge_id as u32) && (!user.0.is_admin()) {
        return Err(error::ErrorForbidden(format!(
            "Judge '{}' not allowed to add score for judge '{}' ",
            user.0.id, score.judge_id
        )));
    }

    let result = Score::add(db.get_ref(), &score).await.map_err(|e| {
        error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
    })?;
    notifier
        .send(
            Channel::Scores,
            json!({
                "heat_id": score.heat_id,
                "judge_id": user.0.id
            }),
        )
        .unwrap();
    Ok(web::Json(result))
}

pub async fn delete(
    delete_score: web::Path<DeleteScore>,
    db: web::Data<Pool>,
    notifier: web::Data<Notifier>,
    user: AuthorizedUser,
) -> Result<web::Json<Option<Score>>> {
    // compare given judge_id with session
    if (user.0.id != delete_score.judge_id as u32) && (!user.0.is_admin()) {
        return Err(error::ErrorForbidden(format!(
            "Judge '{}' not allowed to delete score for judge '{}' ",
            user.0.id, delete_score.judge_id
        )));
    }

    let result = Score::delete(db.get_ref(), &delete_score)
        .await
        .map_err(|e| {
            error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
        })?;
    notifier
        .send(
            Channel::Scores,
            json!({
                "heat_id": delete_score.heat_id,
                "judge_id": user.0.id
            }),
        )
        .unwrap();
    Ok(web::Json(result))
}
