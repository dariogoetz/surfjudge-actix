use crate::authentication::AuthenticatedUser;
use crate::logging::LOG;

use actix_web::{dev::Payload, error::ErrorUnauthorized, web, Error, FromRequest, HttpRequest};
use anyhow::Result;
use futures::future::Future;
use oso::{Oso, PolarClass};
use serde::Serialize;
use slog::{error, warn};
use std::{fmt, pin::Pin, sync::Arc, sync::Mutex};

pub struct OsoState {
    oso: Mutex<Oso>,
}

impl OsoState {
    pub fn new(filename: &str) -> Result<Self> {
        let mut oso = Oso::new();
        oso.register_class(
            AuthenticatedUser::get_polar_class_builder()
                .add_method("is_admin", AuthenticatedUser::is_admin)
                .add_method("is_judge", AuthenticatedUser::is_judge)
                .add_method("is_commentator", AuthenticatedUser::is_commentator)
                .build(),
        )?;

        oso.load_file(filename)?;

        Ok(OsoState {
            oso: Mutex::new(oso),
        })
    }

    pub fn is_allowed(
        &self,
        actor: AuthenticatedUser,
        action: &str,
        resource: &str,
    ) -> Result<bool> {
        let is_allowed = self
            .oso
            .lock()
            .unwrap()
            .is_allowed(actor, action, resource)?;

        Ok(is_allowed)
    }
}

#[derive(Serialize, Debug, Default, Clone)]
pub struct AuthorizedUser(pub AuthenticatedUser);

impl fmt::Display for AuthorizedUser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.username)
    }
}

impl FromRequest for AuthorizedUser {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<AuthorizedUser, Error>>>>;

    fn from_request(req: &HttpRequest, pl: &mut Payload) -> Self::Future {
        let oso_state: Option<&web::Data<Arc<OsoState>>> = req.app_data();

        if oso_state.is_none() {
            warn!(LOG, "Oso state is empty!");
            return Box::pin(async { Err(ErrorUnauthorized("unauthorized")) });
        }

        let oso_state = oso_state.unwrap().clone();
        let fut = AuthenticatedUser::from_request(req, pl);
        let method = req.method().to_string();
        let path = req.path().to_string();
        Box::pin(async move {
            let user = fut.await?;
            if let Ok(is_allowed) = oso_state.is_allowed(user.clone(), &method, &path) {
                if is_allowed {
                    return Ok(AuthorizedUser(user));
                } else {
                    warn!(
                        LOG,
                        "Unauthorized access to {:?} by user {:?}!", path, user.username
                    )
                }
            } else {
                error!(
                    LOG,
                    "Error evaluating auth rule for path {:?} and user {:?}", path, user.username
                )
            }
            Err(ErrorUnauthorized("unauthorized"))
        })
    }
}
