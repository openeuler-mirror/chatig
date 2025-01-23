use actix_web::web;
use async_trait::async_trait;

use crate::apis::models_api::schemas::{ImageGenerationRequest, ImageGenerationResponse};

#[async_trait]
pub trait ImageProvider: Send + Sync {
    async fn image_provider(&self, req_body: web::Json<ImageGenerationRequest>) -> Result<ImageGenerationResponse, String>;
}