use crate::database::Pool;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{Done, FromRow, Type};

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

    pub async fn set_heat_started(db: &Pool, heat_id: u32) -> anyhow::Result<bool> {
        let res = sqlx::query(
            r#"
INSERT INTO heat_state (heat_id, state, start_datetime, end_datetime, duration_m)
SELECT $1, $2, NOW(), NOW() + heats.duration * interval '60 seconds', heats.duration
FROM heats
WHERE heats.id = $1
ON CONFLICT (heat_id)
DO NOTHING
RETURNING heat_id;
        "#,
        )
        .bind(heat_id)
        .bind(HeatStateType::Active)
        .execute(db)
        .await?;
        Ok(res.rows_affected() > 0)
    }

    pub async fn set_heat_stopped(db: &Pool, heat_id: u32) -> anyhow::Result<bool> {
        let res = sqlx::query(
            r#"
DELETE FROM heat_state
WHERE heat_id = $1
RETURNING heat_id;
        "#,
        )
        .bind(heat_id)
        .execute(db)
        .await?;
        Ok(res.rows_affected() > 0)
    }

    pub async fn set_heat_paused(db: &Pool, heat_id: u32) -> anyhow::Result<bool> {
        let res = sqlx::query(
            r#"
UPDATE heat_state
SET
  pause_datetime = NOW(),
  remaining_time_s = GREATEST(0, EXTRACT(EPOCH FROM (end_datetime - NOW()))),
  state = $2
WHERE heat_id = $1
RETURNING heat_id;
        "#,
        )
        .bind(heat_id)
        .bind(HeatStateType::Paused)
        .execute(db)
        .await?;
        Ok(res.rows_affected() > 0)
    }

    pub async fn set_heat_unpaused(db: &Pool, heat_id: u32) -> anyhow::Result<bool> {
        let res = sqlx::query(
            r#"
UPDATE heat_state
SET
  pause_datetime = NULL,
  remaining_time_s = NULL,
  end_datetime = NOW() + remaining_time_s * interval '1 second',
  state = $2
WHERE heat_id = $1
RETURNING heat_id;
        "#,
        )
        .bind(heat_id)
        .bind(HeatStateType::Active)
        .execute(db)
        .await?;
        Ok(res.rows_affected() > 0)
    }

    pub async fn toggle_heat_pause(db: &Pool, heat_id: u32) -> anyhow::Result<bool> {
        let heat_state = Self::find_by_heat_id(db, heat_id).await?;
        if heat_state.is_none() {
            return Ok(false);
        }
        let heat_state = heat_state.unwrap();

        match heat_state.state {
            HeatStateType::Active => Self::set_heat_paused(db, heat_id).await,
            HeatStateType::Paused => Self::set_heat_unpaused(db, heat_id).await,
            _ => Ok(false),
        }
    }

    pub async fn reset_heat_time(db: &Pool, heat_id: u32) -> anyhow::Result<bool> {
        let res = sqlx::query(
            r#"
UPDATE heat_state hs
SET
  end_datetime = NOW() + h.duration * interval '60 seconds',
  remaining_time_s = 60 * h.duration
FROM heats h
WHERE hs.heat_id = $1 AND h.id = $1
RETURNING heat_id;
        "#,
        )
        .bind(heat_id)
        .execute(db)
        .await?;
        Ok(res.rows_affected() > 0)
    }
}
