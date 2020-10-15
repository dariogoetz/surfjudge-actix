use crate::database::Pool;

use chrono::NaiveDate;

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// this struct will be used to represent database record
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Tournament {
    pub id: i32,
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub additional_info: Option<String>,
}


impl Tournament {
    pub async fn find_all(db: &Pool) -> anyhow::Result<Vec<Tournament>> {
        let tournaments = sqlx::query_as::<_, Tournament>(r#"SELECT * FROM tournaments"#)
            .fetch_all(db)
            .await?;
        Ok(tournaments)
    }

    pub async fn find_by_id(db: &Pool, tournament_id: u32) -> anyhow::Result<Option<Tournament>> {
        let tournament = sqlx::query_as::<_, Tournament>(r#"SELECT * FROM tournaments WHERE id = $1"#)
            .bind(tournament_id)
            .fetch_optional(db)
            .await?;
        Ok(tournament)
    }
}
