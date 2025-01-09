use actix_web::{get, post, web, Error, HttpResponse, Responder};
use actix_web::error::ErrorBadRequest;

use crate::apis::models_api::schemas::ChatCompletionRequest;
use crate::apis::schemas::ErrorResponse;

use crate::cores::chat_models::chat_controller::Completions;
use crate::cores::chat_models::qwen::Qwen;
use crate::cores::chat_models::glm::GLM;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(health)
       .service(completions);
}

#[get("/health")]
pub async fn health() -> impl Responder {
    "OK"
}

// define an interface layer that calls the completions method of the large model
struct LLM {
    model: Box<dyn Completions>,
}

impl LLM {
    fn new(model: Box<dyn Completions>) -> Self {
        LLM { model }
    }

    async fn completions(&self, req_body: web::Json<ChatCompletionRequest>) -> Result<HttpResponse, Error> {
        self.model.completions(req_body).await
    }
}

#[post("/v1/chat/completions")]
pub async fn completions(req_body: web::Json<ChatCompletionRequest>) -> Result<impl Responder, Error> {
    // 1. Validate that required fields exist in the request data
    if req_body.model.is_empty() || req_body.messages.is_empty() {
        let error_response = ErrorResponse {
            error: "Invalid request: model or messages cannot be empty.".into(),
        };
        return Ok(HttpResponse::BadRequest().json(error_response));
    }

    // 2. Call the underlying API and return a unified data format
    let model_name = req_body.model.clone();
    let model_series = model_name.split("/").next().unwrap_or("");
    let model: LLM = match model_series {
        "Qwen" => LLM::new(Box::new(Qwen {})),
        "GLM" => LLM::new(Box::new(GLM {})),
        _ => return Err(ErrorBadRequest(format!("Unsupported {} model series!", model_series))),
    };

    // 3. Send the request to the model service
    let response = model.completions(req_body).await;
    match response {
        Ok(resp) => Ok(resp),
        Err(err) => {
            let error_response = ErrorResponse {
                error: format!("Failed to get response from {} chat completions: {}", model_name, err),
            };
            Ok(HttpResponse::InternalServerError().json(error_response))
        }
    }  
}
