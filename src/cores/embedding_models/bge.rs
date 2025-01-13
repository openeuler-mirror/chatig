use actix_web::{web, Result};
use serde_json::json;
use reqwest::Client;
use async_trait::async_trait;

use crate::apis::models_api::schemas::{EmbeddingRequest, EmbeddingResponse};
use crate::configs::settings::load_server_config;
use crate::cores::embedding_models::embedding_controller::EmbeddingProvider;

pub struct Bge;

#[async_trait]
impl EmbeddingProvider for Bge {
    async fn embedding_provider(&self, req_body: web::Json<EmbeddingRequest>) -> Result<EmbeddingResponse, String> {
        // 1. Get the corresponding parameter values from the request
        let input = req_body.input.clone();
        let client = Client::new();

        // 2. Construct the request body for the embedding API
        let server_config = load_server_config().map_err(|err| format!("Failed to load server config: {}", err))?;
        let request_body = json!({
            "input": input,
            "model": req_body.model
        });

        // 3. Send the POST request
        let response = match client.post(&server_config.embeddings.get_embedding)
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