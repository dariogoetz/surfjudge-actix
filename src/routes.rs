use crate::configuration::CONFIG;
use crate::endpoints::{
    auth, category, config, heat, heat_advancement, heat_state, lycra_color, pages, participation,
    result, surfer, tournament,
};

use actix_files as fs;
use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    // serve static files
    cfg.service(fs::Files::new("/static", "./static").show_files_listing());

    // rest API endpoints
    cfg.service(
        web::scope(&CONFIG.ui_settings.api_path)
            .route("/config", web::get().to(config::get_ui_config))
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
            .route(
                "/heats/{heat_id}/remaining_time",
                web::get().to(heat_state::get_remaining_heat_time),
            )
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
            .route("/active_heats", web::get().to(heat::get_active_heats))
            .route(
                "/active_heats/{tournament_id}",
                web::get().to(heat::get_active_heats_by_tournament_id),
            )
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
            )
            .route("/auth/session_test", web::get().to(auth::session_test))
            .route("/auth/protected", web::get().to(auth::protected))
            .route("/auth/login", web::post().to(auth::login))
            .route("/auth/logout", web::post().to(auth::logout)),
    );

    // web page endpoints
    cfg.service(web::scope("").route("/", web::get().to(pages::index)));
}
