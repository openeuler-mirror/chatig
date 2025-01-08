use actix_web::{post, web, Error, HttpResponse};
use actix_web::error::ErrorBadRequest;

use crate::apis::models_api::schemas::ChatCompletionRequest;
use crate::apis::schemas::ErrorResponse;

use crate::cores::rag_apps;
use crate::cores::rag_apps::rag_controller::RAGController;


pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(rag_chat_completions);
}

// define an interface layer that calls the completions method of the large model
struct RAG {
    rag: Box<dyn RAGController>,
}

impl RAG {
    fn new(rag: Box<dyn RAGController>) -> Self {
        RAG { rag }
    }
    async fn rag_chat_completions(&self, req_body: web::Json<ChatCompletionRequest>) -> Result<HttpResponse, Error>{
        self.rag.rag_chat_completions(req_body).await
    }
}

#[post("/v1/rag/completions")]
pub async fn rag_chat_completions(req_body: web::Json<ChatCompletionRequest>) -> Result<HttpResponse, Error> {
    // 1. Validate that required fields exist in the request data
    if req_body.model.is_empty() || req_body.messages.is_empty() {
        let error_response = ErrorResponse {
            error: "Invalid request: model or messages cannot be empty.".into(),
        };
        return Ok(HttpResponse::BadRequest().json(error_response));
    }

    // 2. Call the underlying API and return a unified data format
    let model_name = req_body.model.clone();
    let rag :RAG = match model_name.as_str() {
        "chatchat" => RAG::new(Box::new(rag_apps::chatchat::ChatChatRAG {})),
        "Copilot" => RAG::new(Box::new(rag_apps::copilot::CopilotRAG {})),
        _ => return Err(ErrorBadRequest(format!("Unsupported model {}!", model_name))),
    };
        
    // 3. Construct the response body based on the API's return result
    let response = rag.rag_chat_completions(req_body).await;
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