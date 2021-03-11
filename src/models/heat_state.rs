use crate::database::Pool;
use crate::models::heat::Heat;

use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HeatStateError {
    #[error("Heat {0} not found")]
    NotFound(u32),
}

#[derive(Type, Debug, Serialize, Deserialize)]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum HeatStateType {
    Active,
    Paused,
    Inactive,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct HeatState {
    pub heat_id: i32,
    pub start_datetime: NaiveDateTime,
    pub end_datetime: NaiveDateTime,
    pub pause_datetime: Option<NaiveDateTime>,
    pub remaining_time_s: Option<f64>,
    pub state: HeatStateType,
    pub duration_m: f64,
    pub additional_data: Option<String>,
}

impl HeatState {
    async fn find_option_bind(
        db: &Pool,
        query: &'static str,
        value: u32,
    ) -> anyhow::Result<Option<Self>> {
        let res = sqlx::query_as::<_, HeatState>(query)
            .bind(value)
            .fetch_optional(db)
            .await?
            .map(|c| Self::from(c));

        Ok(res)
    }

    pub async fn find_by_heat_id(db: &Pool, heat_id: u32) -> anyhow::Result<Option<Self>> {
        Self::find_option_bind(
            &db,
            r#"SELECT * FROM heat_state WHERE heat_id = $1"#,
            heat_id,
        )
        .await
    }

    pub async fn set_heat_started(db: &Pool, heat_id: u32) -> anyhow::Result<()> {
        let heat = Heat::find_by_id(db, heat_id, false).await?.ok_or(HeatStateError::NotFound(heat_id))?;

        sqlx::query(
            r#"
INSERT INTO heat_state (heat_id, state, start_datetime, end_datetime, duration_m)
VALUES ($1, $2, NOW(), NOW() + $3 * interval '60 seconds', $3)
ON CONFLICT (heat_id)
DO NOTHING;
        "#,
        )
            .bind(heat_id)
            .bind(HeatStateType::Active)
            .bind(heat.duration)
            .execute(db)
            .await?;
        Ok(())
    }

    pub async fn set_heat_stopped(db: &Pool, heat_id: u32) -> anyhow::Result<()> {
        sqlx::query(
            r#"
DELETE FROM heat_state
WHERE heat_id = $1;
        "#,
        )
            .bind(heat_id)
            .execute(db)
            .await?;
        Ok(())
    }

    pub async fn set_heat_paused(db: &Pool, heat_id: u32) -> anyhow::Result<()> {
        sqlx::query(
            r#"
UPDATE heat_state
SET
  pause_datetime = NOW(),
  remaining_time_s = GREATEST(0, EXTRACT(EPOCH FROM (end_datetime - NOW())));,
  state = $1
WHERE heat_id = $2;
        "#,
        )
            .bind(heat_id)
            .bind(HeatStateType::Paused)
            .execute(db)
            .await?;
        Ok(())
    }
}
