use crate::database::Pool;
use crate::models::permission::{Permission, PermissionType};

use futures::future;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// this struct will be used to represent database record
#[derive(Debug, FromRow)]
pub struct UserCredentialsCore {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
}

// this struct will be used to represent database record
#[derive(Debug, FromRow)]
pub struct UserCredentials {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub permissions: Option<Vec<Permission>>,
}

impl From<UserCredentialsCore> for UserCredentials {
    fn from(user: UserCredentialsCore) -> UserCredentials {
        UserCredentials {
            id: user.id,
            username: user.username,
            password_hash: user.password_hash,
            permissions: None,
        }
    }
}
// this struct will be used to represent database record
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UserCore {
    pub id: i32,
    pub username: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub additional_info: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
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
            first_name: user.first_name.unwrap_or("".to_string()),
            last_name: user.last_name.unwrap_or("".to_string()),
            additional_info: user.additional_info,
            permissions: None,
        }
    }
}

impl User {
    async fn expand(mut self, db: &Pool) -> Self {
        self.permissions = Permission::find_by_user_id(&db, self.id).await.ok();
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

    pub async fn find_credentials_by_username(
        db: &Pool,
        username: &str,
    ) -> anyhow::Result<Option<UserCredentials>> {
        let mut res =
            sqlx::query_as::<_, UserCredentialsCore>(r#"SELECT * FROM users WHERE username = $1"#)
                .bind(username)
                .fetch_optional(db)
                .await?
                .map(|r| UserCredentials::from(r));
        if let Some(res) = &mut res {
            res.permissions = Permission::find_by_user_id(&db, res.id).await.ok();
        }

        Ok(res)
    }

    pub async fn find_by_id(
        db: &Pool,
        id: u32,
        expand_permissions: bool,
    ) -> anyhow::Result<Option<Self>> {
        let res = sqlx::query_as::<_, UserCore>(r#"SELECT * FROM users WHERE id = $1"#)
            .bind(id)
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
        INNER JOIN permissions
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

    pub async fn find_by_judge_assignments(
        db: &Pool,
        heat_id: u32,
        expand_permissions: bool,
    ) -> anyhow::Result<Vec<Self>> {
        let res = sqlx::query_as::<_, UserCore>(
            r#"
        SELECT u.*
        FROM users u
        INNER JOIN permissions p
        ON u.id = p.user_id
          INNER JOIN judge_assignments ja
          ON u.id = ja.judge_id
            INNER JOIN heats h
            ON h.id = ja.heat_id
        WHERE p.permission = $1
          AND h.id = $2
        "#,
        )
        .bind(PermissionType::Judge)
        .bind(heat_id)
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|r| Self::from(r));
        Ok(Self::expand_vec(&db, res, expand_permissions).await)
    }
}
