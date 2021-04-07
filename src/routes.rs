use crate::configuration::CONFIG;
use crate::endpoints::{
    auth, category, heat, heat_advancement, heat_state, judge, lycra_color, pages, participation,
    result, score, surfer, tournament,
};

use actix_files as fs;
use actix_web::web;

pub fn configure_apis(cfg: &mut web::ServiceConfig) {
    if CONFIG.api.public_path.is_some() {
        public_api_routes(cfg);
    };
    let mut require_auth = false;
    if CONFIG.api.admin_path.is_some() {
        admin_api_routes(cfg);
        require_auth = true;
    };
    if CONFIG.api.judging_path.is_some() {
        judging_api_routes(cfg);
        require_auth = true;
    };
    if require_auth {
        auth_api_routes(cfg);
    };

    static_routes(cfg);
    // page routes need to come last due to the "" scope

    page_routes(cfg);
}

pub fn public_api_routes(cfg: &mut web::ServiceConfig) {
    // public rest API endpoints
    cfg.service(
        web::scope(&CONFIG.api.public_path.as_ref().unwrap())
            .route("/heats", web::get().to(heat::get_all))
            .route("/heats/{id}", web::get().to(heat::get_by_id))
            .route("/heats/{id}/results", web::get().to(result::get_by_heat_id))
            .route(
                "/heats/{id}/participations",
                web::get().to(participation::get_by_heat_id),
            )
            .route(
                "/heats/{heat_id}/state",
                web::get().to(heat_state::get_by_heat_id),
            )
            .route("/active_heats", web::get().to(heat::get_active_heats))
            .route("/categories", web::get().to(category::get_all))
            .route("/categories/{id}", web::get().to(category::get_by_id))
            .route(
                "/categories/{id}/heats",
                web::get().to(heat::get_by_category_id),
            )
            .route(
                "/categories/{id}/advancements",
                web::get().to(heat_advancement::get_by_category_id),
            )
            .route(
                "/categories/{id}/results",
                web::get().to(result::get_by_category_id),
            )
            .route(
                "/categories/{id}/participations",
                web::get().to(participation::get_by_category_id),
            )
            .route(
                "/categories/{id}/active_heats",
                web::get().to(heat::get_active_heats_by_category_id),
            )
            .route("/tournaments", web::get().to(tournament::get_all))
            .route("/tournaments/{id}", web::get().to(tournament::get_by_id))
            .route(
                "/tournaments/{id}/categories",
                web::get().to(category::get_by_tournament_id),
            )
            .route(
                "/tournaments/{id}/active_heats",
                web::get().to(heat::get_active_heats_by_tournament_id),
            )
            .route("/results", web::get().to(result::get_all))
            .route("/results/{heat_id}", web::get().to(result::get_by_heat_id))
            .route("/participations", web::get().to(participation::get_all))
            .route(
                "/participations/{heat_id}",
                web::get().to(participation::get_by_heat_id),
            )
            .route("/surfers", web::get().to(surfer::get_all))
            .route("/surfers/{id}", web::get().to(surfer::get_by_id))
            .route("/lycra_colors", web::get().to(lycra_color::get_all))
            .route("/lycra_colors/{id}", web::get().to(lycra_color::get_by_id))
            .route("/advancements", web::get().to(heat_advancement::get_all))
            .route(
                "/advancements/{category_id}",
                web::get().to(heat_advancement::get_by_category_id),
            ),
    );
}

pub fn auth_api_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope(&CONFIG.api.auth_path.as_ref().unwrap())
            .route("/me", web::get().to(auth::me))
            .route("/login", web::post().to(auth::login))
            .route("/logout", web::post().to(auth::logout)),
    );
}

pub fn admin_api_routes(cfg: &mut web::ServiceConfig) {
    // public rest API endpoints
    cfg.service(
        web::scope(&CONFIG.api.admin_path.as_ref().unwrap())
            .route(
                "/heats/{heat_id}/start",
                web::post().to(heat_state::start_heat),
            )
            .route(
                "/heats/{heat_id}/stop",
                web::post().to(heat_state::stop_heat),
            )
            .route(
                "/heats/{heat_id}/toggle_pause",
                web::post().to(heat_state::toggle_heat_pause),
            )
            .route(
                "/heats/{heat_id}/reset_heat_time",
                web::post().to(heat_state::reset_heat_time),
            )
            .route("/judges", web::get().to(judge::get_all))
            .route(
                "/heats/{heat_id}/assigned_judges",
                web::get().to(judge::get_assigned_judges_for_heat),
            )
            .route("/judging_requests", web::get().to(judge::get_requests)),
    );
}

pub fn judging_api_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope(&CONFIG.api.judging_path.as_ref().unwrap())
            .route(
                "/active_judge_assignments",
                web::get().to(judge::get_assigned_active_heats_for_judge),
            )
            .route(
                "/heats/{heat_id}/judges/{judge_id}/scores",
                web::get().to(score::get_by_heat_id_and_judge_id),
            )
            .route(
                "/heats/{heat_id}/scores",
                web::get().to(score::get_by_heat_id),
            )
            .route("/scores", web::put().to(score::put))
            .route("/scores/{heat_id}/{judge_id}/{surfer_id}/{wave}", web::delete().to(score::delete))
            .route("/judging_requests", web::post().to(judge::add_request)),
    );
}

pub fn page_routes(cfg: &mut web::ServiceConfig) {
    // web page endpoints
    cfg.service(web::scope("").route("/", web::get().to(pages::index)));
}

pub fn static_routes(cfg: &mut web::ServiceConfig) {
    // serve static files
    cfg.service(fs::Files::new("/static", "./static").show_files_listing());
}
