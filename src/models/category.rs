use crate::database::Pool;
use crate::models::tournament::Tournament;

use futures::future;

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// this struct will be used to represent database record
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CategoryCore {
    pub id: i32,
    pub tournament_id: i32,
    pub name: String,
    pub additional_info: String,
}

// this struct will be used to represent database record
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Category {
    pub id: i32,
    pub tournament_id: i32,
    pub name: String,
    pub additional_info: String,
    pub tournament: Option<Tournament>,
}

impl From <CategoryCore> for Category {
    fn from(category: CategoryCore) -> Self {
        Category {
            id: category.id,
            tournament_id: category.tournament_id,
            name: category.name,
            additional_info: category.additional_info,
            tournament:  None,
        }
    }
}

async fn expand(db: &Pool, category_core: CategoryCore) -> anyhow::Result<Category> {
    let mut category = Category::from(category_core);
    category.tournament = Tournament::find_by_id(&db, category.tournament_id as u32).await?;
    Ok(category)
}

impl Category {
    pub async fn find_all(db: &Pool) -> anyhow::Result<Vec<Category>> {
        let categories_core = sqlx::query_as::<_, CategoryCore>(r#"SELECT * FROM categories ORDER BY id"#)
            .fetch_all(db)
            .await?;

        let mut categories = Vec::new();
        for category_core in categories_core {
            categories.push(expand(&db, category_core));
        }
        Ok(future::try_join_all(categories).await?)
    }

    pub async fn find_by_id(db: &Pool, category_id: u32) -> anyhow::Result<Option<Category>> {
        let category_core = sqlx::query_as::<_, CategoryCore>(r#"SELECT * FROM categories WHERE id = $1"#)
            .bind(category_id)
            .fetch_optional(db)
            .await?;
        let category = match category_core {
            Some(category_core) => Some(expand(&db, category_core).await?),
            None => None,
        };
        Ok(category)
    }
}
