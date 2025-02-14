use actix_web::{web, HttpResponse, Error};
use actix_web::error::{ErrorInternalServerError, ErrorBadRequest};
use async_trait::async_trait;
use reqwest::{Client, Response};
use serde_json::{Value, json};
use futures::StreamExt;
use bytes::Bytes;
use chrono::Utc;
use log::info;
use chrono::DateTime;

use crate::apis::models_api::schemas::ChatCompletionRequest;
use crate::cores::schemas::{CompletionsResponse, CompletionsStreamResponse};
use crate::cores::control::services::ServiceManager;
use crate::cores::chat_models::chat_controller::Completions;
use crate::GLOBAL_CONFIG;
use crate::middleware::qos::consume;


pub struct Qwen{
    pub model_name: String,
}

#[async_trait]
impl Completions for Qwen{
    async fn completions(&self, req_body: web::Json<ChatCompletionRequest>, userid: String, curl_mode: String, appkey: String) -> Result<HttpResponse, Error> {
        // 1. Read the model's parameter configuration
        let service_manager = ServiceManager::default();
        let service = service_manager.get_service_by_model(&self.model_name).await?;
        let service = match service {
            Some(service) => service,
            None => return Err(ErrorBadRequest(format!("{} model is not supported", self.model_name))),
        };

        // 2. Build the request body
        let stream = req_body.stream.unwrap_or(true);
        let mut request_body = json!({
            "model": service.model_name,
            "temperature": req_body.temperature.unwrap_or(0.3),
            "n": req_body.n.unwrap_or(1),
            "stream": stream,
            "stop": null,
            "presence_penalty": req_body.presence_penalty.unwrap_or(0),
            "frequency_penalty": req_body.frequency_penalty.unwrap_or(0),
            "logit_bias": null,
            "user": req_body.user.clone(),
            "max_tokens": req_body.max_tokens,
            "messages": req_body.messages,
        });

        // Append stream-specific options if needed
        if stream {
            let mut stream_options = serde_json::Map::new();
            stream_options.insert("include_usage".to_string(), json!("True"));
            
            // Convert base_body into a Map and add stream_options
            if let Some(base_map) = request_body.as_object_mut() {
                base_map.insert("stream_options".to_string(), Value::Object(stream_options));
            }
        }

        let start_time = Utc::now();
        // 3. Use reqwest to initiate a POST request
        let client = Client::new();
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
            completions_response_stream(response, userid, curl_mode, appkey, start_time).await
        } else {
            // handle non-streaming response requests
            completions_response_non_stream(response, userid, curl_mode, appkey, start_time).await
        }
    }
}


// Handle non-streaming response requests
async fn completions_response_non_stream(response: Response, userid: String, curl_mode: String, appkey: String, start_time: chrono::DateTime<Utc>) -> Result<HttpResponse, Error> {

    // 1. Parse the JSON response body into the KbChatResponse struct
    let response_text = response.text().await
        .map_err(|err| ErrorInternalServerError(format!("Failed to read response: {}", err)))?;

    // 2. Remove escape characters from the string
    let trimmed_text = response_text.trim_matches('"');
    let unescaped_text = trimmed_text.replace("\\\"", "\"").replace("\\\\", "\\");

    // 3. Convert unescaped_text to a JSON object
    let json_value: Value = serde_json::from_str(&unescaped_text)
        .map_err(|err| ErrorInternalServerError(format!("Failed to parse unescaped JSON: {}, {}", err, unescaped_text)))?;

    let chat_response: CompletionsResponse = match serde_json::from_value(json_value) {
        Ok(chat_response) => chat_response,
        Err(err) => return Err(ErrorInternalServerError(format!("Failed to deserialize into CompletionsChatResponse: {}", err))),
    };
    
    // 5. Return a custom response body
    let res = json!({
      "id": chat_response.id,
      "object": chat_response.object,
      "created": chat_response.created,
      "model": chat_response.model,
      "choices": [
          {
              "index": chat_response.choices[0].index,
              "message": {
                  "role": chat_response.choices[0].message.role,
                  "content": chat_response.choices[0].message.content
              },
              "finish_reason": chat_response.choices[0].finish_reason,
          }
      ],
      "usage": {
          "prompt_tokens": chat_response.usage.prompt_tokens,
          "completion_tokens": chat_response.usage.completion_tokens,
          "total_tokens": chat_response.usage.total_tokens
      }
    });

    let config = &*GLOBAL_CONFIG;
    let utc_time: DateTime<Utc> = Utc::now();
    let end_time = Utc::now();
    let data: Value = json!({
        "userID": userid,
        "cloudRegionName": config.cloud_region_name,
        "cloudRegionId": config.cloud_region_id,
        "modelName": curl_mode,
        "appKey": appkey,
        "startTime": start_time.format("%Y-%m-%d %H:%M:%S").to_string(),
        "endTime": end_time.format("%Y-%m-%d %H:%M:%S").to_string(),
        "totalTokens": chat_response.usage.total_tokens,
        "completionTokens": chat_response.usage.completion_tokens,
        "promptTokens": chat_response.usage.prompt_tokens,
        "time": utc_time.format("%Y-%m-%d %H:%M:%S").to_string(),
    });
    let kafka_json: String = serde_json::to_string(&data).unwrap();
    info!(target: "token", "{}", kafka_json);

    let config = &*GLOBAL_CONFIG;
    let coil_enabled = config.coil_enabled;
    if coil_enabled {
        // 下述的model需要换成上述的chat_response.model；apikey需要传入
        // let status_is_success = consume("sk-4XNwrsq6bS9KD11E6xkrKEItGBcR".to_string(), "deepseek-ai/DeepSeek-R1-Distill-Llama-8B".to_string(), chat_response.usage.total_tokens).await?;
        let status_is_success = consume(userid, curl_mode, chat_response.usage.total_tokens).await?;
        if status_is_success == "success" {
        } else {
            return Err(ErrorInternalServerError("Failed to consume tokens"));
        }
    }

    Ok(HttpResponse::Ok().json(res))
}


// Handle streaming response requests
async fn completions_response_stream(response: Response, userid: String, curl_mode: String, appkey: String, start_time: chrono::DateTime<Utc>) -> Result<HttpResponse, Error> {

    // Get the byte stream of the response body, and skip the first chunk of data
    let mut body_stream = response.bytes_stream();

    let mut first_str = String::new();
    let mut last_str = String::new();
    let first_chunk = body_stream.next().await;
    
    if let Some(Ok(bytes)) = first_chunk {
        // Convert bytes to JSON string
        let json_str = String::from_utf8_lossy(&bytes);
        //  Remove the prefix "data: " from the JSON string
        let json_str = json_str.trim_start_matches("data: ");

        // Deserialize the JSON string into OpenAIDeltaMessage
        let json_value: Value = serde_json::from_str(&json_str)
            .map_err(|err| ErrorInternalServerError(format!("Failed to parse response as JSON: {}", err)))?;

        let chat_response: CompletionsStreamResponse = serde_json::from_value(json_value)
            .map_err(|err| ErrorInternalServerError(format!("Failed to deserialize into CompletionsStreamResponse: {}", err)))?;

        // Create the first response string for streaming data
        let res = json!({
            "id": chat_response.id,
            "model": chat_response.model,
            "created": chat_response.created,
            "object": chat_response.object,
            "choices": [
                {
                    "index": chat_response.choices[0].index,
                    "delta": {
                        "role": chat_response.choices[0].delta.role,
                        "content": chat_response.choices[0].delta.content
                    },
                    "finish_reason": ""
                }
            ]
        });
        first_str = format!("data: {}\n\n", serde_json::to_string(&res).unwrap());

        let res = json!({
            "id": chat_response.id,
            "model": chat_response.model,
            "created": chat_response.created,
            "object": chat_response.object,
            "choices": [
                {
                    "index": chat_response.choices[0].index,
                    "delta": {},
                    "finish_reason": "stop"
                }
            ]
        });
        last_str = format!("data: {}\n\n", serde_json::to_string(&res).unwrap());
    }

    // reate an asynchronous stream that sends each chunk of data obtained from the response to the client
    let stream = async_stream::stream! {
        // let mut body_stream = body_stream;
        while let Some(chunk) = body_stream.next().await {
            match chunk {
                Ok(bytes) => {
                    // Convert bytes to JSON string
                    let json_str = String::from_utf8_lossy(&bytes);
                    //  Remove the prefix "data: " from the JSON string
                    let json_str = json_str.trim_start_matches("data: ");

                    let json_value: Value = match serde_json::from_str(&json_str) {
                        Ok(json) => json,
                        Err(_err) => {
                            // yield Err(format!("Failed to parse response as JSON: {}", err));
                            continue;
                        }
                    };

                    // Try to convert json_value to KbChatResponse
                    let chat_response: CompletionsStreamResponse = match serde_json::from_value(json_value) {
                        Ok(chat_response) => chat_response,
                        Err(_err) => {
                            // yield Err(format!("Failed to deserialize into KbChatResponse: {}", err));
                            continue;
                        }
                    };

                    // If usage is exist
                    match &chat_response.usage {
                        Some(usage) => { 
                            let config = &*GLOBAL_CONFIG;
                            let utc_time: DateTime<Utc> = Utc::now();
                            let end_time = Utc::now();
                            let data: Value = json!({
                                "userID": userid,
                                "cloudRegionName": config.cloud_region_name,
                                "cloudRegionId": config.cloud_region_id,
                                "modelName": curl_mode,
                                "appKey": appkey,
                                "startTime": start_time.format("%Y-%m-%d %H:%M:%S").to_string(),
                                "endTime": end_time.format("%Y-%m-%d %H:%M:%S").to_string(),
                                "totalTokens": usage.total_tokens,
                                "completionTokens": usage.completion_tokens,
                                "promptTokens": usage.prompt_tokens,
                                "time": utc_time.format("%Y-%m-%d %H:%M:%S").to_string(),
                            });
                            let kafka_json: String = serde_json::to_string(&data).unwrap();
                            info!(target: "token", "{}", kafka_json);
                            break;
                        },
                        None => {
                            if chat_response.choices[0].finish_reason == Some("stop".to_string()){
                                continue;
                            }
                        },
                    }

                    let res = json!({
                        "id": chat_response.id,
                        "model": chat_response.model,
                        "created": chat_response.created,
                        "object": chat_response.object,
                        "choices": [
                            {
                                "index": chat_response.choices[0].index,
                                "delta": {
                                    "content": chat_response.choices[0].delta.content
                                },
                                "finish_reason": ""
                            }
                        ] 
                    });

                    // Convert res to String and add "data: " prefix
                    let res_str = format!("data: {}\n\n", serde_json::to_string(&res).unwrap());
                    yield Ok::<Bytes, String>(Bytes::from(res_str));
                },
                Err(err) => {
                    // If reading data fails, return an error
                    yield Err(format!("Stream read error: {}", err));
                }
            }
        }
    };

    // Convert the stream into a Vec for buffering
    let mut stream_iter = Box::pin(stream.fuse());

    // Create a new stream that combines the original stream and the response string
    let combined_stream = async_stream::stream! {
        // Yield first response string
        yield Ok::<Bytes, String>(Bytes::from(first_str));

        // Then yield the remaining data from the original stream
        while let Some(chunk) = stream_iter.next().await {
            yield chunk; // Yield remaining chunks
        }

        // Yield last response string
        yield Ok::<Bytes, String>(Bytes::from(last_str));

        let stop_str =  format!("data: [DONE]\n\n");
        yield Ok::<Bytes, String>(Bytes::from(stop_str));
    };
    
    // Return streaming response
    Ok(HttpResponse::Ok()
        .content_type("text/event-stream")
        .streaming(combined_stream))
}