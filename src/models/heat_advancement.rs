use crate::database::Pool;
use crate::models::{heat::Heat};

use futures::future;

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// this struct will be used to represent database record
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct HeatAdvancementCore {
    pub to_heat_id: i32,
    pub seed: i32,
    pub from_heat_id: i32,
    pub place: i32,
}

// this struct will be used to represent database record
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct HeatAdvancement {
    pub to_heat_id: i32,
    pub seed: i32,
    pub from_heat_id: i32,
    pub place: i32,
    pub from_heat: Option<Heat>,
    pub to_heat: Option<Heat>,
}

impl From<HeatAdvancementCore> for HeatAdvancement {
    fn from(heat_advancement: HeatAdvancementCore) -> HeatAdvancement {
        HeatAdvancement {
            to_heat_id: heat_advancement.to_heat_id,
            seed: heat_advancement.seed,
            from_heat_id: heat_advancement.from_heat_id,
            place: heat_advancement.place,
            to_heat: None,
            from_heat: None,
        }
    }
}


impl HeatAdvancement {
    async fn expand(mut self, db: &Pool) -> Self {
        self.to_heat = Heat::find_by_id(&db, self.to_heat_id as u32, false).await.unwrap_or(None);
        self.from_heat = Heat::find_by_id(&db, self.from_heat_id as u32, false).await.unwrap_or(None);
        self
    }

    pub async fn find_all(db: &Pool, expand: bool) -> anyhow::Result<Vec<Self>> {
        let heat_advancements = sqlx::query_as::<_, HeatAdvancementCore>(r#"SELECT * FROM heat_advancements"#)
            .fetch_all(db)
            .await?
            .into_iter().map(|ha| Self::from(ha));

        let heat_advancements = match expand {
            true => {
                future::join_all(heat_advancements.map(|ha|{ ha.expand(&db) })).await
            },
            false => heat_advancements.collect(),
        };
        Ok(heat_advancements)
    }

    pub async fn find_by_category_id(db: &Pool, category_id: u32, expand: bool) -> anyhow::Result<Vec<Self>> {
        let heat_advancements = sqlx::query_as::<_, HeatAdvancementCore>(
            r#"SELECT adv.* FROM heat_advancements adv JOIN heats ON adv.to_heat_id = heats.id WHERE heats.category_id = $1"#
        )
            .bind(category_id)
            .fetch_all(db)
            .await?
            .into_iter().map(|ha| Self::from(ha));

        let heat_advancements = match expand {
            true => {
                future::join_all(heat_advancements.map(|ha|{ ha.expand(&db) })).await
            },
            false => heat_advancements.collect(),
        };
        Ok(heat_advancements)
    }

    // TODO: remove duplication (to_heat_id and from_heat_id queries are basically identical)
    pub async fn find_by_to_heat_id(db: &Pool, value: u32, expand: bool) -> anyhow::Result<Vec<Self>> {
        let heat_advancements = sqlx::query_as::<_, HeatAdvancementCore>(
            r#"SELECT * FROM heat_advancements WHERE to_heat_id = $1"#
        )
            .bind(value)
            .fetch_all(db)
            .await?
            .into_iter().map(|ha| Self::from(ha));

        let heat_advancements = match expand {
            true => {
                future::join_all(heat_advancements.map(|ha|{ ha.expand(&db) })).await
            },
            false => heat_advancements.collect(),
        };
        Ok(heat_advancements)
    }
    
    pub async fn find_by_from_heat_id(db: &Pool, value: u32, expand: bool) -> anyhow::Result<Vec<Self>> {
        let heat_advancements = sqlx::query_as::<_, HeatAdvancementCore>(
            r#"SELECT * FROM heat_advancements WHERE from_heat_id = $1"#
        )
            .bind(value)
            .fetch_all(db)
            .await?
            .into_iter().map(|ha| Self::from(ha));

        let heat_advancements = match expand {
            true => {
                future::join_all(heat_advancements.map(|ha|{ ha.expand(&db) })).await
            },
            false => heat_advancements.collect(),
        };
        Ok(heat_advancements)
    }

    // TODO: get advancing surfers
}
