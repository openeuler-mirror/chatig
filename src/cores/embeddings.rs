use actix_web::{web, Result};
use serde_json::json;
use reqwest::Client;

use crate::apis::models_api::schemas::{EmbeddingRequest, EmbeddingResponse};
use crate::configs::settings::load_server_config;

pub async fn get_embedding(req_body: web::Json<EmbeddingRequest>, model_name: &str) -> Result<EmbeddingResponse, String> {
    // 1. Get the corresponding parameter values from the request
    let input = req_body.input.clone();
    let client = Client::new();

    // 2. Construct the request body for the embedding API
    let server_config = load_server_config().map_err(|err| format!("Failed to load server config: {}", err))?;
    let request_body = json!({
        "input": input,
        "model": model_name
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
        Ok(embedding_response) // Return EmbeddingResponse
    } else {
        Err(format!("API returned non-success status: {}", response.status()))
    }
}