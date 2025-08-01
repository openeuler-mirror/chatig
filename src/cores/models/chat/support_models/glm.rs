use actix_web::{web, HttpResponse, Error};
use actix_web::error::{ErrorInternalServerError, ErrorBadRequest};
use async_trait::async_trait;
use reqwest::Client;
use chrono::Utc;
use chrono_tz::Asia::Shanghai;
use std::time::Duration;

use crate::cores::control::services::ServiceManager;
use crate::cores::chat_models::chat_controller::{Completions, ChatCompletionRequest};
use crate::cores::chat_models::chat_utils::{completions_response_stream, completions_response_non_stream, get_request_body, RequestInfo};

pub struct GLM{
    pub model_name: String,
}

#[async_trait]
impl Completions for GLM{
    async fn completions(&self, req_body: web::Json<ChatCompletionRequest>, userid: String, appkey: String) -> Result<HttpResponse, Error> {
        // 1. Read the model's parameter configuration
        let service_manager = ServiceManager::default();
        let service = service_manager.get_service_by_model(&self.model_name).await?;
        let service = match service {
            Some(service) => service,
            None => return Err(ErrorBadRequest(format!("{} model is not supported", self.model_name))),
        };

        // 2. Build the request body
        let (request_body, is_stream) = get_request_body(service.model_name, req_body);

        // 3. Use reqwest to initiate a POST request
        let client = Client::builder()
            .timeout(Duration::from_secs(300)) // 设置总超时时间为300秒
            .connect_timeout(Duration::from_secs(10)) // 设置连接超时时间为10秒（可选）
            .build()
            .map_err(|err| ErrorInternalServerError(format!("Failed to build client: {}", err)))?;

        let start_time = Utc::now().with_timezone(&Shanghai);
        let response = match client.post(service.url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await{
                Ok(resp) => resp, 
                Err(err) => return Err(ErrorInternalServerError(format!("Request failed: {}", err))),
            };

        if !response.status().is_success() {
            return Err(ErrorInternalServerError(format!("{} request failed: {}", self.model_name, response.status())));
        }
        
        // 4. Return the response based on the request's streaming status
        let req_info = RequestInfo{
            req_model_name: self.model_name.clone(),
            userid: userid.clone(),
            appkey: appkey.clone(),
            start_time: start_time,
        };
        if is_stream{
            // Handle streaming response requests
            // let body_stream = response.bytes_stream();
            // Ok(HttpResponse::Ok().content_type("text/event-stream").streaming(body_stream))
            completions_response_stream(response, req_info).await
        } else {
            // Handle non-streaming response requests
            completions_response_non_stream(response, req_info).await
        }
    }
}

