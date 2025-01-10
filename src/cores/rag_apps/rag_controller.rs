use actix_web::{web, Error, HttpResponse};
use async_trait::async_trait;
use crate::apis::models_api::schemas::ChatCompletionRequest;

#[async_trait]
pub trait RAGController: Send + Sync {
    async fn rag_chat_completions(&self, req_body: web::Json<ChatCompletionRequest>) -> Result<HttpResponse, Error>;
}