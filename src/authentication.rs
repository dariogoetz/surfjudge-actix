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
use slog::{info, warn};
use std::pin::Pin;

#[derive(Serialize, Debug, Default, Clone, PolarClass)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticatedUser {
    pub username: String,
    pub permissions: Vec<PermissionType>,
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
                warn!(LOG, "User not logged in!")
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
    // find user in database
    let user = User::find_credentials_by_username(db, username)
        .await
        .ok()
        .unwrap_or(None);

    if user.is_none() {
        return None;
    }

    // user with that username exists
    let user = user.unwrap();

    // verify password
    if !verify_password(password, &user.password_hash) {
        info!(LOG, "Wrong password entered for user {:?}", username);
        return None;
    }

    // collect PermissionTypes for AuthenticatedUser
    let permissions = user
        .permissions
        .unwrap_or(Vec::new())
        .iter()
        .map(|p| p.permission.clone())
        .collect();

    Some(AuthenticatedUser {
        username: username.to_string(),
        permissions: permissions,
    })
}
