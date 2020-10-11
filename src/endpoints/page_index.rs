use crate::templates::{Templates, Context};
use crate::configuration::CONFIG;

use actix_web::{error, web, Responder, HttpResponse};

pub async fn index(templates: web::Data<Templates>) -> impl Responder {
    let mut ctx = Context::new();
    ctx.insert("global_is_judge", &false);
    ctx.insert("global_is_admin", &false);
    ctx.insert("global_is_commentator", &false);
    ctx.insert("global_is_headjudge", &false);
    ctx.insert("global_logged_in", &false);
    ctx.insert("message", &"Hello world!");
    ctx.insert("description", &"Some description");
    ctx.insert("websocket_url", &CONFIG.websocket_url);

    let rendered = templates.render("live_results.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}
