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

    async fn expand_option(db: &Pool, v: Option<Self>,  expand: bool) -> Option<Self> {
        if expand {
            return match v {
                Some(val) => Some(val.expand(&db).await),
                None => None
            };
        } else {
            return v
        }
    }

    async fn expand_vec(db: &Pool, v: impl std::iter::Iterator<Item=Self>, expand: bool) -> Vec<Self> {
        match expand {
            true => {
                future::join_all(v.map(|r|{ r.expand(&db) })).await
            },
            false => v.collect(),
        }
    }

    async fn find_option_bind(db: &Pool, query: &'static str, value: u32, expand: bool) -> anyhow::Result<Option<Self>> {
        let res = sqlx::query_as::<_, CategoryCore>(query)
            .bind(value)
            .fetch_optional(db)
            .await?
            .map(|c| Self::from(c));

        Ok(Self::expand_option(&db, res, expand).await)
    }

    async fn find_vec(db: &Pool, query: &'static str, expand: bool) -> anyhow::Result<Vec<Self>> {
        let res = sqlx::query_as::<_, CategoryCore>(query)
            .fetch_all(db)
            .await?
            .into_iter().map(|r| Self::from(r));
        Ok(Self::expand_vec(&db, res, expand).await)
    }

    async fn find_vec_bind(db: &Pool, query: &'static str, value: u32, expand: bool) -> anyhow::Result<Vec<Self>> {
        let res = sqlx::query_as::<_, CategoryCore>(query)
            .bind(value)
            .fetch_all(db)
            .await?
            .into_iter().map(|r| Self::from(r));
        Ok(Self::expand_vec(&db, res, expand).await)
    }


    pub async fn find_all(db: &Pool, expand: bool) -> anyhow::Result<Vec<Self>> {
        Self::find_vec(&db, r#"SELECT * FROM categories"#, expand).await
    }

    pub async fn find_by_id(db: &Pool, category_id: u32, expand: bool) -> anyhow::Result<Option<Self>> {
        Self::find_option_bind(&db, r#"SELECT * FROM categories WHERE id = $1"#, category_id, expand).await
    }

    pub async fn find_by_tournament_id(db: &Pool, tournament_id: u32, expand: bool) -> anyhow::Result<Vec<Self>> {
        Self::find_vec_bind(
            &db,
            r#"SELECT c.* FROM categories c INNER JOIN tournaments t ON c.tournament_id = t.id WHERE t.id = $1"#,
            tournament_id,
            expand
        ).await
    }
}
