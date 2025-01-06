use actix_web::{web, Error, HttpResponse};
use crate::apis::models_api::schemas::ChatCompletionRequest;

#[async_trait::async_trait]
pub trait Completions: Send + Sync {
    async fn completions(&self, req_body: web::Json<ChatCompletionRequest>) -> Result<HttpResponse, Error>;
}

