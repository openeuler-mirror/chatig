use actix_web::{error, get, post, web, Error, HttpResponse, Responder,HttpRequest};
use std::sync::Arc;

pub fn configure(cfg: &mut web::ServiceConfig, auth4manage: Arc<Auth4ManageMiddleware>) {
    cfg.service(
        web::scope("/v1/public-keys")
            .wrap(auth4manage) // 应用中间件
            .service(get_ras_key)
            .service(encrypt)
    );
}

#[get("/latest")]
async fn get_ras_key(
    req: HttpRequest,
) -> Result<impl Responder, Error> {}

#[post("/encrypt")]
async fn encrypt(
    request: web::Json<EncryptRequest>,
) -> Result<impl Responder, Error> {}

