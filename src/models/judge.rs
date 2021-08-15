use crate::database::Pool;
use crate::models::user::User;

use chrono::NaiveDateTime;
use futures::future;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// this struct represents a judging request database record
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct JudgingRequestCore {
    pub judge_id: i32,
    pub expire_date: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JudgingRequest {
    pub judge_id: i32,
    pub expire_date: NaiveDateTime,
    pub judge: Option<User>,
}

impl From<JudgingRequestCore> for JudgingRequest {
    fn from(req: JudgingRequestCore) -> JudgingRequest {
        JudgingRequest {
            judge_id: req.judge_id,
            expire_date: req.expire_date,
            judge: None,
        }
    }
}

impl JudgingRequest {
    async fn expand(mut self, db: &Pool) -> Self {
        self.judge = User::find_by_id(&db, self.judge_id as u32, false)
            .await
            .unwrap_or(None);
        self
    }

    async fn expand_vec(
        db: &Pool,
        v: impl std::iter::Iterator<Item = Self>,
        expand: bool,
    ) -> Vec<Self> {
        match expand {
            true => future::join_all(v.map(|r| r.expand(&db))).await,
            false => v.collect(),
        }
    }

    // TODO: invalidate old judging requests

    async fn find_vec(db: &Pool, query: &'static str, expand: bool) -> anyhow::Result<Vec<Self>> {
        let res = sqlx::query_as::<_, JudgingRequestCore>(query)
            .fetch_all(db)
            .await?
            .into_iter()
            .map(|r| Self::from(r));
        Ok(Self::expand_vec(&db, res, expand).await)
    }

    pub async fn find_all(db: &Pool, expand: bool) -> anyhow::Result<Vec<Self>> {
        Self::find_vec(&db, r#"SELECT * FROM judging_requests"#, expand).await
    }

    pub async fn add(db: &Pool, judge_id: u32, expire_s: u32) -> anyhow::Result<bool> {
        let res = sqlx::query(
            r#"
INSERT INTO judging_requests (judge_id, expire_date)
VALUES ($1, NOW() + interval '$2' second)
ON CONFLICT (judge_id) DO UPDATE
SET expire_date = EXCLUDED.expire_date
RETURNING judge_id;
        "#,
        )
        .bind(judge_id)
        .bind(expire_s)
        .execute(db)
        .await?;
        Ok(res.rows_affected() > 0)
    }
}


// this struct represents a judging request database record
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct JudgingAssignment {
    pub judge_id: i32,
    pub heat_id: i32,
}

impl JudgingAssignment {
    pub async fn add(db: &Pool, heat_id: u32, judge_id: u32) -> anyhow::Result<bool> {
        let res = sqlx::query(
            r#"
INSERT INTO judge_assignments (heat_id, judge_id)
VALUES ($1, $2)
ON CONFLICT (heat_id, judge_id) DO NOTHING;
        "#,
        )
        .bind(heat_id)
        .bind(judge_id)
        .execute(db)
        .await?;
        Ok(res.rows_affected() > 0)
    }

    pub async fn delete(db: &Pool, heat_id: u32, judge_id: u32) -> anyhow::Result<bool> {
        let res = sqlx::query(
            r#"
DELETE FROM judge_assignments
WHERE heat_id = $1 AND judge_id = $2;
        "#,
        )
        .bind(heat_id)
        .bind(judge_id)
        .execute(db)
        .await?;
        Ok(res.rows_affected() > 0)
    }
}
