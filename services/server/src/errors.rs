use actix_web::{HttpResponse};
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use common::errors::NavajoError;

pub fn error_response(error: NavajoError) -> HttpResponse<BoxBody> {
    HttpResponse::build(StatusCode::BAD_REQUEST)
        .body(error.to_string())
}