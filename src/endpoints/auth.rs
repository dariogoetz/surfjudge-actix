use crate::auth::{AuthenticatedUser, AuthorizedUser, Sessions};
use crate::database::Pool;
use crate::logging::LOG;
use crate::models::user::User;

use actix_identity::Identity;
use actix_web::{error, web, HttpResponse, Responder, Result};
use serde::{Deserialize, Serialize};
use slog::info;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Login {
    pub username: String,
    pub password: String,
}

pub async fn session_test(identity: Identity) -> Result<String> {
    if let Some(username) = identity.identity() {
        info!(LOG, "You are {:?}", username);
        Ok(format!("Welcome {:?}", username))
    } else {
        info!(LOG, "You are not logged in!");
        Ok("Welcome unknown person!".to_string())
    }
}

pub async fn protected(user: AuthorizedUser) -> Result<&'static str> {
    info!(LOG, "Accessing protected resource from {:?}", user);
    Ok("Welcome to protected land!")
}

pub async fn login(
    login: web::Json<Login>,
    db: web::Data<Pool>,
    sessions: web::Data<Sessions>,
    identity: Identity,
) -> Result<web::Json<Option<AuthenticatedUser>>> {
    let id = &login.username;

    let user = User::find_by_username(db.get_ref(), id, true)
        .await
        .map_err(|e| {
            error::ErrorInternalServerError(format!("Error fetching data from database: {:?}", e))
        })?;

    if user.is_none() {
        return Ok(web::Json(None));
    }

    // user with that username exists
    let user = user.unwrap();

    // TODO: check password

    let permissions = user
        .permissions
        .unwrap_or(Vec::new())
        .iter()
        .map(|p| p.permission.clone())
        .collect();
    let auth_user = AuthenticatedUser {
        username: id.clone(),
        permissions: permissions,
    };

    identity.remember(id.clone());

    info!(LOG, "Login user: {:?}", auth_user);
    sessions.insert(id.clone(), auth_user.clone());

    Ok(web::Json(Some(auth_user)))
}

pub async fn logout(sessions: web::Data<Sessions>, identity: Identity) -> impl Responder {
    if let Some(username) = identity.identity() {
        identity.forget();
        if let Some(user) = sessions.remove(&username) {
            info!(LOG, "Logout user {:?}", user);
        }
    } else {
        info!(LOG, "Can not log out user that is not logged in!");
    }
    HttpResponse::Unauthorized().finish()
}
