use actix_web::{web, Result};
use serde_json::json;
use reqwest::Client;
use async_trait::async_trait;

use crate::apis::models_api::schemas::{EmbeddingRequest, EmbeddingResponse};
use crate::cores::embedding_models::embedding_controller::EmbeddingProvider;
use crate::cores::control::services::ServiceManager;

pub struct Bge;

#[async_trait]
impl EmbeddingProvider for Bge {
    async fn embedding_provider(&self, req_body: web::Json<EmbeddingRequest>) -> Result<EmbeddingResponse, String> {
        // 1. Get the corresponding parameter values from the request
        let input = req_body.input.clone();
        let client = Client::new();

        // 2. Construct the request body for the embedding API
        let request_body = json!({
            "input": input,
            "model": req_body.model
        });

        // 3. Send the POST request
        let service_manager = ServiceManager::default();
        // let service = service_manager.get_service_by_model(&req_body.model).await?;
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

        // 4. Parse the response content into EmbeddingResponse
        if response.status().is_success() {
            let embedding_response: EmbeddingResponse = response.json().await.map_err(|err| format!("Failed to parse response: {}", err))?;
            Ok(embedding_response)
        } else {
            Err(format!("API returned non-success status: {}", response.status()))
        }
    }
}