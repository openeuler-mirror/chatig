use actix_web::{web, HttpResponse, Error};
use actix_web::error::{ErrorInternalServerError, ErrorBadRequest};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use chrono::Utc;
use chrono_tz::Asia::Shanghai;

use crate::apis::models_api::schemas::ChatCompletionRequest;
use crate::cores::chat_models::chat_controller::Completions;
use crate::cores::chat_models::chat_utils::{completions_response_stream, completions_response_non_stream};

pub struct Bailian;

#[async_trait]
impl Completions for Bailian{
    async fn completions(&self, req_body: web::Json<ChatCompletionRequest>, userid: String, appkey: String) -> Result<HttpResponse, Error> {

        // 1. Read the model's parameter configuration
        let (reqwest_url, model_name, api_key) = get_model_params(&req_body.model)?;

        // 2. Build the request body
        let stream = req_body.stream.unwrap_or(false).clone();
        let request_body = json!({
            "model": &model_name,
            "temperature": req_body.temperature.unwrap_or(0.7).clone(),
            "n": req_body.n.unwrap_or(1).clone(),
            "stream": stream,
            "stop": null,
            "presence_penalty": req_body.presence_penalty.unwrap_or(0).clone(),
            "frequency_penalty": req_body.frequency_penalty.unwrap_or(0).clone(),
            "logit_bias": null,
            "user": req_body.user.clone(),
            "max_tokens": req_body.max_tokens,
            "messages": req_body.messages
        });

        // 3. Use reqwest to initiate a POST request
        let start_time = Utc::now().with_timezone(&Shanghai);
        let client = Client::new();
        let response = match client.post(&reqwest_url)
            .header("Authorization", format!("Bearer {}", api_key))
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

// get the parameter information of the specific model
fn get_model_params(model_name: &str) -> Result<(String, String, String), Error> {
    // 待数据表字段确定后，通过数据访问
    let api_key = "sk-xxxxxxx".to_string();

    let platform_series = model_name.split("/").next().unwrap_or("");
    let reqwest_url  = match platform_series {
        "Bailian" => "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions",
        _ => return Err(ErrorBadRequest(format!("Unsupported {} platform series!", platform_series))),
    };
    let parts: Vec<&str> = model_name.split('/').collect();
    let model = parts.get(1).unwrap_or(&"");

    Ok((reqwest_url.to_string(), model.to_string(), api_key))
}
