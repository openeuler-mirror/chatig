use actix_web::{post, web, Error, HttpResponse, Responder};
use actix_web::error::ErrorBadRequest;
use std::sync::Arc;

use crate::apis::models_api::schemas::{EmbeddingRequest, EmbeddingResponse};
use crate::apis::schemas::ErrorResponse;
use crate::cores::embedding_models::embedding_controller::EmbeddingProvider;
use crate::cores::embedding_models::bge::Bge;
use crate::middleware::auth4model::Auth4ModelMiddleware;

// Configure the actix_web service routes.
#[allow(dead_code)]
pub fn configure(cfg: &mut web::ServiceConfig, auth_middleware: Arc<Auth4ModelMiddleware>) {
    cfg.service(
        web::scope("/v1/embeddings") 
            .wrap(auth_middleware)  // 在这个作用域内应用中间件
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

    async fn embedding_provider(&self, req_body: web::Json<EmbeddingRequest>) -> Result<EmbeddingResponse, String> {
        self.model.embedding_provider(req_body).await
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
#[post("/embeddings")]
async fn v1_embeddings(req_body: web::Json<EmbeddingRequest>) -> Result<impl Responder, Error> {
    // 1. Validate the required fields.
    if req_body.input.is_empty() || req_body.model.is_empty() {
        let error_response = ErrorResponse {
            error: "Invalid request: input and model are required fields".into(),
        };
        return Ok(HttpResponse::BadRequest().json(error_response));
    }

    // 2. Call the underlying API and return a unified data format
    let model_name = req_body.model.clone();
    let model_series = model_name.split("/").next().unwrap_or("");
    let model: EMB = match model_series {
        "bge-large-zh-v1.5" => EMB::new(Box::new(Bge {})),
        "bert-large-uncased" => EMB::new(Box::new(Bge {})),
        _ => return Err(ErrorBadRequest(format!("Unsupported {} model series!", model_series))),
    };

    // 3. Send the request to the model service
    let response = model.embedding_provider(req_body).await;
    match response {
        Ok(resp) => Ok(HttpResponse::Ok().json(resp)),
        Err(err) => {
            let error_response = ErrorResponse {
                error: format!("Failed to get response from {} embeddings: {}", model_name, err),
            };
            Ok(HttpResponse::InternalServerError().json(error_response))
        }
    }
}
