use actix_web::{web, HttpResponse, Error};
use actix_web::error::{ErrorBadRequest, ErrorInternalServerError};
use async_trait::async_trait;
use serde_json::Value;

use crate::cores::control::services::ServiceManager;
use crate::cores::models::rerank::rerank_controller::{LlamaBoxRerankTrait, LlamaBoxRerankRequest};
use crate::cores::utils::get_response;

pub struct LlamaBoxRerank{
    pub active_model: String,
}

#[async_trait]
impl LlamaBoxRerankTrait for LlamaBoxRerank {
    async fn rerank(&self, req_body: web::Json<LlamaBoxRerankRequest>, aicpid: String) -> Result<HttpResponse, Error> {
        // 1. Read the model's parameter configuration
        let req_model_name = req_body.model.clone();

        let service_manager = ServiceManager::default();
        let service_option = service_manager.get_service_by_model(&self.active_model, None, aicpid).await?;
        let mut service = match service_option {
            Some(svc) => svc,
            None => return Err(ErrorBadRequest(format!("{} model is not supported", req_model_name))),
        };

        // 2. Build the request body
        let request_body = match get_request_body(req_body.into_inner(), service.model_name.clone()) {
            Ok(body) => body,
            Err(err) => return Err(ErrorInternalServerError(format!("{}", err))),
        };

        // 3. Initiate a POST request via reqwest
        let (response, _) = match get_response(request_body, &mut service, req_model_name.clone()).await {
            Ok((resp, start_time)) => (resp, start_time),
            Err(err) => return Err(ErrorInternalServerError(format!("{}", err))),
        };

        let mut json_resp = response.json::<Value>().await.map_err(|err| {
            ErrorInternalServerError(format!("Failed to parse JSON: {}", err))
        })?;
        json_resp["model"] = req_model_name.into();

        Ok(HttpResponse::Ok().json(json_resp))
    }
}

// fn get_request_body(
fn get_request_body(
    req_body: LlamaBoxRerankRequest,
    model_name: String,
) -> Result<Value, String> {
    let mut body = req_body;
    body.model = model_name;
    let request_body = serde_json::to_value(body)
        .map_err(|e| format!("Serialization error: {}", e))?;

    Ok(request_body)
}