use crate::templates::{Context, Templates};

use actix_web::{web, HttpResponse, Responder};

pub fn get_default_template_context() -> Context {
    let mut ctx = Context::new();
    ctx.insert("description", &"Surfjudge - actix");

    ctx
}

pub async fn index(templates: web::Data<Templates>) -> impl Responder {
    let ctx = get_default_template_context();

    let rendered = templates.render("index.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}
