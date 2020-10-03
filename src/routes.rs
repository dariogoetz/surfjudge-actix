use actix_web::web;

use crate::endpoints::base;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg
        .service(
            web::scope("")
                .route("/", web::get().to(base::test_endpoint))
        );
}
