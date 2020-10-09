use actix_web::web;

use crate::endpoints::{base, heat, category, tournament, result, participation, surfer, lycra_color, heat_advancement};

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
            .route("/participations", web::get().to(participation::get_all))
            .route("/participations/{heat_id}", web::get().to(participation::get_by_heat_id))
            .route("/surfers", web::get().to(surfer::get_all))
            .route("/surfers/{id}", web::get().to(surfer::get_by_id))
            .route("/lycra_colors", web::get().to(lycra_color::get_all))
            .route("/lycra_colors/{id}", web::get().to(lycra_color::get_by_id))
            .route("/advancements", web::get().to(heat_advancement::get_all))
            .route("/advancements/{category_id}", web::get().to(heat_advancement::get_by_category_id))
    );
}
