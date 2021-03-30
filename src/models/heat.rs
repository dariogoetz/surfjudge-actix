use crate::database::Pool;
use crate::models::participation::Participation; //{category::Category, participation::Participation};

use chrono::NaiveDateTime;
use futures::future;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};

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
    pub additional_info: Option<String>,
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
    pub additional_info: Option<String>,
    //pub category: Option<Category>,
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
            //category: None,
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
        //self.category = Category::find_by_id(&db, self.category_id as u32, false).await.unwrap_or(None);
        self.participations = Participation::find_by_heat_id(&db, self.id as u32, true)
            .await
            .ok();
        self
    }

    async fn expand_option(db: &Pool, v: Option<Self>, expand: bool) -> Option<Self> {
        if expand {
            return match v {
                Some(val) => Some(val.expand(&db).await),
                None => None,
            };
        } else {
            return v;
        }
    }

    async fn expand_vec(
        db: &Pool,
        v: impl std::iter::Iterator<Item = Self>,
        expand: bool,
    ) -> Vec<Self> {
        match expand {
            true => future::join_all(v.map(|r| r.expand(&db))).await,
            false => v.collect(),
        }
    }

    async fn find_option_bind(
        db: &Pool,
        query: &'static str,
        value: u32,
        expand: bool,
    ) -> anyhow::Result<Option<Self>> {
        let res = sqlx::query_as::<_, HeatCore>(query)
            .bind(value)
            .fetch_optional(db)
            .await?
            .map(|c| Self::from(c));

        Ok(Self::expand_option(&db, res, expand).await)
    }

    async fn find_vec(db: &Pool, query: &'static str, expand: bool) -> anyhow::Result<Vec<Self>> {
        let res = sqlx::query_as::<_, HeatCore>(query)
            .fetch_all(db)
            .await?
            .into_iter()
            .map(|r| Self::from(r));
        Ok(Self::expand_vec(&db, res, expand).await)
    }

    async fn find_vec_bind(
        db: &Pool,
        query: &'static str,
        value: u32,
        expand: bool,
    ) -> anyhow::Result<Vec<Self>> {
        let res = sqlx::query_as::<_, HeatCore>(query)
            .bind(value)
            .fetch_all(db)
            .await?
            .into_iter()
            .map(|r| Self::from(r));
        Ok(Self::expand_vec(&db, res, expand).await)
    }

    pub async fn find_all(db: &Pool, expand: bool) -> anyhow::Result<Vec<Self>> {
        Self::find_vec(&db, r#"SELECT * FROM heats"#, expand).await
    }

    pub async fn find_by_id(db: &Pool, heat_id: u32, expand: bool) -> anyhow::Result<Option<Self>> {
        Self::find_option_bind(&db, r#"SELECT * FROM heats WHERE id = $1"#, heat_id, expand).await
    }

    pub async fn find_by_category_id(
        db: &Pool,
        category_id: u32,
        expand: bool,
    ) -> anyhow::Result<Vec<Self>> {
        Self::find_vec_bind(
            &db,
            r#"SELECT * FROM heats WHERE category_id = $1"#,
            category_id,
            expand,
        )
        .await
    }

    pub async fn find_active_heats_by_category_id(
        db: &Pool,
        category_id: u32,
        expand: bool,
    ) -> anyhow::Result<Vec<Self>> {
        Self::find_vec_bind(
            &db,
            r#"
                SELECT h.*
                FROM heats h
                INNER JOIN heat_state s
                ON s.heat_id = h.id
                WHERE s.state in ('active', 'paused') AND h.category_id = $1
            "#,
            category_id,
            expand,
        )
        .await
    }

    pub async fn find_active_heats_by_tournament_id(
        db: &Pool,
        tournament_id: u32,
        expand: bool,
    ) -> anyhow::Result<Vec<Self>> {
        Self::find_vec_bind(
            &db,
            r#"
                SELECT h.*
                FROM heats h
                INNER JOIN categories c
                ON h.category_id = c.id
                  INNER JOIN heat_state s
                  ON s.heat_id = h.id
                WHERE s.state in ('active', 'paused') AND c.tournament_id = $1
            "#,
            tournament_id,
            expand,
        )
        .await
    }

    pub async fn find_active_heats_by_judge_id(
        db: &Pool,
        judge_id: u32,
        expand: bool,
    ) -> anyhow::Result<Vec<Self>> {
        Self::find_vec_bind(
            &db,
            r#"
                SELECT h.*
                FROM heats h
                INNER JOIN heat_state s
                ON s.heat_id = h.id
                  INNER JOIN judge_assignments ja
                  ON h.id = ja.heat_id
                WHERE s.state in ('active', 'paused')
                AND ja.judge_id = $1
            "#,
            judge_id,
            expand,
        )
        .await
    }

    pub async fn find_active_heats(db: &Pool, expand: bool) -> anyhow::Result<Vec<Self>> {
        Self::find_vec(
            &db,
            r#"
                SELECT h.*
                FROM heats h
                INNER JOIN heat_state s
                ON s.heat_id = h.id
                WHERE s.state in ('active', 'paused')
            "#,
            expand,
        )
        .await
    }
}
