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

#[derive(Debug, Deserialize)]
pub struct DeleteScore {
    pub surfer_id: i32,
    pub judge_id: i32,
    pub heat_id: i32,
    pub wave: i32,
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

    pub async fn find_by_heat_and_judge(
        db: &Pool,
        heat_id: u32,
        judge_id: u32,
    ) -> anyhow::Result<Vec<Self>> {
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

    pub async fn add(db: &Pool, score: &Score) -> anyhow::Result<Option<Score>> {
        let query = r#"
        INSERT INTO scores (heat_id, judge_id, surfer_id, wave, score, missed, interference)
        (SELECT ja.heat_id, ja.judge_id, p.surfer_id, $4, $5, $6, $7
        FROM judge_assignments ja    -- only scores by assigned judges
        INNER JOIN participations p  -- only scores for participating surfers
        ON ja.heat_id = p.heat_id
        WHERE ja.heat_id = $1 AND ja.judge_id = $2 AND p.surfer_id = $3)
        ON CONFLICT (heat_id, surfer_id, judge_id, wave) DO UPDATE
        SET
          score = EXCLUDED.score,
          missed = EXCLUDED.missed,
          interference = EXCLUDED.interference
        RETURNING *
        "#;
        let res = sqlx::query_as::<_, Score>(query)
            .bind(score.heat_id)
            .bind(score.judge_id)
            .bind(score.surfer_id)
            .bind(score.wave)
            .bind(score.score)
            .bind(score.missed)
            .bind(score.interference)
            .fetch_optional(db)
            .await?;
        Ok(res)
    }

    pub async fn delete(db: &Pool, score: &DeleteScore) -> anyhow::Result<Option<Score>> {
        let query = r#"
        DELETE FROM scores s
        WHERE s.heat_id = $1 AND s.judge_id = $2 AND s.surfer_id = $3 AND s.wave = $4
        RETURNING *
        "#;
        let res = sqlx::query_as::<_, Score>(query)
            .bind(score.heat_id)
            .bind(score.judge_id)
            .bind(score.surfer_id)
            .bind(score.wave)
            .fetch_optional(db)
            .await?;
        Ok(res)
    }
}
