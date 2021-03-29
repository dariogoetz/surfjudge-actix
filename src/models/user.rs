use crate::database::Pool;
use crate::models::permission::{Permission, PermissionType};

use futures::future;
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
            permissions: None,
        }
    }
}

impl User {
    async fn expand_permissions(mut self, db: &Pool) -> Self {
        self.permissions = Permission::find_by_user_id(&db, self.id).await.ok();
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
    async fn expand_vec(
        db: &Pool,
        v: impl std::iter::Iterator<Item = Self>,
        expand: bool,
    ) -> Vec<Self> {
        match expand {
            true => future::join_all(v.map(|r| r.expand_permissions(&db))).await,
            false => v.collect(),
        }
    }

    pub async fn find_by_username(
        db: &Pool,
        username: &str,
        expand_permissions: bool,
    ) -> anyhow::Result<Option<Self>> {
        let res = sqlx::query_as::<_, UserCore>(r#"SELECT * FROM users WHERE username = $1"#)
            .bind(username)
            .fetch_optional(db)
            .await?
            .map(|r| Self::from(r));
        Ok(Self::expand_option(&db, res, expand_permissions).await)
    }

    pub async fn find_by_permission(
        db: &Pool,
        permission: PermissionType,
        expand_permissions: bool,
    ) -> anyhow::Result<Vec<Self>> {
        let res = sqlx::query_as::<_, UserCore>(
            r#"
        SELECT users.*
        FROM users 
        JOIN permissions
        ON users.id = permissions.user_id
        WHERE permissions.permission = $1
        "#,
        )
        .bind(permission)
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|r| Self::from(r));
        Ok(Self::expand_vec(&db, res, expand_permissions).await)
    }
}
