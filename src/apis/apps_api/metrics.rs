use actix_web::web;
use crate::utils::metrics;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/metrics")
            .route(web::get().to(|| async { metrics::metrics_handler().await }))
    );
}