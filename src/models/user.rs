use crate::database::Pool;
use crate::models::permission::Permission;

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// this struct will be used to represent database record
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UserCore {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub first_name: String,
    pub last_name: String,
    pub additional_info: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub first_name: String,
    pub last_name: String,
    pub additional_info: Option<String>,
    pub permissions: Option<Vec<Permission>>,
}

impl From<UserCore> for User {
    fn from(user: UserCore) -> User {
        User {
            id: user.id,
            username: user.username,
            password_hash: user.password_hash,
            first_name: user.first_name,
            last_name: user.last_name,
            additional_info: user.additional_info,
            permissions: None
        }
    }
}

impl User {
    async fn expand_permissions(mut self, db: &Pool) -> Self {
        self.permissions = Permission::find_by_user_id(&db, self.id as u32)
            .await
            .ok();
        self
    }
    async fn expand_option(db: &Pool, v: Option<Self>, expand: bool) -> Option<Self> {
        if expand {
            return match v {
                Some(val) => Some(val.expand_permissions(&db).await),
                None => None,
            };
        } else {
            return v;
        }
    }



    pub async fn find_by_username(db: &Pool, username: &str, expand_permissions: bool) -> anyhow::Result<Option<Self>> {
        let res =
            sqlx::query_as::<_, UserCore>(r#"SELECT * FROM users WHERE username = $1"#)
                .bind(username)
                .fetch_optional(db)
                .await?
                .map(|r| Self::from(r));
        Ok(Self::expand_option(&db, res, expand_permissions).await)
    }

    pub async fn find_by_id(db: &Pool, user_id: u32, expand_permissions: bool) -> anyhow::Result<Option<Self>> {
        let res =
            sqlx::query_as::<_, UserCore>(r#"SELECT * FROM users WHERE id = $1"#)
                .bind(user_id)
                .fetch_optional(db)
                .await?
                .map(|r| Self::from(r));
        Ok(Self::expand_option(&db, res, expand_permissions).await)
    }
}
