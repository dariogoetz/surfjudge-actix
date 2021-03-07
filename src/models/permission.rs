use crate::database::Pool;

use serde::{Deserialize, Serialize};
use sqlx::{Type, FromRow};



#[derive(Type, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum PermissionType {
    #[serde(rename = "ac_admin")]
    #[sqlx(rename = "ac_admin")]
    Admin,
    #[serde(rename = "ac_judge")]
    #[sqlx(rename = "ac_judge")]
    Judge,
    #[serde(rename = "ac_commentator")]
    #[sqlx(rename = "ac_commentator")]
    Commentator,
}


// this struct will be used to represent database record
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Permission {
    pub id: i32,
    pub user_id: i32,
    pub permission: PermissionType,
}

impl Permission {
    pub async fn find_by_user_id(db: &Pool, user_id: i32) -> anyhow::Result<Vec<Self>> {
        let permissions =
            sqlx::query_as::<_, Permission>(r#"SELECT * FROM permissions WHERE user_id = $1"#)
                .bind(user_id)
                .fetch_all(db)
                .await?;
        Ok(permissions)
    }
}
