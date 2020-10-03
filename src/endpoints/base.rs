use actix_web::{Responder, HttpResponse};

pub async fn test_endpoint() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}
