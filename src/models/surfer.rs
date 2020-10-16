use crate::database::Pool;

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// this struct will be used to represent database record
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Surfer {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub country: Option<String>,
    pub additional_info: Option<String>,
}


impl Surfer {
    pub async fn find_all(db: &Pool) -> anyhow::Result<Vec<Self>> {
        let surfers = sqlx::query_as::<_, Surfer>(r#"SELECT * FROM surfers"#)
            .fetch_all(db)
            .await?;
        Ok(surfers)
    }

    pub async fn find_by_id(db: &Pool, surfer_id: u32) -> anyhow::Result<Option<Self>> {
        let surfer = sqlx::query_as::<_, Surfer>(r#"SELECT * FROM surfers WHERE id = $1"#)
            .bind(surfer_id)
            .fetch_optional(db)
            .await?;
        Ok(surfer)
    }

    pub async fn find_surfers_advancing_to_heat(db: &Pool, heat_id: u32) -> anyhow::Result<Vec<Self>> {
        let adv = sqlx::query(
            r#"SELECT r.surfer_id, r.heat_id, adv.seed
            FROM surfers s
                INNER JOIN results r
                ON s.id = r.surfer_id
                    INNER JOIN heat_advancements adv
                    ON adv.to_heat_id = r.heat_id
            WHERE adv.to_heat_id = $1"#
        )
            .bind(heat_id)
            .fetch_all(db)
            .await?;

        // TODO: iterate through rows, collect heats and surfers and generate corresponding result structure
        // TODO: fix lycra color if heat is a call
        Ok(Vec::new())
    }
}
