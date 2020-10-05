use crate::database::Pool;
use crate::models::heat::Heat;

use futures::future;

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, types::Json};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct WaveScore {
    pub surfer_id: i32,
    pub wave: i32,
    pub score: f64,
}

// this struct will be used to represent database record
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ResultCore {
    pub heat_id: i32,
    pub surfer_id: i32,
    pub total_score: f64,
    pub place: i32,
    pub wave_scores: Json<Vec<WaveScore>>,
}

// this struct will be used to represent database record
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Result {
    pub heat_id: i32,
    pub surfer_id: i32,
    pub total_score: f64,
    pub place: i32,
    pub wave_scores: Json<Vec<WaveScore>>,
    pub heat: Option<Heat>,
}
// TODO: add surfer from surfer struct

impl From<ResultCore> for Result {
    fn from(result: ResultCore) -> Self {
        Result {
            heat_id: result.heat_id,
            surfer_id: result.surfer_id,
            total_score: result.total_score,
            place: result.place,
            wave_scores: result.wave_scores,
            heat: None,
        }
    }
}

async fn expand(db: &Pool, result_core: ResultCore) -> anyhow::Result<Result> {
    let mut result = Result::from(result_core);
    // getting the heat leads to a lot of overhead
    // because the heat gets the category and the category gets the tournament
    //result.heat = Heat::find_by_id(&db, result.heat_id as u32).await?;
    Ok(result)
}

impl Result {
    pub async fn find_all(db: &Pool) -> anyhow::Result<Vec<Result>> {
        let results_core = sqlx::query_as::<_, ResultCore>(r#"SELECT * FROM results"#)
            .fetch_all(db)
            .await?;

        let mut results = Vec::new();
        for result_core in results_core {
            results.push(expand(&db, result_core));
        }
        Ok(future::try_join_all(results).await?)
    }


    pub async fn find_by_heat_id(db: &Pool, heat_id: u32) -> anyhow::Result<Vec<Result>> {
        let results_core = sqlx::query_as::<_, ResultCore>(r#"SELECT * FROM results WHERE heat_id = $1"#)
            .bind(heat_id)
            .fetch_all(db)
            .await?;

        let mut results = Vec::new();
        for result_core in results_core {
            results.push(expand(&db, result_core));
        }
        Ok(future::try_join_all(results).await?)
    }
}