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

use crate::cores::chat_models::chat_controller::{CompletionsResponse, CompletionsStreamResponse, ChatCompletionRequest};
use crate::middleware::qos::consume;
use crate::GLOBAL_CONFIG;
use crate::configs::settings::Config;

pub struct RequestInfo{
    pub req_model_name: String,
    pub userid: String, 
    pub appkey: String, 
    pub start_time: DateTime<Tz>
}

pub fn get_request_body(model_name: String, req_body: web::Json<ChatCompletionRequest>) -> (Value, bool) {
    // Build the basic request body
    let mut request_body = json!({
        "model": model_name,
        "messages": req_body.messages.clone(),
    });

    let mut is_stream = false;

    // Add optional parameters
    if let Some(temperature) = req_body.temperature {
        request_body["temperature"] = json!(temperature);
    }
    if let Some(top_p) = req_body.top_p {
        request_body["top_p"] = json!(top_p);
    }
    if let Some(n) = req_body.n {
        request_body["n"] = json!(n);
    }
    if let Some(stream) = req_body.stream {
        request_body["stream"] = json!(stream);
        is_stream = stream;
    }
    if let Some(stop) = req_body.stop.clone() {
        request_body["stop"] = json!(stop);
    }
    if let Some(max_tokens) = req_body.max_tokens {
        request_body["max_tokens"] = json!(max_tokens);
    }
    if let Some(presence_penalty) = req_body.presence_penalty {
        request_body["presence_penalty"] = json!(presence_penalty);
    }
    if let Some(frequency_penalty) = req_body.frequency_penalty {
        request_body["frequency_penalty"] = json!(frequency_penalty);
    }
    if let Some(logit_bias) = req_body.logit_bias {
        request_body["logit_bias"] = json!(logit_bias);
    }
    if let Some(user) = req_body.user.clone() {
        request_body["user"] = json!(user);
    }
    if let Some(file_id) = req_body.file_id.clone() {
        request_body["file_id"] = json!(file_id);
    }

    // Add optional stream_options
    if let Some(stream_options) = req_body.stream_options.clone() {
        request_body["stream_options"] = json!(stream_options);
    }

    return (request_body, is_stream);
}


// Handle non-streaming response requests
pub async fn completions_response_non_stream(response: Response, req_info: RequestInfo) -> Result<HttpResponse, Error> {
    // 1. Convet the response body to a CompletionResponse struct
    let response_text = response.text().await
        .map_err(|err| ErrorInternalServerError(format!("Failed to read response: {}", err)))?;

    let json_value: Value = serde_json::from_str(&response_text)
        .map_err(|err| ErrorInternalServerError(format!("Failed to parse unescaped JSON: {}, {}", err, response_text)))?;

    let chat_response: CompletionsResponse = match serde_json::from_value(json_value) {
        Ok(chat_response) => chat_response,
        Err(err) => return Err(ErrorInternalServerError(format!("Failed to deserialize into CompletionsChatResponse: {}", err))),
    };
    
    // 2. Return a custom response body
    let req_model_name = req_info.req_model_name.clone();
    let res = json!({
      "id": chat_response.id,
      "object": chat_response.object,
      "created": chat_response.created,
      "model": req_model_name,
      "choices": [
            {
                "index": chat_response.choices[0].index,
                "message": {
                    "role": chat_response.choices[0].message.role,
                    "reasoning_content": chat_response.choices[0].message.reasoning_content,
                    "content": chat_response.choices[0].message.content,
                    "tool_calls": chat_response.choices[0].message.tool_calls
                },
                "logprobs": chat_response.choices[0].logprobs,
                "finish_reason": chat_response.choices[0].finish_reason,
                "stop_reason": chat_response.choices[0].stop_reason
            }
      ],
      "usage": chat_response.usage,
      "prompt_logprobs": chat_response.prompt_logprobs
    });

    // 4. push kafka data
    let config = &*GLOBAL_CONFIG;
    push_kafka_data(req_model_name.clone(), config, chat_response.usage.total_tokens, chat_response.usage.completion_tokens, 
        chat_response.usage.prompt_tokens, req_info.userid.clone(), req_info.appkey, req_info.start_time);

    // 5. Consume tokens
    if config.coil_enabled {
        // 下述的model需要换成上述的chat_response.model；apikey需要传入
        // let status_is_success = consume("sk-4XNwrsq6bS9KD11E6xkrKEItGBcR".to_string(), "deepseek-ai/DeepSeek-R1-Distill-Llama-8B".to_string(), chat_response.usage.total_tokens).await?;
        if consume(req_info.userid, req_model_name, chat_response.usage.total_tokens).await? != "success" {
            return Err(ErrorInternalServerError("Failed to consume tokens"));
        }
    }

    Ok(HttpResponse::Ok().json(res))
}


// Handle streaming response requests
pub async fn completions_response_stream(response: Response, req_info: RequestInfo) -> Result<HttpResponse, Error> {
    // 1. create an asynchronous stream that sends each chunk of data obtained from the response to the client
    let mut body_stream = response.bytes_stream();
    let req_model_name = req_info.req_model_name.clone();
    let stream = async_stream::stream! {
        // let mut body_stream = body_stream;
        while let Some(chunk) = body_stream.next().await {
            match chunk {
                Ok(bytes) => {
                    // Convert bytes to string
                    let json_str = String::from_utf8_lossy(&bytes).to_string();
                    // Check if the chunk contains the "data: [DONE]" string
                    if json_str.contains("data: [DONE]") {
                        break;
                    }
                    //  Remove the prefix "data: " and the suffix "\n\n"
                    let json_str = json_str.trim_start_matches("data: ");
                    let json_str = json_str.trim_end(); 
                    
                    // Deserialize the JSON string into a Value
                    let json_value = match serde_json::from_str::<Value>(&json_str) {
                        Ok(value) => value,
                        Err(err) => {
                            yield Err(format!("Chunk failed to parse JSON form json_str: {}, Err: {}", json_str, err));
                            continue;
                        },   
                    };

                    // Try to convert json_value to CompletionsStreamResponse
                    let chat_response: CompletionsStreamResponse = match serde_json::from_value(json_value.clone()) {
                        Ok(chat_response) => chat_response,
                        Err(err) => {
                            yield Err(format!("Chunk failed to deserialize into CompletionsStreamResponse: {}, Err: {}", json_value, err));
                            continue;
                        },
                    };

                    // 判断是否为usage chunk
                    if let Some(usage) = &chat_response.usage {
                        let config = &*GLOBAL_CONFIG;
                        push_kafka_data(req_model_name.clone(), config, usage.total_tokens, usage.completion_tokens, 
                            usage.prompt_tokens, req_info.userid.clone(), req_info.appkey.clone(), req_info.start_time);

                        if config.coil_enabled {
                            if let Err(_) = consume(req_info.userid.clone(), req_model_name.clone(), usage.total_tokens).await {
                                yield Err(format!("Failed to consume tokens"));
                            }
                        }

                        let usage_chunk = json!({
                            "id": chat_response.id,
                            "model": req_model_name,
                            "created": chat_response.created,
                            "object": chat_response.object,
                            "choices": [],
                            "usage": usage
                        });

                        let usage_chunk = format!("data: {}\n\n", serde_json::to_string(&usage_chunk).unwrap());
                        yield Ok::<Bytes, String>(Bytes::from(usage_chunk));
                        break;
                    }

                    let chunk = transfer_chunk(chat_response, req_model_name.clone()).await.unwrap();
                    let chunk_str = format!("data: {}\n\n", serde_json::to_string(&chunk).unwrap());
                    yield Ok::<Bytes, String>(Bytes::from(chunk_str));
                },
                Err(err) => {
                    // If reading data fails, return an error
                    yield Err(format!("Stream read error: {}", err));
                }
            }
        }
    };  

    // 2. Create a new stream that combines the original stream and the response string
    let mut stream_iter = Box::pin(stream.fuse());
    let combined_stream = async_stream::stream! {
        // Then yield the remaining data from the original stream
        while let Some(chunk) = stream_iter.next().await {
            yield chunk; // Yield remaining chunks
        }

        let stop_str =  format!("data: [DONE]\n\n");
        yield Ok::<Bytes, String>(Bytes::from(stop_str));
    };
    
    // Return streaming response
    Ok(HttpResponse::Ok().content_type("text/event-stream").streaming(combined_stream))  
}

async fn transfer_chunk(chat_response: CompletionsStreamResponse, model_name: String) -> Result<Value, Box<dyn error::Error>> {
    // 判断是否为stop chunk
    if chat_response.choices[0].finish_reason == Some("stop".to_string()){
        let stop_chunk = json!({
            "id": chat_response.id,
            "model": model_name,
            "created": chat_response.created,
            "object": chat_response.object,
            "choices": [
                {
                    "index": chat_response.choices[0].index,
                    "delta": {
                        "content": ""
                    },
                    "finish_reason": "stop",
                    "stop_reason": null,
                    "logprobs": chat_response.choices[0].logprobs
                }
            ]
        });

        return Ok(stop_chunk);
    }

    // 判断是否为first chunk
    if let Some(role) = &chat_response.choices[0].delta.role {
        let first_chunk = json!({
            "id": chat_response.id,
            "model": model_name,
            "created": chat_response.created,
            "object": chat_response.object,
            "choices": [
                {
                    "index": chat_response.choices[0].index,
                    "delta": {
                        "role": role,
                        "content": chat_response.choices[0].delta.content
                    },
                    "finish_reason": null,
                    "logprobs": chat_response.choices[0].logprobs
                }
            ]
        });
    
        return Ok(first_chunk);
    }

    // 其他情况
    let normal_chunk = json!({
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
                "finish_reason": "",
                "logprobs": chat_response.choices[0].logprobs
            }
        ] 
    });

    return Ok(normal_chunk);
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