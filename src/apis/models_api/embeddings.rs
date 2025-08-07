use actix_web::{
    post, web, Error, HttpResponse, Responder, HttpRequest, HttpMessage,
};
use actix_web::error::ErrorBadRequest;
use std::collections::HashMap;

use crate::cores::models::embedding::embedding_controller::{EmbeddingRequest, EmbeddingResponse};
use crate::cores::models::embedding::embedding_controller::EmbeddingProvider;
use crate::cores::models::embedding::support_engines::std_embedding::StdEmbedding;

// Configure the actix_web service routes.
#[allow(dead_code)]
pub fn configure(cfg: &mut web::ServiceConfig,) {
    cfg.service(
        web::scope("/v1/embeddings") 
            .service(v1_embeddings)
    );
}

// define an interface layer that calls the completions method of the large model
struct EMB {
    model: Box<dyn EmbeddingProvider>,
}

impl EMB {
    fn new(model: Box<dyn EmbeddingProvider>) -> Self {
        EMB { model }
    }

    async fn embedding_provider(&self, req_body: web::Json<EmbeddingRequest>, aicpid: String) -> Result<EmbeddingResponse, Error> {
        self.model.embedding_provider(req_body, aicpid).await
    }
}

#[utoipa::path(
    post,  // 请求方法
    path = "/v1/embeddings/embeddings",  // 路径
    request_body = EmbeddingRequest,
    responses(
        (status = 200, body = EmbeddingResponse),
        (status = 400, body = ErrorResponse),
        (status = 500, body = ErrorResponse),
    )  // 响应内容
)]

// Handle the POST request for /v1/embeddings.
#[post("")]
async fn v1_embeddings(req: HttpRequest, req_body: web::Json<EmbeddingRequest>) -> Result<impl Responder, Error> {

    // 1. Validate the required fields.
    if (req_body.input.as_ref().map_or(true, |v| v.is_empty()) 
    && req_body.inputs.as_ref().map_or(true, |s| s.is_empty()))
    || req_body.model.is_empty() {
        let error_response = format!("Invalid embedding request: input and model are required fields");
        return Ok(HttpResponse::BadRequest().json(error_response));
    }
    let aicpid = req.extensions().get::<HashMap<&str, String>>().and_then(|data| data.get("aicpid").cloned()).unwrap_or_else(|| "[]".to_string());

    // 2. Parse the model name and series from the model field (Qwen/Qwen2.5-7B-Instruct)
    let (model_series, model_name) = match req_body.model.split_once('/') {
        Some((series, name)) => (series, name),
        None => ("std", req_body.model.as_str()),
    };

    // 3. Call the underlying API and return a unified data format
    let model: EMB = match model_series {
        "BAAI" | "std" => EMB::new(Box::new(StdEmbedding {active_model: model_name.to_string()})),
        _ => return Err(ErrorBadRequest(format!("Unsupported {} model series!", model_series))),
    };

    // 4. Send the request to the model service
    let response = model.embedding_provider(req_body, aicpid).await;
    match response {
        Ok(resp) => Ok(HttpResponse::Ok().json(resp)),
        Err(err) => {
            let error_response = format!("Failed to get response: {}", err);
            Ok(HttpResponse::InternalServerError().json(error_response))
        }
    }
}
