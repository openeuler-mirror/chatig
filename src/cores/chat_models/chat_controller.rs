use actix_web::{web, Error, HttpResponse};
use async_trait::async_trait;

use crate::apis::models_api::schemas::ChatCompletionRequest;

#[async_trait]
pub trait Completions: Send + Sync {
    async fn completions(&self, req_body: web::Json<ChatCompletionRequest>, apikey: String, curl_mode: String) -> Result<HttpResponse, Error>;
}

