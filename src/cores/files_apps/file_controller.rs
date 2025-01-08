use actix_web::{web, Error, HttpResponse};
// use actix_multipart::Multipart;
use async_trait::async_trait;

use crate::apis::models_api::schemas::ChatCompletionRequest;

#[async_trait]
pub trait FileChatController: Send + Sync {
    // async fn upload_temp_docs(&self, payload: Multipart) -> Result<HttpResponse, Error>;
    async fn file_chat_completions(&self, req_body: web::Json<ChatCompletionRequest>) -> Result<HttpResponse, Error>;
}