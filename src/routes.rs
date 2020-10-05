use actix_web::web;

use crate::endpoints::{base, heat, category, tournament, result};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/rest")
            .route("/", web::get().to(base::test_endpoint))
            .route("/heats", web::get().to(heat::get_all))
            .route("/heats/{id}", web::get().to(heat::get_by_id))
            .route("/categories", web::get().to(category::get_all))
            .route("/categories/{id}", web::get().to(category::get_by_id))
            .route("/tournaments", web::get().to(tournament::get_all))
            .route("/tournaments/{id}", web::get().to(tournament::get_by_id))
            .route("/results", web::get().to(result::get_all))
            .route("/results/{heat_id}", web::get().to(result::get_by_heat_id))
    );
}
