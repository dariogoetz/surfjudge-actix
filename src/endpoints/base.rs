use actix_web::{web, Responder, HttpResponse};

use crate::database::Pool;
use crate::models::heat;

pub async fn test_endpoint(db: web::Data<Pool>) -> impl Responder {
    let result = heat::Heat::find_all(db.get_ref()).await.unwrap();
    HttpResponse::Ok().body("Hello world!")
}
