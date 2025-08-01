use actix_web::{post, web, HttpResponse, Responder, get, delete, put, HttpRequest};
use std::sync::Arc;
use crate::meta::connection::{DbConnection, get_db_connection};

pub fn configure(cfg: &mut web::ServiceConfig, auth4manage: Arc<Auth4ManageMiddleware>) {
    cfg.service(
        web::scope("/v1/auth")
            .wrap(auth4manage) // 应用中间件
            .service(invalidate_cache)
            .service(create_userkeys_model)
            .service(delete_userkeys_model)
            .service(get_userkeys_model)
            .service(get_all_userkeys_models)
            .service(sync_user_limits),
    );
}

#[post("/invalidate_cache")]
async fn invalidate_cache(
    request: web::Json<InvalidateCacheRequest>,
) -> impl Responder {}

#[post("/userkeys_models")]
async fn create_userkeys_model(
    request: web::Json<UserKeysModelRequest>,
    req: HttpRequest,
) -> impl Responder {}

#[delete("/userkeys_models/{id}")]
async fn delete_userkeys_model(
    path: web::Path<i32>,
    req: HttpRequest,
) -> impl Responder {}

#[put("/userkeys_models/{id}")]
async fn update_userkeys_model(
    path: web::Path<i32>,
    request: web::Json<UserKeysModelUpdateRequest>,
    req: HttpRequest,
) -> impl Responder {}

#[get("/userkeys_models/{id}")]
async fn get_userkeys_model(
    path: web::Path<i32>,
    req: HttpRequest,
) -> impl Responder {}

#[get("/userkeys_models")]
async fn get_all_userkeys_models(req: HttpRequest) -> impl Responder {}

#[post("/sync_user_limits")]
async fn sync_user_limits(req: HttpRequest) -> impl Responder {}
