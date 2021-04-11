use crate::database::Pool;
use crate::logging::LOG;
use crate::models::{permission::PermissionType, user::User};

use actix_identity::Identity;
use actix_web::{dev::Payload, error::ErrorUnauthorized, web, Error, FromRequest, HttpRequest};
use anyhow::Result;
use bcrypt::verify;
use dashmap::DashMap;
use futures::future::Future;
use oso::PolarClass;
use serde::Serialize;
use slog::{debug, info, warn};
use std::pin::Pin;

#[derive(Serialize, Debug, Default, Clone, PolarClass)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticatedUser {
    pub id: u32,
    pub username: String,
    pub permissions: Vec<PermissionType>,
    pub first_name: String,
    pub last_name: String,
}

impl AuthenticatedUser {
    fn has_permission(&self, permission: &PermissionType) -> bool {
        self.permissions.iter().any(|r| r == permission)
    }

    pub fn is_admin(&self) -> bool {
        self.has_permission(&PermissionType::Admin)
    }

    pub fn is_judge(&self) -> bool {
        self.has_permission(&PermissionType::Judge)
    }

    pub fn is_commentator(&self) -> bool {
        self.has_permission(&PermissionType::Commentator)
    }
}

pub type Sessions = DashMap<String, AuthenticatedUser>;

impl FromRequest for AuthenticatedUser {
    type Config = ();
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<AuthenticatedUser, Error>>>>;

    fn from_request(req: &HttpRequest, pl: &mut Payload) -> Self::Future {
        let sessions: Option<&web::Data<Sessions>> = req.app_data();
        if sessions.is_none() {
            warn!(LOG, "Sessions is empty!");
            return Box::pin(async { Err(ErrorUnauthorized("unauthorized")) });
        }

        let sessions = sessions.unwrap().clone();
        let fut = Identity::from_request(req, pl);
        Box::pin(async move {
            let identity = fut.await?;
            if let Some(username) = identity.identity() {
                if let Some(user) = sessions.get(&username).map(|x| x.clone()) {
                    return Ok(user);
                } else {
                    warn!(
                        LOG,
                        "Logging out user {:?} for which no session data is available!", username
                    );
                    identity.forget();
                }
            } else {
                debug!(LOG, "User not logged in!");
            };
            Err(ErrorUnauthorized("unauthorized"))
        })
    }
}

fn verify_password(password: &str, hash: &str) -> bool {
    verify(password, hash).unwrap_or(false)
}

pub async fn authenticate_user(
    db: &Pool,
    username: &str,
    password: &str,
) -> Option<AuthenticatedUser> {
    // find user credentials in database
    let user_credentials = User::find_credentials_by_username(db, username)
        .await
        .ok()
        .unwrap_or(None);

    if user_credentials.is_none() {
        return None;
    }

    // user with that username exists
    let user_credentials = user_credentials.unwrap();

    // verify password
    if !verify_password(password, &user_credentials.password_hash) {
        info!(LOG, "Wrong password entered for user {:?}", username);
        return None;
    }

    // collect PermissionTypes for AuthenticatedUser
    let permissions = user_credentials
        .permissions
        .unwrap_or(Vec::new())
        .iter()
        .map(|p| p.permission.clone())
        .collect();

    // find user credentials in database
    let user = User::find_by_id(db, user_credentials.id as u32, false)
        .await
        .unwrap()
        .unwrap();

    Some(AuthenticatedUser {
        id: user.id as u32,
        username: username.to_string(),
        permissions: permissions,
        first_name: user.first_name,
        last_name: user.last_name,
    })
}
