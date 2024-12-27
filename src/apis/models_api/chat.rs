use actix_web::{get, post, web, Error, HttpResponse, Responder};

use crate::apis::models_api::schemas::ChatCompletionRequest;
use crate::apis::schemas::ErrorResponse;
use crate::cores::funs::{chatchat, copilot};
use crate::cores::models::chat_completions;

// Define supported models
const SUPPORTED_MODELS: [&str; 4] = ["chatchat", "copilot", "vllm", "mindie"];

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(health)
       .service(rag_chat_completions)
       .service(completions);
}

#[get("/health")]
pub async fn health() -> impl Responder {
    "OK"
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

    // 2. Check if the model is supported
    let model_name = req_body.model.clone();
    if !SUPPORTED_MODELS.contains(&model_name.as_str()) {
        let error_response = ErrorResponse {
            error: format!("Unsupported model: {}. Supported models are: {:?}", model_name, SUPPORTED_MODELS),
        };
        return Ok(HttpResponse::BadRequest().json(error_response));
    }

    // 3. Call the underlying API and return a unified data format
    if let Some(_file_id) = &req_body.file_id {
        let response = match model_name.as_str() {
            "chatchat" => chatchat::file_chat(req_body).await,
            _ => Err("Unsupported model".into()),
        };
            
        match response {
            Ok(resp) => {
                Ok(resp)
            }
            Err(err) => {
                let error_response = ErrorResponse {
                    error: format!("Failed to get response from kb_chat: {}", err),
                };
                Ok(HttpResponse::InternalServerError().json(error_response))
            }
        }
    } else {
        let response = chat_completions::completions(req_body).await;

        match response {
            Ok(resp) => Ok(resp),
            Err(err) => {
                let error_response = ErrorResponse { error: format!("Failed to get response from chat completions: {}", err), };
                Ok(HttpResponse::InternalServerError().json(error_response))
            }
        }
    }
}

#[post("/v1/rag/completions")]
pub async fn rag_chat_completions(req_body: web::Json<ChatCompletionRequest>) -> Result<impl Responder, Error> {
    // 1. Validate that required fields exist in the request data
    if req_body.model.is_empty() || req_body.messages.is_empty() {
        let error_response = ErrorResponse {
            error: "Invalid request: model or messages cannot be empty.".into(),
        };
        return Ok(HttpResponse::BadRequest().json(error_response));
    }

    // 2. Check if the model is supported
    let model_name = req_body.model.clone();
    if !SUPPORTED_MODELS.contains(&model_name.as_str()) {
        let error_response = ErrorResponse {
            error: format!("Unsupported model: {}. Supported models are: {:?}", model_name, SUPPORTED_MODELS),
        };
        return Ok(HttpResponse::BadRequest().json(error_response));
    }

    // 3. Call the underlying API and return a unified data format
    let response = match model_name.as_str() {
        "chatchat" => chatchat::kb_chat(req_body).await,
        "copilot" => copilot::get_answer(req_body).await,
        _ => Err("Unsupported model".into()),
    };
        
    // 3. Construct the response body based on the API's return result
    match response {
        Ok(resp) => {
            Ok(resp)
        }
        Err(err) => {
            let error_response = ErrorResponse { error: format!("Failed to get response from kb_chat: {}", err), };
            Ok(HttpResponse::InternalServerError().json(error_response))
        }
    }
}