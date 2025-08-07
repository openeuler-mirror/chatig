use actix_web::error::{ErrorInternalServerError, ErrorBadRequest};
use actix_web::{web, Result, Error};
use async_trait::async_trait;

use crate::cores::models::embedding::embedding_controller::{EmbeddingProvider, EmbeddingRequest, EmbeddingResponse};
use crate::cores::control::services::ServiceManager;
use crate::cores::models::embedding::embedding_utils::{build_embedding_request, get_embedding_response};

pub struct StdEmbedding {
    pub active_model: String,
}

#[async_trait]
impl EmbeddingProvider for StdEmbedding {
    async fn embedding_provider(&self, req_body: web::Json<EmbeddingRequest>, aicpid: String) -> Result<EmbeddingResponse, Error> {
        // 1. Read the model's parameter configuration
        let req_model_name = req_body.model.clone();

        let service_manager = ServiceManager::default();
        let service_option = service_manager.get_service_by_model(&self.active_model, None, aicpid).await?;
        let service = match service_option {
            Some(svc) => svc,
            None => return Err(ErrorBadRequest(format!("{} model is not supported", req_model_name))),
        };
        // println!("Service: {:?}", service);

        // 2. Build the request body
        let request_body = build_embedding_request(&service.model_name, &req_body);
        // println!("Request Body: {:?}", request_body);

        // 3. Parse the response content into EmbeddingResponse
        // let (response, _) = get_embedding_response(request_body, service.url).await.map_err(|err|ErrorInternalServerError(format!("{}", err)))?;

        let (embedding_response, _) = get_embedding_response(request_body, service.url, service.api_key)
        .await
        .map_err(|err| {
            ErrorInternalServerError(format!("API request failed: {}", err))
        })?;

        Ok(embedding_response)
        // if response.status().is_success() {
        //     let embedding_response: EmbeddingResponse = response.json().await
        //     .map_err(|err| {
        //         ErrorInternalServerError(format!("Failed to parse response: {}", err))
        //     })?;
        //     // .map_err(|err| format!("Failed to parse response: {}", err))?;
        //     Ok(embedding_response)
        // } else {
        //     // Err(format!("API returned non-success status: {}", response.status()))
        //     Err(ErrorInternalServerError(format!("API returned non-success status: {}", response.status())))
        // }
    }
}