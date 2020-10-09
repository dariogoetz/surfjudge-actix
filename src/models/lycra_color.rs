use crate::database::Pool;

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// this struct will be used to represent database record
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct LycraColor {
    pub id: i32,
    pub seed: i32,
    pub name: String,
    pub hex: String,
}


impl LycraColor {
    pub async fn find_all(db: &Pool) -> anyhow::Result<Vec<LycraColor>> {
        let lycra_colors = sqlx::query_as::<_, LycraColor>(r#"SELECT * FROM lycra_colors ORDER BY id"#)
            .fetch_all(db)
            .await?;
        Ok(lycra_colors)
    }

    pub async fn find_by_id(db: &Pool, lycra_color_id: u32) -> anyhow::Result<Option<LycraColor>> {
        let tournament = sqlx::query_as::<_, LycraColor>(r#"SELECT * FROM lycra_colors WHERE id = $1"#)
            .bind(lycra_color_id)
            .fetch_optional(db)
            .await?;
        Ok(tournament)
    }
}
