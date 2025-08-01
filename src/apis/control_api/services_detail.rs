use actix_web::{delete, error, get, post, put, web, Error, HttpResponse, Responder};
use std::sync::Arc;
use serde::{Serialize, Deserialize};

pub fn configure(cfg: &mut web::ServiceConfig, auth4manage: Arc<Auth4ManageMiddleware>) {
    cfg.service(
        web::scope("/v1/services_detail")
            .wrap(auth4manage) // 应用中间件
            .service(create_services_detail)
            .service(delete_service_detail)
            .service(update_service_detail)
            .service(get_service_detail)
            .service(get_all_service_details)
    );
}

#[post("")]
async fn create_services_detail(
    service_derail: web::Json<ServicesDetail>,
) -> Result<impl Responder, Error> {}

#[delete("/{service_id}")]
async fn delete_service_detail(
    service_id: web::Path<String>,
) -> Result<impl Responder, Error> {}

#[put("/{service_id}")]
async fn update_service_detail(
    service_id: web::Path<String>,
    update_service_detail: web::Json<UpdateServiceDetail>,
) -> Result<impl Responder, Error> {}

#[get("/{service_id}")]
async fn get_service_detail(
    service_id: web::Path<String>,
) -> Result<impl Responder, Error> {}

#[get("")]
async fn get_all_service_details() -> Result<impl Responder, Error> {}
