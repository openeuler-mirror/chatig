use actix_web::{delete, get, post, put, web, Error, HttpResponse, Responder};
use actix_web::error::ErrorInternalServerError;
use std::sync::Arc;
use serde::Deserialize;

pub fn configure(cfg: &mut web::ServiceConfig, auth_middleware: Arc<Auth4ManageMiddleware>) {
    cfg.service(
        web::scope("/v1/user/limits")
            .wrap(auth_middleware)
            .service(create_user_model_limits)
            .service(delete_user_model_limits)
            .service(update_user_model_limits)
            .service(get_all_user_model_limits)
            .service(get_user_model_limits),
    );
}

#[post("")]
pub async fn create_user_model_limits(
    user_limits: web::Json<UserLimits>,
) -> Result<impl Responder, Error> {}

#[delete("")]
async fn delete_user_model_limits(
    body: web::Json<DeleteRequestBody>,
) -> Result<impl Responder, Error> {}

#[put("")]
async fn update_user_model_limits(
    user_limits: web::Json<UserLimits>,
) -> Result<impl Responder, Error> {}

#[get("")]
async fn get_all_user_model_limits() -> Result<impl Responder, Error> {}

#[post("/get")]
async fn get_user_model_limits(
    body: web::Json<RequestBody>,
) -> Result<impl Responder, Error> {}

