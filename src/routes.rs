use actix_web::web;

use crate::endpoints::{base, heat};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/rest")
            .route("/", web::get().to(base::test_endpoint))
            .route("/heats", web::get().to(heat::get_all_heats))
            .route("/heats/{id}", web::get().to(heat::get_heat_by_id))
    );
}
