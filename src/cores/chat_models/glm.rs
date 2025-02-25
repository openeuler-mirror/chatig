use actix_web::{web, HttpResponse, Error};
use actix_web::error::{ErrorInternalServerError, ErrorBadRequest};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use chrono::Utc;
use chrono_tz::Asia::Shanghai;
use std::time::Duration;

use crate::apis::models_api::schemas::ChatCompletionRequest;
use crate::cores::control::services::ServiceManager;
use crate::cores::chat_models::chat_controller::Completions;
use crate::cores::chat_models::chat_utils::{completions_response_stream, completions_response_non_stream, add_stream_options};

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
        let stream = req_body.stream.unwrap_or(false).clone();
        let mut request_body = json!({
            "model": service.model_name,
            "temperature": req_body.temperature.unwrap_or(0.95).clone(),
            "n": req_body.n.unwrap_or(1).clone(),
            "stream": stream,
            "stop": null,
            "presence_penalty": req_body.presence_penalty.unwrap_or(0).clone(),
            "frequency_penalty": req_body.frequency_penalty.unwrap_or(0).clone(),
            "logit_bias": null,
            "user": req_body.user.clone(),
            // "max_tokens": req_body.max_tokens.unwrap_or(max_tokens).clone(),
            "messages": req_body.messages
        });

        // Append stream-specific options if needed
        if stream {
            request_body = add_stream_options(request_body, service.servicetype);
        }

        let start_time = Utc::now().with_timezone(&Shanghai);
        // 3. Use reqwest to initiate a POST request
        let client = Client::builder()
            .timeout(Duration::from_secs(300)) // 设置总超时时间为300秒
            .connect_timeout(Duration::from_secs(10)) // 设置连接超时时间为10秒（可选）
            .build()
            .map_err(|err| ErrorInternalServerError(format!("Failed to build client: {}", err)))?;

        let response = match client.post(service.url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await{
                Ok(resp) => resp, 
                Err(err) => return Err(ErrorInternalServerError(format!("Request failed: {}", err))),
            };
        
        // 4. Return the response based on the request's streaming status
        if stream {
            // Handle streaming response requests
            completions_response_stream(req_body, response, userid, appkey, start_time).await
        } else {
            // handle non-streaming response requests
            completions_response_non_stream(req_body, response, userid, appkey, start_time).await
        }
    }
}

