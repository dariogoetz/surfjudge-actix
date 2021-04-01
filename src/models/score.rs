use crate::database::Pool;

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Score {
    pub surfer_id: i32,
    pub judge_id: i32,
    pub heat_id: i32,
    pub wave: i32,
    pub score: f64,
    pub interference: bool,
    pub missed: bool,
}

impl Score {
    pub async fn find_by_heat(db: &Pool, heat_id: u32) -> anyhow::Result<Vec<Self>> {
        let query = r#"
            SELECT * FROM scores s
            INNER JOIN heats h
            ON s.heat_id = h.id
            WHERE s.heat_id = $1
        "#;
        let res = sqlx::query_as::<_, Score>(query)
            .bind(heat_id)
            .fetch_all(db)
            .await?;
        Ok(res)
    }

    pub async fn find_by_heat_and_judge(db: &Pool, heat_id: u32, judge_id: u32) -> anyhow::Result<Vec<Self>> {
        let query = r#"
            SELECT * FROM scores s
            INNER JOIN heats h
            ON s.heat_id = h.id
              INNER JOIN users u
              ON s.judge_id = u.id
            WHERE s.heat_id = $1 AND s.judge_id = $2
        "#;
        let res = sqlx::query_as::<_, Score>(query)
            .bind(heat_id)
            .bind(judge_id)
            .fetch_all(db)
            .await?;
        Ok(res)
    }
}
