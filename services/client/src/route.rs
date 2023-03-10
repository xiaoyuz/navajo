use actix_web::{get, HttpResponse, post, Responder, web};
use serde::Deserialize;
use crate::errors::error_response;
use crate::WebServer;

#[derive(Deserialize)]
struct Info {
    to: String,
}

pub fn device_scope_cfg(cfg: &mut web::ServiceConfig) {
    cfg
        .service(register)
        .service(login)
        .service(logout)
        .service(create_session)
        .service(testchat);
}

#[get("/register")]
async fn register(data: web::Data<WebServer>) -> impl Responder {
    let account = data.register().await.unwrap();
    HttpResponse::Ok().json(account)
}

#[post("/login")]
async fn login(data: web::Data<WebServer>, body: String) -> impl Responder {
    data.login(&body).await.map_or_else(
        error_response,
        |res| HttpResponse::Ok().json(res)
    )
}

#[get("/logout")]
async fn logout(data: web::Data<WebServer>) -> impl Responder {
    data.logout().await;
    HttpResponse::Ok().body(())
}

#[get("/create_session")]
async fn create_session(data: web::Data<WebServer>) -> impl Responder {
    data.create_session().await.map_or_else(
        error_response,
        |res| HttpResponse::Ok().json(res)
    )
}

#[get("/testchat")]
async fn testchat(data: web::Data<WebServer>, info: web::Query<Info>) -> impl Responder {
    data.test_p2p(&info.to).await.unwrap();
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}