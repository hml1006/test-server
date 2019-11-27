use actix_web::get;
use actix_web::{HttpResponse, Responder};

#[get("/")]
pub fn index() -> impl Responder {
    HttpResponse::Ok().body("root")
}

#[get("/ping")]
pub fn ping() -> impl Responder {
    HttpResponse::Ok().body("pong")
}

#[get("/check")]
pub fn check() -> impl Responder {
    HttpResponse::Ok().body("ok")
}
