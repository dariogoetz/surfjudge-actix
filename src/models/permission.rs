use crate::database::Pool;

use serde::{Deserialize, Serialize};
use sqlx::{Type, FromRow};



#[derive(Type, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum PermissionType {
    Admin,
    Judge,
    Commentator,
}


// this struct will be used to represent database record
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Permission {
    pub id: i32,
    pub user_id: String,
    pub permission: PermissionType,
}

impl Permission {
    pub async fn find_by_user_id(db: &Pool, user_id: u32) -> anyhow::Result<Vec<Self>> {
        let permissions =
            sqlx::query_as::<_, Permission>(r#"SELECT * FROM permissions WHERE user_id = $1"#)
                .bind(user_id)
                .fetch_all(db)
                .await?;
        Ok(permissions)
    }
}
