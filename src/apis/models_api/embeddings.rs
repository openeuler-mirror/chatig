use actix_web::{get, post, web, Error, HttpResponse, Responder};

use crate::apis::models_api::schemas::{EmbeddingRequest, EmbeddingResponse, EmbeddingData, Usage};
use crate::apis::schemas::ErrorResponse;

// Add some common models. BERT-related embedding models are often used. This is just a simple example.
const SUPPORTED_MODELS: [&str; 4] = [
    "text-embedding-ada-002",
    "bert-base-uncased",
    "bert-large-uncased",
    "roberta-base"
];

// Configure the actix_web service routes.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(embeddings)
     .service(health);
}

// Health check route. When accessing /health, it returns "OK", used to confirm if the service is running normally.
#[get("/health")]
pub async fn health() -> impl Responder {
    "OK"
}

// Handle the POST request for /v1/embeddings.
#[post("/v1/embeddings")]
async fn embeddings(req_body: web::Json<EmbeddingRequest>) -> Result<impl Responder, Error> {
    // Validate the required fields.
    if req_body.input.is_empty() || req_body.model.is_empty() {
        let error_response = ErrorResponse {
            error: "Invalid request: input and model are required fields".into(),
        };
        return Ok(HttpResponse::BadRequest().json(error_response));
    }

    // Check if the model is supported.
    let model_name = req_body.model.clone();
    if!SUPPORTED_MODELS.contains(&model_name.as_str()) {
        let error_response = ErrorResponse {
            error: format!("Unsupported model: {}. Supported models are: {:?}", model_name, SUPPORTED_MODELS),
        };
        return Ok(HttpResponse::BadRequest().json(error_response));
    }

    // Call the corresponding embedding acquisition function according to the model name.
    let response = match model_name.as_str() {
        "text-embedding-ada-002" => get_embedding(req_body, model_name.as_str()).await,
        "bert-base-uncased" => get_embedding(req_body, model_name.as_str()).await,
        _ => Err("Unsupported model".into()),
    };

    match response {
        Ok(resp) => Ok(HttpResponse::Ok().json(resp)),
        Err(err) => {
            let error_response = ErrorResponse {
                error: format!("Failed to get response: {}", err),
            };
            Ok(HttpResponse::InternalServerError().json(error_response))
        }
    }
}

// Simulate the embedding acquisition logic for different models. It needs to be connected to the real API later.
// Currently, it's just a placeholder, generating some empty embedding data structures.
async fn get_embedding(req_body: web::Json<EmbeddingRequest>, model: &str) -> Result<EmbeddingResponse, String> {
    let mut data: Vec<EmbeddingData> = Vec::new();
    for (index, _) in req_body.input.iter().enumerate() {
        data.push(EmbeddingData {
            object: "embedding".to_string(),
            embedding: Vec::new(),
            index,
        });
    }

    Ok(EmbeddingResponse {
        object: "list".to_string(),
        data,
        model: model.to_string(),
        usage: Usage {
            prompt_tokens: 0,
            total_tokens: 0,
            completion_tokens: 0,
        },
    })
}