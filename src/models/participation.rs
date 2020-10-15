use crate::database::Pool;
use crate::models::{surfer::Surfer, lycra_color::LycraColor};

use futures::future;

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// this struct will be used to represent database record
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ParticipationCore {
    pub surfer_id: i32,
    pub heat_id: i32,
    pub lycra_color_id: i32,
    pub seed: i32,
}

// this struct will be used to represent database record
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Participation {
    pub surfer_id: i32,
    pub heat_id: i32,
    pub lycra_color_id: i32,
    pub seed: i32,
    //pub heat: Option<Heat>,
    pub surfer: Option<Surfer>,
    pub lycra_color: Option<LycraColor>,
}

impl From<ParticipationCore> for Participation {
    fn from(participation: ParticipationCore) -> Participation {
        Participation {
            surfer_id: participation.surfer_id,
            heat_id: participation.heat_id,
            lycra_color_id: participation.lycra_color_id,
            seed: participation.seed,
            //heat: None,
            surfer: None,
            lycra_color: None,
        }
    }
}


impl Participation {
    async fn expand(mut self, db: &Pool) -> Self {
        // let heat_fut = Heat::find_by_id(&db, participation.heat_id as u32);
        self.lycra_color = LycraColor::find_by_id(&db, self.lycra_color_id as u32).await.unwrap_or(None);
        self.surfer = Surfer::find_by_id(&db, self.surfer_id as u32).await.unwrap_or(None);
        self
    }

    pub async fn find_all(db: &Pool, expand: bool) -> anyhow::Result<Vec<Self>> {
        let participations = sqlx::query_as::<_, ParticipationCore>(r#"SELECT * FROM participations"#)
            .fetch_all(db)
            .await?
            .into_iter().map(|p| Self::from(p));

        let participations = match expand {
            true => {
                future::join_all(participations.map(|p|{ p.expand(&db) })).await
            },
            false => participations.collect(),
        };
        Ok(participations)
    }

    pub async fn find_by_heat_id(db: &Pool, heat_id: u32, expand: bool) -> anyhow::Result<Vec<Self>> {
        let participations = sqlx::query_as::<_, ParticipationCore>(r#"SELECT * FROM participations WHERE heat_id = $1"#)
            .bind(heat_id)
            .fetch_all(db)
            .await?
            .into_iter().map(|p| Self::from(p));

        let participations = match expand {
            true => {
                future::join_all(participations.map(|p|{ p.expand(&db) })).await
            },
            false => participations.collect(),
        };
        Ok(participations)
    }
}
