use crate::database::Pool;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};

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
}
