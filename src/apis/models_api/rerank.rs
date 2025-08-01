use actix_web::{
    post, web, Error, HttpResponse, Responder, HttpRequest, HttpMessage,
};
use std::sync::Arc;
use std::collections::HashMap;

pub fn configure(cfg: &mut web::ServiceConfig, auth4model: Arc<Auth4ModelMiddleware>) {
    cfg.service(web::scope("/v1/rerank")
        .wrap(auth4model)
        .service(rerank_std)
        .service(rerank_vllm)
        .service(rerank_llamabox)
    );
}

#[post("")]
pub async fn rerank_std(req: HttpRequest, req_body: web::Json<StdRerankRequest>) -> Result<impl Responder, Error> {}

#[post("/llamabox")]
pub async fn rerank_llamabox(req: HttpRequest, req_body: web::Json<LlamaBoxRerankRequest>) -> Result<impl Responder, Error> {}

#[post("/vllm")]
pub async fn rerank_vllm(req: HttpRequest, req_body: web::Json<VLLMRerankRequest>) -> Result<impl Responder, Error> {}

