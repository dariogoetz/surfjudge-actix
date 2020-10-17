use crate::templates::{Templates, Context};
use crate::configuration::CONFIG;

use actix_web::{error, web, Responder, HttpResponse};


pub fn get_default_template_context() -> Context {
    let mut ctx = Context::new();
    ctx.insert("global_is_judge", &false);
    ctx.insert("global_is_admin", &false);
    ctx.insert("global_is_commentator", &false);
    ctx.insert("global_is_headjudge", &false);
    ctx.insert("global_logged_in", &false);
    ctx.insert("description", &"Surfjudge - actix");
    ctx.insert("websocket_url", &CONFIG.websocket_url);

    ctx
}


pub async fn live_results(templates: web::Data<Templates>) -> impl Responder {
    let ctx = get_default_template_context();

    let rendered = templates.render("live_results.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

pub async fn results(templates: web::Data<Templates>) -> impl Responder {
    let ctx = get_default_template_context();

    let rendered = templates.render("results.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

pub async fn heatcharts(templates: web::Data<Templates>) -> impl Responder {
    let ctx = get_default_template_context();

    let rendered = templates.render("heatcharts.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}