use crate::models::{ErrorResponse, Message};
use actix_web::HttpResponse;

#[inline]
pub fn get_error(error: &'static str) -> HttpResponse {
    HttpResponse::InternalServerError().json(ErrorResponse { error: error })
}

#[inline]
pub fn get_fetch_users_error() -> HttpResponse {
    get_error("user_get_fail")
}

#[inline]
pub fn get_registration_error() -> HttpResponse {
    HttpResponse::Unauthorized().json(ErrorResponse { error: "reg_fail" })
}

#[inline]
pub fn get_auth_failed_error() -> HttpResponse {
    HttpResponse::Unauthorized().json(ErrorResponse { error: "auth_fail" })
}

#[inline]
pub fn get_success_with_message(message: &'static str) -> HttpResponse {
    HttpResponse::Ok().json(Message { message: message })
}

#[inline]
pub fn get_success() -> HttpResponse {
    get_success_with_message("Success")
}
