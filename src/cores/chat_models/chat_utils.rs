use actix_web::{web, HttpResponse, Error};
use actix_web::error::ErrorInternalServerError;
use reqwest::Response;
use serde_json::{Value, json};
use futures::StreamExt;
use bytes::Bytes;
use chrono::{Utc, DateTime};
use chrono_tz::Tz;
use chrono_tz::Asia::Shanghai;
use log::info;
use std::error;

use crate::apis::models_api::schemas::ChatCompletionRequest;
use crate::cores::schemas::{CompletionsResponse, CompletionsStreamResponse};
use crate::middleware::qos::consume;
use crate::GLOBAL_CONFIG;
use crate::configs::settings::Config;

// Add stream options based on the underlying inference engine
pub fn add_stream_options(mut request_body: Value, infer_engin_type: String) -> Value {
    if infer_engin_type == "vllm" {
        let mut stream_options = serde_json::Map::new();
        stream_options.insert("include_usage".to_string(), json!("True"));
        
        // Convert base_body into a Map and add stream_options
        if let Some(base_map) = request_body.as_object_mut() {
            base_map.insert("stream_options".to_string(), Value::Object(stream_options));
        }
    } else if infer_engin_type == "ollama" {
        let mut stream_options = serde_json::Map::new();
        stream_options.insert("include_usage".to_string(), json!(true));
        
        // Convert base_body into a Map and add stream_options
        if let Some(base_map) = request_body.as_object_mut() {
            base_map.insert("stream_options".to_string(), Value::Object(stream_options));
        }
    }

    request_body
}


// Handle non-streaming response requests
pub async fn completions_response_non_stream(
    req_body: web::Json<ChatCompletionRequest>, 
    response: Response, 
    userid: String, 
    appkey: String, 
    start_time: DateTime<Tz>) 
    -> Result<HttpResponse, Error> {
    // 1. Check if the response is successful
    let model_name = req_body.model.clone();
    if !response.status().is_success() {
        return Err(ErrorInternalServerError(format!("{} request failed: {}", model_name, response.status())));
    }

    // 2. Convet the response body to a CompletionResponse struct
    let response_text = response.text().await
        .map_err(|err| ErrorInternalServerError(format!("Failed to read response: {}", err)))?;

    let trimmed_text = response_text.trim_matches('"');

    let unescaped_text = trimmed_text.replace("\\\"", "\"").replace("\\\\", "\\");

    let json_value: Value = serde_json::from_str(&unescaped_text)
        .map_err(|err| ErrorInternalServerError(format!("Failed to parse unescaped JSON: {}, {}", err, unescaped_text)))?;

    let chat_response: CompletionsResponse = match serde_json::from_value(json_value) {
        Ok(chat_response) => chat_response,
        Err(err) => return Err(ErrorInternalServerError(format!("Failed to deserialize into CompletionsChatResponse: {}", err))),
    };
    
    // 3. Return a custom response body
    let res = json!({
      "id": chat_response.id,
      "object": chat_response.object,
      "created": chat_response.created,
      "model": model_name,
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

    // 4. push kafka data
    let config = &*GLOBAL_CONFIG;
    push_kafka_data(model_name.clone(), config, chat_response.usage.total_tokens, chat_response.usage.completion_tokens, 
        chat_response.usage.prompt_tokens, userid.clone(), appkey, start_time);

    // 5. Consume tokens
    if config.coil_enabled {
        // 下述的model需要换成上述的chat_response.model；apikey需要传入
        // let status_is_success = consume("sk-4XNwrsq6bS9KD11E6xkrKEItGBcR".to_string(), "deepseek-ai/DeepSeek-R1-Distill-Llama-8B".to_string(), chat_response.usage.total_tokens).await?;
        if consume(userid, model_name, chat_response.usage.total_tokens).await? != "success" {
            return Err(ErrorInternalServerError("Failed to consume tokens"));
        }
    }

    Ok(HttpResponse::Ok().json(res))
}


// Handle streaming response requests
pub async fn completions_response_stream(
    req_body: web::Json<ChatCompletionRequest>, 
    response: Response, 
    userid: String, 
    appkey: String, 
    start_time: DateTime<Tz>) 
    -> Result<HttpResponse, Error> {
    // 1. Check if the response is successful
    let model_name = req_body.model.clone();
    if !response.status().is_success() {
        return Err(ErrorInternalServerError(format!("{} request failed: {}", model_name, response.status())));
    }

    // 2. Get the byte stream of the response body, and skip the first chunk of data
    let mut body_stream = response.bytes_stream();
    let first_chunk = body_stream.next().await;
    let (first_str, last_str) = match first_chunk {
        Some(chunk) => {
            if let Ok(bytes) = chunk {
                get_first_last_str(bytes, model_name.clone()).map_err(|err| ErrorInternalServerError(err))?
            } else {
                return Err(ErrorInternalServerError("Failed to read response"));
            }
        },
        None => return Err(ErrorInternalServerError("Failed to read response")),
    };

    // 3. reate an asynchronous stream that sends each chunk of data obtained from the response to the client
    let stream = async_stream::stream! {
        // let mut body_stream = body_stream;
        while let Some(chunk) = body_stream.next().await {
            match chunk {
                Ok(bytes) => {
                    // Convert bytes to JSON string, Remove the prefix "data: " and suffix "data: [DONE]" from the JSON string
                    let json_str = String::from_utf8_lossy(&bytes).to_string();
                    let json_str = json_str.trim_start_matches("data: ");             
                    let json_str = json_str.replace("data: [DONE]", "");
                    let json_str = json_str.trim_end();    
                    
                    // Deserialize the JSON string into a Value
                    let json_value = match serde_json::from_str::<Value>(&json_str) {
                        Ok(value) => value,
                        Err(_) => continue,   
                    };

                    // Try to convert json_value to KbChatResponse
                    let chat_response: CompletionsStreamResponse = match serde_json::from_value(json_value) {
                        Ok(chat_response) => chat_response,
                        Err(_err) => continue,
                    };

                    // If usage is exist
                    match &chat_response.usage {
                        Some(usage) => {
                            let config = &*GLOBAL_CONFIG;
                            push_kafka_data(req_body.model.clone(), config, usage.total_tokens, usage.completion_tokens, 
                                usage.prompt_tokens, userid.clone(), appkey.clone(), start_time);

                            if config.coil_enabled {
                                if let Err(_) = consume(userid.clone(), model_name.clone(), usage.total_tokens).await {
                                    yield Err(format!("Failed to consume tokens"));
                                }
                            }
                            let res = json!({
                                "id": chat_response.id,
                                "model": model_name,
                                "created": chat_response.created,
                                "object": chat_response.object,
                                "choices": [],
                                "usage": {
                                    "prompt_tokens": usage.prompt_tokens,
                                    "completion_tokens": usage.completion_tokens,
                                    "total_tokens": usage.total_tokens
                                }
                            });
                            let res_str = format!("data: {}\n\n", serde_json::to_string(&res).unwrap());
                            yield Ok::<Bytes, String>(Bytes::from(res_str));
                            continue;
                        },
                        None => {
                            if chat_response.choices[0].finish_reason == Some("stop".to_string()){
                                continue;
                            }
                        },
                    }

                    let res = json!({
                        "id": chat_response.id,
                        "model": model_name,
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

    // 4. Create a new stream that combines the original stream and the response string
    let mut stream_iter = Box::pin(stream.fuse());
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
    Ok(HttpResponse::Ok().content_type("text/event-stream").streaming(combined_stream))
}

fn get_first_last_str(bytes: Bytes, model_name: String) -> Result<(String, String), Box<dyn error::Error>> {
    // Convert bytes to JSON string
    let json_str_owned = String::from_utf8_lossy(&bytes).to_string();
    // Remove the prefix "data: " from the JSON string
    let json_str = json_str_owned.trim_start_matches("data: ").to_string();

    // Deserialize the JSON string into OpenAIDeltaMessage
    let json_value: Value = serde_json::from_str(&json_str)
        .map_err(|err| format!("Failed to parse response as JSON: {}", err))?;

    let chat_response: CompletionsStreamResponse = serde_json::from_value(json_value)
        .map_err(|err| format!("Failed to deserialize into CompletionsStreamResponse: {}", err))?;

    // Create the first response string for streaming data
    let res = json!({
        "id": chat_response.id,
        "model": model_name,
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
    let first_str = format!("data: {}\n\n", serde_json::to_string(&res).map_err(|e| format!("Failed to serialize JSON: {}", e))?);

    let res = json!({
        "id": chat_response.id,
        "model": model_name,
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
    let last_str = format!("data: {}\n\n", serde_json::to_string(&res).map_err(|e| format!("Failed to serialize JSON: {}", e))?);

    Ok((first_str, last_str))
}


fn push_kafka_data(model_name: String,
                   config: &Config,
                   total_tokens: u32,
                   completion_tokens: u32,
                   prompt_tokens: u32,
                   userid: String,
                   appkey: String,
                   start_time: DateTime<Tz>) {
    let utc_time = Utc::now().with_timezone(&Shanghai); // 转换为上海时间
    let end_time = Utc::now().with_timezone(&Shanghai); // 转换为上海时间
    let data: Value = json!({
        "accountId": userid,
        "cloudRegionName": config.cloud_region_name,
        "cloudRegionId": config.cloud_region_id,
        "modelName": model_name,
        "appKey": appkey,
        "startTime": start_time.with_timezone(&Shanghai).to_rfc3339(),
        "endTime": end_time.to_rfc3339(),
        "totalTokens": total_tokens,
        "completionTokens": completion_tokens,
        "promptTokens": prompt_tokens,
        "time": utc_time.to_rfc3339(),
    });
    let kafka_json: String = serde_json::to_string(&data).unwrap();
    info!(target: "token", "{}", kafka_json);
}