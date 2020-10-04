use crate::database::Pool;

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// this struct will be used to represent database record
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Category {
    pub id: i32,
    pub tournament_id: i32,
    pub name: String,
    pub additional_info: String,
}


impl Category {
    pub async fn find_all(db: &Pool) -> anyhow::Result<Vec<Category>> {
        let categories = sqlx::query_as::<_, Category>(r#"SELECT * FROM categories ORDER BY id"#)
            .fetch_all(db)
            .await?;
        Ok(categories)
    }

    pub async fn find_by_id(db: &Pool, category_id: u32) -> anyhow::Result<Option<Category>> {
        let category = sqlx::query_as::<_, Category>(r#"SELECT * FROM categories WHERE id = $1"#)
            .bind(category_id)
            .fetch_optional(db)
            .await?;
        Ok(category)
    }

}
