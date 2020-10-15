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

    async fn expand_vec(db: &Pool, v: impl std::iter::Iterator<Item=Self>, expand: bool) -> Vec<Self> {
        match expand {
            true => {
                future::join_all(v.map(|r|{ r.expand(&db) })).await
            },
            false => v.collect(),
        }
    }

    async fn find_vec(db: &Pool, query: &'static str, expand: bool) -> anyhow::Result<Vec<Self>> {
        let res = sqlx::query_as::<_, ResultCore>(query)
            .fetch_all(db)
            .await?
            .into_iter().map(|r| Self::from(r));
        Ok(Self::expand_vec(&db, res, expand).await)
    }

    async fn find_vec_bind(db: &Pool, query: &'static str, value: u32, expand: bool) -> anyhow::Result<Vec<Self>> {
        let res = sqlx::query_as::<_, ResultCore>(query)
            .bind(value)
            .fetch_all(db)
            .await?
            .into_iter().map(|r| Self::from(r));
        Ok(Self::expand_vec(&db, res, expand).await)
    }

    
    pub async fn find_all(db: &Pool, expand: bool) -> anyhow::Result<Vec<Self>> {
        Self::find_vec(&db, r#"SELECT * FROM results"#, expand).await
    }


    pub async fn find_by_heat_id(db: &Pool, heat_id: u32, expand: bool) -> anyhow::Result<Vec<Self>> {
        Self::find_vec_bind(&db, r#"SELECT * FROM results WHERE heat_id = $1"#, heat_id, expand).await
    }
}
