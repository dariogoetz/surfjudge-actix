use crate::auth::{AuthenticatedUser, Role, Sessions};
use crate::logging::LOG;

use actix_identity::Identity;
use actix_web::{web, HttpResponse, Responder, Result};
use serde::{Deserialize, Serialize};
use slog::info;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Login {
    pub username: String,
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

pub async fn protected(user: AuthenticatedUser) -> Result<&'static str> {
    info!(LOG, "Accessing protected resource from {:?}", user);
    Ok("Welcome to protected land!")
}

pub async fn login(
    login: web::Json<Login>,
    sessions: web::Data<Sessions>,
    identity: Identity,
) -> impl Responder {
    let username = login.username.to_string();

    let user = AuthenticatedUser {
        username: username.clone(),
        role: Role::Admin,
    };
    // let user = fetch_user(login).await // from db?
    identity.remember(username.clone());

    sessions.insert(username, user.clone());
    info!(LOG, "Login user: {:?}", user);
    HttpResponse::Ok().json(user)
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
