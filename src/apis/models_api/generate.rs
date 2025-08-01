use actix_web::{
    post, web, Error, HttpResponse, Responder, HttpRequest, HttpMessage,
};
use actix_web::error::ErrorBadRequest;
use log::{info, error};
use std::sync::Arc;
use std::collections::HashMap;

pub fn configure(
    cfg: &mut web::ServiceConfig,
    auth4model: Arc<Auth4ModelMiddleware>,
    qos: Arc<Qos>,
    monitor_metrics: Arc<MetricsMiddleware>,
) {
    cfg.service(
        web::scope("/v1/tgi")
            .wrap(monitor_metrics)
            .wrap(qos)
            .wrap(auth4model)   // 这里中间件顺序不能改
            .service(generate)
            .service(generate_stream)
    );  
}

#[post("/generate")]
pub async fn generate(
    req: HttpRequest,
    req_body: web::Json<GenerateRequest>,
) -> Result<impl Responder, Error> {}

#[post("/generate_stream")]
pub async fn generate_stream(
    req: HttpRequest,
    req_body: web::Json<GenerateRequest>,
) -> Result<impl Responder, Error> {}

