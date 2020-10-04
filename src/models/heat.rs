use crate::database::Pool;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// this struct will be used to represent database record
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Heat {
    pub id: i32,
    pub category_id: i32,
    pub name: String,
    pub round: i32,
    pub number_in_round: i32,
    pub start_datetime: NaiveDateTime,
    pub number_of_waves: i32,
    pub duration: f64,
    pub heat_type: HeatType,
    pub additional_info: String,
}

#[derive(sqlx::Type, Debug, Serialize, Deserialize)]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum HeatType {
    Standard,
    Call,
}

impl Heat {
    pub async fn find_all(db: &Pool) -> anyhow::Result<Vec<Heat>> {
        let heats = sqlx::query_as::<_, Heat>(r#"SELECT * FROM heats ORDER BY id"#)
            .fetch_all(db)
            .await?;
        Ok(heats)
    }

    pub async fn find_by_id(db: &Pool, heat_id: u32) -> anyhow::Result<Option<Heat>> {
        let heat = sqlx::query_as::<_, Heat>(r#"SELECT * FROM heats WHERE id = $1"#)
            .bind(heat_id)
            .fetch_optional(db)
            .await?;
        Ok(heat)
    }

}
