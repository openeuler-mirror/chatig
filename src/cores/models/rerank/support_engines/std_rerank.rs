use actix_web::{web, HttpResponse, Error};
use actix_web::error::{ErrorBadRequest, ErrorInternalServerError};
use async_trait::async_trait;

use crate::cores::models::rerank::rerank_utils::{get_request_body, build_std_rerank_response};
use crate::cores::control::services::ServiceManager;
use crate::cores::models::rerank::rerank_controller::{StdRerankRequest, StdRerankTrait};
use crate::cores::utils::get_response;

pub struct StdRerank{
    pub active_model: String,
}

#[async_trait]
impl StdRerankTrait for StdRerank {
    async fn rerank(&self, req_body: web::Json<StdRerankRequest>, aicpid: String) -> Result<HttpResponse, Error> {
        // 1. Read the model's parameter configuration
        let req_model_name = req_body.model.clone();

        let service_manager = ServiceManager::default();
        let service_option = service_manager.get_service_by_model(&self.active_model, None, aicpid).await?;
        let mut service = match service_option {
            Some(svc) => svc,
            None => return Err(ErrorBadRequest(format!("{} model is not supported", req_model_name))),
        };

        // 2. Build the request body
        let request_body = match get_request_body(req_body, service.model_name.clone(), service.servicetype.clone()) {
            Ok(body) => body,
            Err(err) => return Err(ErrorInternalServerError(format!("{}", err))),
        };
        
        // 3. Initiate a POST request via reqwest
        let (response, _) = match get_response(request_body, &mut service, req_model_name.clone()).await {
            Ok((resp, start_time)) => (resp, start_time),
            Err(err) => return Err(ErrorInternalServerError(format!("{}", err))),
        };

        // // 4. Parse the response
        // let std_response = match build_std_rerank_response(response, service.servicetype.clone()).await {
        //     Ok(resp) => resp,
        //     Err(err) => return Err(ErrorInternalServerError(format!("{}", err))),
        // };
        
    
        // Ok(HttpResponse::Ok().json(std_response))
        // 4) 读取上游响应并判错
        let status = response.status();
        let text = response.text().await.unwrap_or_default();

        if !status.is_success() {
            log::error!("Upstream rerank error {}: {}", status, text);
            return Ok(HttpResponse::build(status).body(text));
        }

        // 5) 宽松解析，兼容多种返回形态
        let root: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| ErrorInternalServerError(format!("bad json: {}, body={}", e, text)))?;

        // 根即数组：直接作为 items
        let items: Vec<serde_json::Value> = if let Some(arr) = root.as_array() {
            arr.clone()
        } else {
            // 兼容包裹层：body / output / results
            let body = root.get("body").unwrap_or(&root);
            let results = body
                .pointer("/output/results")
                .or_else(|| body.get("results"))
                .ok_or_else(|| ErrorInternalServerError(format!("Missing results array in response: {}", root)))?;

            results
                .as_array()
                .cloned()
                .ok_or_else(|| ErrorInternalServerError(format!("results is not an array: {}", results)))?
        };

        // 6) 统一返回
        Ok(HttpResponse::Ok().json(serde_json::json!({ "results": items })))
    }
}