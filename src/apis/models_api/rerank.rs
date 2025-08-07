use actix_web::{
    post, web, Error, HttpResponse, Responder, HttpRequest, HttpMessage,
};
use std::collections::HashMap;

use crate::cores::models::rerank::rerank_controller::{LlamaBoxRerankRequest, LlamaBoxRerankTrait, 
    StdRerankRequest, StdRerankTrait, VLLMRerankRequest, VLLMRerankTrait};
use crate::cores::models::rerank::support_engines::{vllm_rerank, llamabox_rerank, std_rerank};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/v1/rerank")
        .service(rerank_std)
        .service(rerank_vllm)
        .service(rerank_llamabox)
    );
}

#[post("")]
pub async fn rerank_std(req: HttpRequest, req_body: web::Json<StdRerankRequest>) -> Result<impl Responder, Error> {
    // 1. Validate that required fields exist in the request data
    if req_body.model.is_empty() || req_body.query.is_empty() || req_body.documents.is_empty() {
        let error_response = format!("Invalid request: model, query or documents cannot be empty.");
        return Ok(HttpResponse::BadRequest().json(error_response));
    }
    let aicpid = req.extensions().get::<HashMap<&str, String>>().and_then(|data| data.get("aicpid").cloned()).unwrap_or_else(|| "[]".to_string());

    // 2. build the rerank model
    let (_, active_model) = match req_body.model.split_once('/') {
        Some((series, name)) => (series, name),
        None => ("std", req_body.model.as_str()),
    };
    
    let rerank_model = std_rerank::StdRerank { active_model: active_model.to_string() };

    // 3. Send the request to the model
    let response = rerank_model.rerank(req_body, aicpid).await;

    response
}


#[post("/llamabox")]
pub async fn rerank_llamabox(req: HttpRequest, req_body: web::Json<LlamaBoxRerankRequest>) -> Result<impl Responder, Error> {
    // 1. Validate that required fields exist in the request data
    if req_body.model.is_empty() || req_body.query.is_empty() || req_body.documents.is_empty() {
        let error_response = format!("Invalid request: model, query or documents cannot be empty.");
        return Ok(HttpResponse::BadRequest().json(error_response));
    }
    let aicpid = req.extensions().get::<HashMap<&str, String>>().and_then(|data| data.get("aicpid").cloned()).unwrap_or_else(|| "[]".to_string());

    // 2. build the rerank model
    let (_, model_name) = match req_body.model.split_once('/') {
        Some((series, name)) => (series, name),
        None => ("std", req_body.model.as_str()),
    };

    let rerank_model = llamabox_rerank::LlamaBoxRerank { active_model: model_name.to_string() };

    // 3. Send the request to the model
    let response = rerank_model.rerank(req_body, aicpid).await;

    response
}


#[post("/vllm")]
pub async fn rerank_vllm(req: HttpRequest, req_body: web::Json<VLLMRerankRequest>) -> Result<impl Responder, Error> {
    // 1. Validate that required fields exist in the request data
    if req_body.model.is_empty() || req_body.query.is_empty() || req_body.documents.is_empty() {
        let error_response = format!("Invalid request: model, query or documents cannot be empty.");
        return Ok(HttpResponse::BadRequest().json(error_response));
    }
    let aicpid = req.extensions().get::<HashMap<&str, String>>().and_then(|data| data.get("aicpid").cloned()).unwrap_or_else(|| "[]".to_string());

    // 2. build the rerank model
    let (_, model_name) = match req_body.model.split_once('/') {
        Some((series, name)) => (series, name),
        None => ("std", req_body.model.as_str()),
    };
    let rerank_model = vllm_rerank::VLLMRerank { active_model: model_name.to_string() };

    // 3. Send the request to the model
    let response = rerank_model.rerank(req_body, aicpid).await;

    response
}