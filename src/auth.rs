use crate::logging::LOG;

use actix_identity::Identity;
use actix_web::{dev::Payload, error::ErrorUnauthorized, web, Error, FromRequest, HttpRequest};
use dashmap::DashMap;
use futures::future::Future;
use serde::{Deserialize, Serialize};
use slog::warn;
use std::pin::Pin;

pub type Sessions = DashMap<String, AuthenticatedUser>;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticatedUser {
    pub username: String,
    pub role: Role,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Role {
    Admin,
    Judge,
    Commentator,
    None,
}

impl Default for Role {
    fn default() -> Self {
        Role::None
    }
}

impl FromRequest for AuthenticatedUser {
    type Config = ();
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<AuthenticatedUser, Error>>>>;

    fn from_request(req: &HttpRequest, pl: &mut Payload) -> Self::Future {
        let fut = Identity::from_request(req, pl);
        let sessions: Option<&web::Data<Sessions>> = req.app_data();
        if sessions.is_none() {
            warn!(LOG, "Sessions is empty!");
            return Box::pin(async { Err(ErrorUnauthorized("unauthorized")) });
        }

        let sessions = sessions.unwrap().clone();
        Box::pin(async move {
            if let Some(username) = fut.await?.identity() {
                if let Some(user) = sessions.get(&username).map(|x| x.clone()) {
                    return Ok(user);
                }
            };

            Err(ErrorUnauthorized("unauthorized"))
        })
    }
}
