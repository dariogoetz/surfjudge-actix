use crate::database::Pool;
use crate::models::{heat::Heat, lycra_color::LycraColor};

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

#[derive(Debug, Serialize, Deserialize)]
pub struct AdvancingSurfer {
    pub surfer_id: i32,
    pub heat_id: i32,
    pub seed: i32,
    pub surfer: Option<Surfer>,
    pub heat: Option<Heat>,
    pub lycra_color: Option<LycraColor>,
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
}
