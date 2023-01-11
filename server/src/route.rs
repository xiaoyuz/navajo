use actix_web::{HttpResponse, post, Responder, web};
use common::beans::{ApiResponse, DeviceInfoRequest};
use crate::errors::error_response;
use crate::Server;

pub fn device_scope_cfg(cfg: &mut web::ServiceConfig) {
    cfg.service(create_session);
}

#[post("/create_session")]
async fn create_session(data: web::Data<Server>, body: web::Json<DeviceInfoRequest>) -> impl Responder {
    let request = body.0;
    data.create_session(&request).await.map_or_else(
        |e| error_response(e),
        |res|{
            let response = ApiResponse::success(res);
            HttpResponse::Ok().json(response)
        }
    )
}