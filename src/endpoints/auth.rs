use crate::authentication::{authenticate_user, AuthenticatedUser, Sessions};
use crate::authorization::AuthorizedUser;
use crate::database::Pool;
use crate::logging::LOG;

use actix_identity::Identity;
use actix_web::{web, HttpResponse, Responder, Result};
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
    let user = authenticate_user(db.get_ref(), &login.username, &login.password).await;

    if let Some(user) = &user {
        info!(LOG, "Login user: {:?}", user);
        identity.remember(login.username.clone());
        sessions.insert(login.username.clone(), user.clone());
    }

    Ok(web::Json(user))
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
