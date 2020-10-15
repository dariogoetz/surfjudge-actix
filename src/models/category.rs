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
    pub additional_info: Option<String>,
}

// this struct will be used to represent database record
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Category {
    pub id: i32,
    pub tournament_id: i32,
    pub name: String,
    pub additional_info: Option<String>,
    pub tournament: Option<Tournament>,
}

impl From <CategoryCore> for Category {
    fn from(category: CategoryCore) -> Category {
        Category {
            id: category.id,
            tournament_id: category.tournament_id,
            name: category.name,
            additional_info: category.additional_info,
            tournament:  None,
        }
    }
}

impl Category {

    async fn expand(mut self, db: &Pool) -> Self {
        self.tournament = Tournament::find_by_id(&db, self.tournament_id as u32).await.unwrap_or(None);
        self
    }

    pub async fn find_all(db: &Pool, expand: bool) -> anyhow::Result<Vec<Self>> {
        let categories = sqlx::query_as::<_, CategoryCore>(r#"SELECT * FROM categories"#)
            .fetch_all(db)
            .await?
            .into_iter().map(|c| Self::from(c));

        let categories = match expand {
            true => {
                future::join_all(categories.map(|c|{ c.expand(&db) })).await
            },
            false => categories.collect(),
        };
        Ok(categories)
    }

    pub async fn find_by_id(db: &Pool, category_id: u32, expand: bool) -> anyhow::Result<Option<Self>> {
        let mut category = sqlx::query_as::<_, CategoryCore>(r#"SELECT * FROM categories WHERE id = $1"#)
            .bind(category_id)
            .fetch_optional(db)
            .await?
            .map(|c| Self::from(c));

        if expand {
            category = match category {
                Some(c) => Some(c.expand(&db).await),
                None => None,
            };
        }
        Ok(category)
    }

    pub async fn find_by_tournament_id(db: &Pool, tournament_id: u32, expand: bool) -> anyhow::Result<Vec<Self>> {
        let categories = sqlx::query_as::<_, CategoryCore>(
            r#"SELECT c.* FROM categories c INNER JOIN tournaments t ON c.tournament_id = t.id WHERE t.id = $1"#
        )
            .bind(tournament_id)
            .fetch_all(db)
            .await?
            .into_iter().map(|c| Self::from(c));

        let categories = match expand {
            true => {
                future::join_all(categories.map(|c|{ c.expand(&db) })).await
            },
            false => categories.collect(),
        };
        Ok(categories)
    }
}
