use crate::logging::LOG;

use actix_identity::Identity;
use actix_web::{dev::Payload, error::ErrorUnauthorized, web, Error, FromRequest, HttpRequest};
use dashmap::DashMap;
use futures::future::Future;
use serde::{Deserialize, Serialize};
use slog::warn;
use std::pin::Pin;

pub type Sessions = DashMap<String, AuthenticatedUser>;

#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticatedUser {
    pub username: String,
    pub roles: Vec<Role>,
}

impl AuthenticatedUser{

    fn has_role(&self, role: &Role) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    pub fn is_admin(&self) -> bool {
        self.has_role(&Role::Admin)
    }

    pub fn is_judge(&self) -> bool {
        self.has_role(&Role::Judge)
    }
    
    pub fn is_commentator(&self) -> bool {
        self.has_role(&Role::Commentator)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Role {
    Admin,
    Judge,
    Commentator,
}

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
                    warn!(LOG, "Logging out user {:?} for which no session data is available!", username);
                    identity.forget();
                }
            };

            Err(ErrorUnauthorized("unauthorized"))
        })
    }
}


#[derive(Serialize, Debug, Default, Clone)]
pub struct AuthenticatedAdmin(AuthenticatedUser);

impl FromRequest for AuthenticatedAdmin {
    type Config = ();
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<AuthenticatedAdmin, Error>>>>;

    fn from_request(req: &HttpRequest, pl: &mut Payload) -> Self::Future {
        let fut = AuthenticatedUser::from_request(req, pl);
        Box::pin(async move {
            let user = fut.await?;
            if user.is_admin() {
                return Ok(AuthenticatedAdmin(user));
            } else {
                warn!(LOG, "Unauthorized: User {:?} is not an admin!", user.username)
            }
            Err(ErrorUnauthorized("unauthorized"))
        })

    }
}


#[derive(Serialize, Debug, Default, Clone)]
pub struct AuthenticatedJudge(AuthenticatedUser); // TODO: get judge id and store here

impl FromRequest for AuthenticatedJudge {
    type Config = ();
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<AuthenticatedJudge, Error>>>>;

    fn from_request(req: &HttpRequest, pl: &mut Payload) -> Self::Future {
        let fut = AuthenticatedUser::from_request(req, pl);
        Box::pin(async move {
            let user = fut.await?;
            if user.is_judge() {
                return Ok(AuthenticatedJudge(user));
            } else {
                warn!(LOG, "Unauthorized: User {:?} is not a judge!", user.username)
            }
            Err(ErrorUnauthorized("unauthorized"))
        })

    }
}


#[derive(Serialize, Debug, Default, Clone)]
pub struct AuthenticatedCommentator(AuthenticatedUser);

impl FromRequest for AuthenticatedCommentator {
    type Config = ();
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<AuthenticatedCommentator, Error>>>>;

    fn from_request(req: &HttpRequest, pl: &mut Payload) -> Self::Future {
        let fut = AuthenticatedUser::from_request(req, pl);
        Box::pin(async move {
            let user = fut.await?;
            if user.is_commentator() {
                return Ok(AuthenticatedCommentator(user));
            } else {
                warn!(LOG, "Unauthorized: User {:?} is not a commentator!", user.username)
            }
            Err(ErrorUnauthorized("unauthorized"))
        })

    }
}
