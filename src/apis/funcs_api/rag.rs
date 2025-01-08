use actix_web::{get, post, web, Error, HttpResponse, Responder};

use crate::apis::models_api::schemas::ChatCompletionRequest;
use crate::apis::schemas::ErrorResponse;
use crate::cores::{chatchat, copilot};
use crate::cores::chat_completions;


pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(rag_chat_completions)
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