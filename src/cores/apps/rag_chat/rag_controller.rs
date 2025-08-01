use actix_web::{web, Error, HttpResponse};
use async_trait::async_trait;
use crate::cores::chat_models::chat_controller::ChatCompletionRequest;

#[async_trait]
pub trait RAGController: Send + Sync {
    async fn rag_chat_completions(&self, req_body: web::Json<ChatCompletionRequest>) -> Result<HttpResponse, Error>;
}