use actix_web::{web, Result};
use serde_json::json;
use reqwest::Client;
use async_trait::async_trait;

use crate::apis::models_api::schemas::{ImageGenerationRequest, ImageGenerationResponse};
use crate::cores::image_models::image_controller::ImageProvider;
use crate::cores::control::services::ServiceManager;

pub struct SdxlTurbo;

#[async_trait]
impl ImageProvider for SdxlTurbo {
    async fn image_provider(&self, req_body: web::Json<ImageGenerationRequest>) -> Result<ImageGenerationResponse, String> {
        // 1. Get the corresponding parameter values from the request
        let prompt = req_body.prompt.clone();
        let client = Client::new();

        // 2. Construct the request body for the image generation API
        let request_body = json!({
            "model": req_body.model,
            "prompt": prompt
        });

        // 3. Send the POST request
        let service_manager = ServiceManager::default();
        let service = service_manager.get_service_by_model(&req_body.model)
            .await
            .map_err(|err| format!("Failed to get service by model: {}", err))?;
        let service = match service {
            Some(service) => service,
            None => return Err(format!("{} model is not supported", req_body.model)),
        };
        let response = match client.post(service.url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await {
                Ok(resp) => resp,
                Err(err) => return Err(format!("Request failed: {}", err)),
            };

        // 4. Parse the response content into ImageGenerationResponse
        if response.status().is_success() {
            let image_response: ImageGenerationResponse = response.json().await.map_err(|err| format!("Failed to parse response: {}", err))?;
            Ok(image_response)
        } else {
            Err(format!("API returned non-success status: {}", response.status()))
        }
    }
}