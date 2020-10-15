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
    fn from(result: ResultCore) -> Result {
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


impl Result {
    async fn expand(mut self, db: &Pool) -> Self {
        // getting the heat leads to a lot of overhead
        // because the heat gets the category and the category gets the tournament
        //self.heat = Heat::find_by_id(&db, self.heat_id as u32, false).await.unwrap_or(None);
        self
    }
    
    pub async fn find_all(db: &Pool, expand: bool) -> anyhow::Result<Vec<Self>> {
        let results = sqlx::query_as::<_, ResultCore>(r#"SELECT * FROM results"#)
            .fetch_all(db)
            .await?
            .into_iter().map(|r| Self::from(r));

        let results = match expand {
            true => {
                future::join_all(results.map(|r|{ r.expand(&db) })).await
            },
            false => results.collect(),
        };
        Ok(results)
    }


    pub async fn find_by_heat_id(db: &Pool, heat_id: u32, expand: bool) -> anyhow::Result<Vec<Self>> {
        let results = sqlx::query_as::<_, ResultCore>(r#"SELECT * FROM results WHERE heat_id = $1"#)
            .bind(heat_id)
            .fetch_all(db)
            .await?
            .into_iter().map(|r| Self::from(r));

        let results = match expand {
            true => {
                future::join_all(results.map(|r|{ r.expand(&db) })).await
            },
            false => results.collect(),
        };
        Ok(results)
    }
}
