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
    fn from(heat_advancement: HeatAdvancementCore) -> Self {
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

async fn expand(db: &Pool, heat_advancement_core: HeatAdvancementCore) -> anyhow::Result<HeatAdvancement> {
    let mut heat_advancement = HeatAdvancement::from(heat_advancement_core);
    let to_heat_fut = Heat::find_by_id(&db, heat_advancement.to_heat_id as u32);
    let from_heat_fut = Heat::find_by_id(&db, heat_advancement.from_heat_id as u32);
    let pair = future::try_join(to_heat_fut, from_heat_fut).await?;
    heat_advancement.to_heat = pair.0;
    heat_advancement.from_heat = pair.1;
    Ok(heat_advancement)
}

impl HeatAdvancement {
    pub async fn find_all(db: &Pool) -> anyhow::Result<Vec<HeatAdvancement>> {
        let heat_advancements_core = sqlx::query_as::<_, HeatAdvancementCore>(r#"SELECT * FROM heat_advancements"#)
            .fetch_all(db)
            .await?;

        let mut heat_advancements = Vec::new();
        for heat_advancement_core in heat_advancements_core {
            heat_advancements.push(expand(&db, heat_advancement_core));
        }
        Ok(future::try_join_all(heat_advancements).await?)
    }


    pub async fn find_by_category_id(db: &Pool, category_id: u32) -> anyhow::Result<Vec<HeatAdvancement>> {
        let heat_advancements_core = sqlx::query_as::<_, HeatAdvancementCore>(
            r#"SELECT adv.* FROM heat_advancements adv JOIN heats ON adv.to_heat_id = heats.id WHERE heats.category_id = $1"#
        )
            .bind(category_id)
            .fetch_all(db)
            .await?;

        let mut heat_advancements = Vec::new();
        for heat_advancement_core in heat_advancements_core {
            heat_advancements.push(expand(&db, heat_advancement_core));
        }
        Ok(future::try_join_all(heat_advancements).await?)
    }

    // TODO: allow querying by to- or from_heat_id

    // TODO: get advancing surfers
}
