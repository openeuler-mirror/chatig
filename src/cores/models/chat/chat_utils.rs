use actix_web::{web, HttpResponse, Error};
use actix_web::error::ErrorInternalServerError;
use serde_json::{Value, json};
use bytes::Bytes;
use log;
use std;
use serde::Serialize;
use reqwest::Response;
use chrono_tz::Asia::Shanghai;
use chrono::{Utc, DateTime};
use chrono_tz::Tz;
use futures_util::StreamExt;

use crate::cores::models::chat::chat_controller::{
    CompletionsResponse,
    CompletionsStreamResponse,
    ChatCompletionRequest,
    Content,
    ContentData,
};
use crate::GLOBAL_CONFIG;
use crate::configs::settings::Config;

#[derive(Clone)]
#[allow(dead_code)]
pub struct RequestInfo {
    pub req_model_name: String,
    pub user_name: String, 
    pub user_id: String, 
    pub service_name: String, 
    pub service_id: String, 
    pub start_time: DateTime<Tz>,
    pub return_usage: bool,
    pub user_type: String,
    pub user_level: String,
    pub pay_status: String,
    pub instance_id: String,
}

#[derive(Serialize)]
pub struct LLMMessage {
    pub role: String,
    pub content: String,
}

pub fn get_messages(req_body: &ChatCompletionRequest) -> Result<Vec<LLMMessage>, String> {
    // Initialize messages as an empty vector
    let mut messages = Vec::new();
    
    // Process each message in the request body
    for mes in &req_body.messages {
        let message = match &mes.content {
            Content::Text(text) => {
                let role_str = mes.role.to_string();
                if text.is_empty() {
                    return Err(format!("{} content cannot be empty", role_str).to_string());
                }
                let message = LLMMessage {
                    role: role_str,
                    content: text.clone(),
                };
                
                message
            },
            Content::Array(content_parts) => {
                let role_str = mes.role.to_string();
                if content_parts.is_empty() {
                    return Err(format!("{} content cannot be empty", role_str).to_string());
                }
                
                let mut message = None;
                for content_part in content_parts {
                    if &content_part.r#type == "text" {
                        if let ContentData::Text { text } = &content_part.data {
                            let msg = LLMMessage {
                                role: role_str,
                                content: text.clone(),
                            };
                            message = Some(msg);
                            break; // Take the first text content
                        }
                    } else if &content_part.r#type == "image_url" {
                        return Err("Image Content is not supported for model!".to_string());
                    } else if &content_part.r#type == "input_audio for model!" {
                        return Err("Audio Content is not supported for model!".to_string());
                    } else if &content_part.r#type == "file" {
                        return Err("File Content is not supported for model!".to_string());
                    } else {
                        return Err("Content type is supported: ['text', 'image_url', 'input_audio', 'file']".to_string());
                    }
                }
                
                message.ok_or_else(|| "No valid text content found in array".to_string())?
            },
            Content::Assistant(assistant_content) => {
                let message = LLMMessage {
                    role: mes.role.to_string(),
                    content: assistant_content.content.as_ref().unwrap().clone(),
                };
                
                message
            }
        };
        
        messages.push(message);
    }
    
    Ok(messages)
}

pub async fn get_request_body(
    req_body: web::Json<ChatCompletionRequest>,
    model_name: String,
) -> Result<(Value, bool, bool), String> {
    // Build the basic request body
    let llm_messages = match get_messages(&req_body) {
        Ok(messages) => messages,
        Err(err) => return Err(err),
    };

    let mut request_body = json!({
        "model": model_name,
        "messages": llm_messages,
    });

    let mut is_stream = false;
    let mut return_usage = false;

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

        if stream {
            // Always return usage
            let stream_options = json!({ "include_usage": true });
            request_body["stream_options"] = stream_options;
        }
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
    if let Some(tool_choice) = req_body.tool_choice.clone() {
        request_body["tool_choice"] = json!(tool_choice);
    }
    if let Some(tools) = req_body.tools.clone() {
        request_body["tools"] = json!(tools);
    }

    // Judge whether to return usage
    if let Some(options) = req_body.stream_options.clone() {
        if let Some(include_usage) = options.include_usage {
            if include_usage {
                return_usage = true;
            }
        }
    }

    Ok((request_body, is_stream, return_usage))
}

pub async fn completions_response_non_stream(
    response: Response,
    req_info: RequestInfo,
) -> Result<HttpResponse, Error> {
    // 1. Convert the response body to a CompletionResponse struct
    let response_text = response
        .text()
        .await
        .map_err(|err| ErrorInternalServerError(format!("Failed to read response: {}", err)))?;

    let json_value: Value = serde_json::from_str(&response_text).map_err(|err| {
        ErrorInternalServerError(format!(
            "Failed to parse unescaped JSON: {}, {}",
            err, response_text
        ))
    })?;

    let chat_response: CompletionsResponse = serde_json::from_value(json_value).map_err(|err| {
        ErrorInternalServerError(format!(
            "Failed to deserialize into CompletionsChatResponse: {}",
            err
        ))
    })?;

    // 2. Return a custom response body
    let req_model_name = req_info.req_model_name.clone();
    let res = build_res(chat_response.clone(), req_model_name.clone());

    // 3. Push Kafka data
    let config = &*GLOBAL_CONFIG;
    let total_tokens = chat_response.usage.total_tokens;
    push_kafka_data(req_model_name.clone(), config, total_tokens, chat_response.usage.completion_tokens, 
        chat_response.usage.prompt_tokens, req_info.user_id.clone(), req_info.service_id.clone(), req_info.start_time);

    Ok(HttpResponse::Ok().json(res))
}

fn build_res(chat_response: CompletionsResponse,req_model_name: String) -> Value {
    let mut res = json!({
        "id": chat_response.id,
        "object": chat_response.object,
        "created": chat_response.created,
        "model": req_model_name,
        "usage": chat_response.usage,
        "choices": [{
            "index": &chat_response.choices[0].index,
            "stop_reason": &chat_response.choices[0].stop_reason,
            "message": {
                "role": &chat_response.choices[0].message.role,
                "content": &chat_response.choices[0].message.content
            },
            
        }],
    });

    if let Some(reasoning_content) = &chat_response.choices[0].message.reasoning_content {
        res["choices"][0]["message"]["reasoning_content"] = json!(reasoning_content);
    }
    if let Some(refusal) = &chat_response.choices[0].message.refusal {
        res["choices"][0]["message"]["refusal"] = json!(refusal);
    }
    if let Some(tool_calls) = &chat_response.choices[0].message.tool_calls {
        res["choices"][0]["message"]["tool_calls"] = json!(tool_calls);
    }

    if let Some(logprobs) = &chat_response.choices[0].logprobs {
        res["choices"][0]["logprobs"] = json!(logprobs);
    }
    if let Some(stop_reason) = &chat_response.choices[0].stop_reason {
        res["choices"][0]["stop_reason"] = json!(stop_reason);
    }

    // Optional fields for OpenAI
    if let Some(system_fingerprint) = chat_response.system_fingerprint {
        res["system_fingerprint"] = json!(system_fingerprint);
    }
    if let Some(prompt_logprobs) = chat_response.prompt_logprobs {
        res["prompt_logprobs"] = json!(prompt_logprobs);
    }

    // Optional fields for MindIE
    if let Some(prefill_time) = chat_response.prefill_time {
        res["prefill_time"] = json!(prefill_time);
    }
    if let Some(decode_time_arr) = chat_response.decode_time_arr {
        res["decode_time_arr"] = json!(decode_time_arr);
    }

    return res;
}

pub async fn completions_response_stream(
    response: Response,
    req_info: RequestInfo,
) -> Result<HttpResponse, Error> {
    let mut body_stream = response.bytes_stream();
    let req_model_name = req_info.req_model_name.clone();

    let stream = async_stream::stream! {
        while let Some(chunk) = body_stream.next().await {
            match chunk {
                Ok(bytes) => {
                    let json_str = String::from_utf8_lossy(&bytes).to_string();
                    let json_str = json_str.replace("data: [DONE]", "");
                    let json_str = json_str.trim_start_matches("data: ").trim_end();
                    let json_strs = json_str.split("data: ").collect::<Vec<&str>>();

                    for json_str in json_strs {
                        let json_str = json_str.trim();
                        if json_str.is_empty() {
                            continue;
                        }

                        let json_value = match serde_json::from_str::<Value>(&json_str) {
                            Ok(value) => value,
                            Err(err) => {
                                yield Err(format!(
                                    "Chunk failed to parse JSON from json_str: {}, Err: {}",
                                    json_str, err

                                ));
                                continue;
                            },
                        };
    
                        let chat_response: CompletionsStreamResponse = match serde_json::from_value(json_value.clone()) {
                            Ok(resp) => resp,
                            Err(err) => {
                                yield Err(format!(
                                    "Chunk failed to deserialize into CompletionsStreamResponse: {}, Err: {}",
                                    json_value, err
                                ));
                                continue;
                            },
                        };

                        // basic chunk
                        let mut chunk = json!({
                            "id": chat_response.id,
                            "model": req_model_name,
                            "created": chat_response.created,
                            "object": chat_response.object,
                        });
    
                        // Check for usage chunk
                        if let Some(usage) = &chat_response.usage {
                            let config = &*GLOBAL_CONFIG;
                            push_kafka_data(
                                req_model_name.clone(),
                                config,
                                usage.total_tokens,
                                usage.completion_tokens,
                                usage.prompt_tokens,
                                req_info.user_id.clone(),
                                req_info.service_id.clone(),
                                req_info.start_time,
                            );

                            let _total_tokens = usage.total_tokens.clone();
                        
                            if req_info.return_usage {
                                chunk["usage"] = json!(usage);
                            }
                        }
    
                        let chunk = transfer_chunk(chat_response, chunk)
                            .await
                            .unwrap();
                        let chunk_str = format!("data: {}\n\n", serde_json::to_string(&chunk).unwrap());
                        yield Ok::<Bytes, String>(Bytes::from(chunk_str));
                    }
                },
                Err(err) => {
                    yield Err(format!("Stream read error: {}", err));
                }
            }
        }
    };

    let mut stream_iter = Box::pin(stream.fuse());
    let combined_stream = async_stream::stream! {
        while let Some(chunk) = stream_iter.next().await {
            yield chunk;
        }
        let stop_str = "data: [DONE]\n\n".to_string();
        yield Ok::<Bytes, String>(Bytes::from(stop_str));
    };

    Ok(HttpResponse::Ok()
        .content_type("text/event-stream")
        .streaming(combined_stream))
}

async fn transfer_chunk(
    chat_response: CompletionsStreamResponse,
    mut chunk: Value,
) -> Result<Value, Box<dyn std::error::Error>> {
    if let Some(choices) = chat_response.choices {
        let mut choices_chunk = json!({});

        if choices.is_empty() {
            return Ok(chunk);
        }

        if let Some(index) = choices[0].clone().index {
            choices_chunk["index"] = json!(index);
        }
        if let Some(finish_reason) = choices[0].clone().finish_reason {
            choices_chunk["finish_reason"] = json!(finish_reason);
        }
        if let Some(stop_reason) = choices[0].clone().stop_reason {
            choices_chunk["stop_reason"] = json!(stop_reason);
        }
        if let Some(logprobs) = choices[0].clone().logprobs {
            choices_chunk["logprobs"] = json!(logprobs);
        }

        let mut delta_chunk = json!({});
        if let Some(delta) = choices[0].clone().delta {
            if let Some(r) = delta.role {
                delta_chunk["role"] = json!(r);
            }
            if let Some(content) = delta.content {
                delta_chunk["content"] = json!(content);
            }
            if let Some(refusal) = delta.refusal {
                delta_chunk["refusal"] = json!(refusal);
            }
            if let Some(function_call) = delta.function_call {
                delta_chunk["function_call"] = json!(function_call);
            }
            if let Some(tool_calls) = delta.tool_calls {
                delta_chunk["tool_calls"] = json!(tool_calls);
            }
            choices_chunk["delta"] = json!(delta_chunk);
        }

        chunk["choices"] = json!([choices_chunk]);
    }

    if let Some(prompt_logprobs) = chat_response.system_fingerprint {
        chunk["prompt_logprobs"] = json!(prompt_logprobs);
    }

    Ok(chunk)
}

fn push_kafka_data(
    model_name: String,
    config: &Config,
    total_tokens: u32,
    completion_tokens: u32,
    prompt_tokens: u32,
    userid: String,
    service_id: String,
    start_time: DateTime<Tz>,
) {
    let utc_time = Utc::now().with_timezone(&Shanghai);
    let end_time = Utc::now().with_timezone(&Shanghai);
    let data: Value = json!({
        "accountId": userid,
        "cloudRegionId": config.cloud_region_id,
        "modelName": model_name,
        "serviceId": service_id,
        "startTime": start_time.with_timezone(&Shanghai).to_rfc3339(),
        "endTime": end_time.to_rfc3339(),
        "totalTokens": total_tokens,
        "completionTokens": completion_tokens,
        "promptTokens": prompt_tokens,
        "time": utc_time.to_rfc3339(),
    });
    let kafka_json: String = serde_json::to_string(&data).unwrap();
    log::info!(target: "token", "{}", kafka_json);
}