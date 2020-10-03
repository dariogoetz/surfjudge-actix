use crate::database::Pool;

use serde::{Serialize, Deserialize};
use sqlx::{FromRow, Row};
use chrono::{NaiveDateTime, Utc};
use anyhow::Result;


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
    pub additional_info: String
}

#[derive(sqlx::Type, Debug, Serialize, Deserialize)]
#[sqlx(rename_all = "lowercase")]
pub enum HeatType {
    Standard,
    Call
}


impl Heat {
    pub async fn find_all(db: &Pool) -> Result<Vec<Heat>> {
        let mut heats = vec![];
        let recs = sqlx::query_as::<_, Heat>(r#"SELECT * FROM heats ORDER BY id"#)
            .fetch_all(db)
            .await?;

        for rec in recs {
            println!("[heat]: {:?}", rec);
        }

        Ok(heats)
    }
}
