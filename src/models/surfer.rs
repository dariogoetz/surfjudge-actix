use crate::database::Pool;
use crate::models::{heat::Heat, lycra_color::LycraColor};

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use futures::TryStreamExt;

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

    pub async fn find_surfers_advancing_to_heat(db: &Pool, heat_id: u32) -> anyhow::Result<Vec<AdvancingSurfer>> {
        let mut rows = sqlx::query(
            r#"SELECT r.surfer_id, r.heat_id, adv.seed
            FROM surfers s
                INNER JOIN results r
                ON s.id = r.surfer_id
                    INNER JOIN heat_advancements adv
                    ON adv.to_heat_id = r.heat_id
            WHERE adv.to_heat_id = $1"#
        )
            .bind(heat_id)
            .fetch(db);

        let mut advs = Vec::new();
        while let Some(row) = rows.try_next().await? {
            let mut adv = AdvancingSurfer {
                surfer_id: row.try_get("surfer_id")?,
                heat_id: row.try_get("heat_id")?,
                seed: row.try_get("seed")?,
                surfer: None,
                heat: None,
                lycra_color: None,
            };
            adv.heat = Heat::find_by_id(&db, adv.heat_id as u32, false).await.unwrap_or(None);
            adv.surfer = Surfer::find_by_id(&db, adv.surfer_id as u32).await.unwrap_or(None);
            advs.push(adv);
        }

        // TODO: fix lycra color if heat is a call
        Ok(advs)
    }
}
