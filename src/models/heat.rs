use crate::database::Pool;
use crate::models::{category::Category, participation::Participation};

use futures::future;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{Type, FromRow};

// this struct will be used to represent database record
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct HeatCore {
    pub id: i32,
    pub category_id: i32,
    pub name: String,
    pub round: i32,
    pub number_in_round: i32,
    pub start_datetime: NaiveDateTime,
    pub number_of_waves: i32,
    pub duration: f64,
    pub heat_type: HeatType,
    pub additional_info: String,
}

// this struct will be used to represent database record
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Heat {
    pub id: i32,
    pub category_id: i32,
    pub name: String,
    pub round: i32,
    pub number_in_round: i32,
    pub start_datetime: NaiveDateTime,
    pub number_of_waves: i32,
    pub duration: f64,
    pub heat_type: HeatType,
    pub additional_info: String,
    pub category: Option<Category>,
    pub participations: Option<Vec<Participation>>,
}

impl From<HeatCore> for Heat {
    fn from(heat: HeatCore) -> Heat {
        Heat {
            id: heat.id,
            category_id: heat.category_id,
            name: heat.name,
            round: heat.round,
            number_in_round: heat.number_in_round,
            start_datetime: heat.start_datetime,
            number_of_waves: heat.number_of_waves,
            duration: heat.duration,
            heat_type: heat.heat_type,
            additional_info: heat.additional_info,
            category: None,
            participations: None,

        }
    }
}

#[derive(Type, Debug, Serialize, Deserialize)]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum HeatType {
    Standard,
    Call,
}

impl Heat {
    async fn expand(mut self, db: &Pool) -> Self {
        //self.category = Category::find_by_id(&db, self.category_id as u32).await.unwrap_or(None);
        self.participations = Participation::find_by_heat_id(&db, self.id as u32, false).await.ok();
        self
    }

    pub async fn find_all(db: &Pool, expand: bool) -> anyhow::Result<Vec<Self>> {
        let heats = sqlx::query_as::<_, HeatCore>(r#"SELECT * FROM heats"#)
            .fetch_all(db)
            .await?
            .into_iter().map(|c| Self::from(c));

        let heats = match expand {
            true => {
                future::join_all(heats.map(|h|{ h.expand(&db) })).await
            },
            false => heats.collect(),
        };
        Ok(heats)
    }

    pub async fn find_by_id(db: &Pool, heat_id: u32, expand: bool) -> anyhow::Result<Option<Self>> {
        let mut heat = sqlx::query_as::<_, HeatCore>(r#"SELECT * FROM heats WHERE id = $1"#)
            .bind(heat_id)
            .fetch_optional(db)
            .await?
            .map(|c| Self::from(c));

        if expand {
            heat = match heat {
                Some(h) => Some(h.expand(&db).await),
                None => None,
            }
        }
        Ok(heat)
    }


    pub async fn find_active_heats_by_tournament_id(db: &Pool, tournament_id: u32, expand:bool) -> anyhow::Result<Vec<Self>> {
        let heats = sqlx::query_as::<_, HeatCore>(
            r#"
SELECT h.*
FROM heats h
INNER JOIN categories c
ON h.category_id = c.id
  INNER JOIN tournaments t
  ON c.tournament_id = t.id
    INNER JOIN heat_state s
    ON s.heat_id = h.id
  WHERE s.state = 'active' AND t.id = $1"#
        )
            .bind(tournament_id)
            .fetch_all(db)
            .await?
            .into_iter().map(|h| Self::from(h));

        let heats = match expand {
            true => {
                future::join_all(heats.map(|h|{ h.expand(&db) })).await
            },
            false => heats.collect(),
        };
        Ok(heats)
    }
}
