use actix_web::web;
use async_trait::async_trait;

use crate::apis::models_api::schemas::{EmbeddingRequest, EmbeddingResponse};

#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embedding_provider(&self, req_body: web::Json<EmbeddingRequest>) -> Result<EmbeddingResponse, String>;
}

