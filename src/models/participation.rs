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
    fn from(participation: ParticipationCore) -> Self {
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

async fn expand(db: &Pool, participation_core: ParticipationCore) -> anyhow::Result<Participation> {
    let mut participation = Participation::from(participation_core);
    // let heat_fut = Heat::find_by_id(&db, participation.heat_id as u32);
    let lycra_color_fut = LycraColor::find_by_id(&db, participation.lycra_color_id as u32);
    let surfer_fut = Surfer::find_by_id(&db, participation.surfer_id as u32);
    let pair = future::try_join(lycra_color_fut, surfer_fut).await?;
    // participation.heat = pair.0;
    participation.lycra_color = pair.0;
    participation.surfer = pair.1;
    Ok(participation)
}

impl Participation {
    pub async fn find_all(db: &Pool) -> anyhow::Result<Vec<Participation>> {
        let participations_core = sqlx::query_as::<_, ParticipationCore>(r#"SELECT * FROM participations"#)
            .fetch_all(db)
            .await?;

        let mut participations = Vec::new();
        for participation_core in participations_core {
            participations.push(expand(&db, participation_core));
        }
        Ok(future::try_join_all(participations).await?)
    }


    pub async fn find_by_heat_id(db: &Pool, heat_id: u32) -> anyhow::Result<Vec<Participation>> {
        let participations_core = sqlx::query_as::<_, ParticipationCore>(r#"SELECT * FROM participations WHERE heat_id = $1"#)
            .bind(heat_id)
            .fetch_all(db)
            .await?;

        let mut participations = Vec::new();
        for participation_core in participations_core {
            participations.push(expand(&db, participation_core));
        }
        Ok(future::try_join_all(participations).await?)
    }
}
