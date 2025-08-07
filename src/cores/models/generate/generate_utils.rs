use actix_web::{web, HttpResponse, Error};
use reqwest::{Client, Response};
use std::time::Duration;
use actix_web::error::ErrorInternalServerError;
use serde_json::{Value, json};
use bytes::Bytes;
use log;
use std;
use chrono_tz::Asia::Shanghai;
use chrono::{Utc, DateTime};
use chrono_tz::Tz;
use futures_util::StreamExt;
use crate::meta::services::traits::Services;

use crate::cores::models::chat::chat_utils::RequestInfo;
use crate::cores::models::generate::generate_controller::{
    Input,
    InputData,
    GenerateResponse,
    GenerateStreamResponse,
    GenerateRequest,
};
use crate::GLOBAL_CONFIG;
use crate::configs::settings::Config;

pub async fn get_generate_request_body(
    req_body: web::Json<GenerateRequest>,
) -> Result<Value, String> {

    let mut request_body = json!({});

    match &req_body.inputs {
        Input::Single(text) => {
            request_body["inputs"] = json!(text);
        }
        Input::Multi(parts) => {
            let mut inputs_array = Vec::new();
            for part in parts {
                let mut input_part = json!({
                    "type": part.input_type
                });
                match &part.data {
                    InputData::Text { text } => {
                        input_part["text"] = json!(text);
                    }
                    InputData::Image { image_url } => {
                        input_part["image_url"] = json!(image_url);
                    }
                    InputData::Video { video_url } => {
                        input_part["video_url"] = json!(video_url);
                    }
                    InputData::Audio { audio_url } => {
                        input_part["audio_url"] = json!(audio_url);
                    }
                }
                inputs_array.push(input_part);
            }
            request_body["inputs"] = json!(inputs_array);
        }
    }
    let default_parameters = json!({
        "decoder_input_details": false,
        "details": false,
        "max_new_tokens": 20,
        "repetition_penalty": 1.0,
        "return_full_text": false,
        "seed": null,
        "temperature": 1.0,
        "truncate": null,
        "typical_p": 1.0,
        "watermark": false,
        "stop": null,
        "adapter_id": "None"
    });
    // 处理参数
    if let Some(parameters) = &req_body.parameters {
       let mut merged_parameters = default_parameters.clone();
        if let Some(value) = parameters.decoder_input_details {
            merged_parameters["decoder_input_details"] = json!(value);
        }
        if let Some(value) = parameters.details {
            merged_parameters["details"] = json!(value);
        }
        if let Some(value) = parameters.do_sample {
            merged_parameters["do_sample"] = json!(value);
        }
        if let Some(value) = parameters.max_new_tokens {
            merged_parameters["max_new_tokens"] = json!(value);
        }
        if let Some(value) = parameters.repetition_penalty {
            merged_parameters["repetition_penalty"] = json!(value);
        }
        if let Some(value) = parameters.return_full_text {
            merged_parameters["return_full_text"] = json!(value);
        }
        if let Some(value) = parameters.seed {
            merged_parameters["seed"] = json!(value);
        }
        if let Some(value) = parameters.temperature {
            merged_parameters["temperature"] = json!(value);
        }
        if let Some(value) = parameters.top_k {
            merged_parameters["top_k"] = json!(value);
        }
        if let Some(value) = parameters.top_p{
            merged_parameters["top_p"] = json!(value);
        }
        if let Some(value) = parameters.truncate {
            merged_parameters["truncate"] = json!(value);
        }
        if let Some(value) = parameters.typical_p {
            merged_parameters["typical_p"] = json!(value);
        }
        if let Some(value) = parameters.watermark {
            merged_parameters["watermark"] = json!(value);
        }
        if let Some(value) = &parameters.stop {
            merged_parameters["stop"] = json!(value);
        }
        if let Some(value) = &parameters.adapter_id {
            merged_parameters["adapter_id"] = json!(value);
        }
        request_body["parameters"] = merged_parameters;
    } else {
        request_body["parameters"] = default_parameters;
    }
    println!("Generated request body: {}", request_body);
    Ok(request_body)

}

//多次重试请求接口
pub async fn get_generate_response(
    request_body: Value,
    service: &mut Services,
    is_stream: bool
) -> Result<(Response, DateTime<Tz>), String> {
    let config = &*GLOBAL_CONFIG;
    let client = Client::builder()
        .timeout(Duration::from_secs(config.request_timeout))
        .connect_timeout(Duration::from_secs(config.connection_timeout))
        .build()
        .map_err(|err| format!("Failed to build client: {}", err))?;
    
    //处理原始url拼接generate
    let url = service.url.clone();
    let generate_url = if is_stream {
        url.replace("/v1/chat/completions", "/generate_stream")
    } else {
        url.replace("/v1/chat/completions", "/generate")
    };

    let mut retry_count = 0;
    loop {
        let start_time = Utc::now().with_timezone(&Shanghai);
        match client
            .post(generate_url.clone())
            .header("Content-Type", "application/json")
            .header("Authorization", service.api_key.clone())
            .json(&request_body)
            .send()
            .await
        {
            Ok(resp) => match resp.status() {
                status if status.is_success() => {
                    return Ok((resp, start_time));
                }
                status if status.is_server_error() => {
                    retry_count += 1;
                    if retry_count <= config.request_retry {
                        println!(
                            "Request failed for service id: {}: {}. Retrying ({}/{}). ",
                            status,
                            service.id,
                            retry_count,
                            config.request_retry
                        );
                        continue;
                    } else {
                        return Err(format!("Request failed: {} generate_url request failed, status: {}", generate_url, status));
                    }
                }
                status => {
                    return Err(format!("Request failed: {} generate_url request failed, status: {}", generate_url, status));
                }
            },
            Err(_) => {
                retry_count += 1;
                if retry_count <= config.request_retry {
                    println!(
                        "Request failed for service id: {}. Retrying ({}/{})",
                        service.id,
                        retry_count,
                        config.request_retry
                    );
                    continue;
                } else {
                    return Err(format!("Request failed: {} generate_url request failed.", generate_url));
                }
            }
        };
    }
}

//非流式返回处理
pub async fn generate_response_non_stream(
    response: Response,
    req_info: RequestInfo,
) -> Result<HttpResponse, Error> {
    
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

    let generate_response: GenerateResponse = serde_json::from_value(json_value).map_err(|err| {
        ErrorInternalServerError(format!(
            "Failed to deserialize into GenerateResponse: {}",
            err
        ))
    })?;


    let req_model_name = req_info.req_model_name.clone();
    let res = build_res(generate_response.clone());

    let config = &*GLOBAL_CONFIG;

    let (generated_tokens, prompt_tokens) = get_generate_token(generate_response);
    let total_tokens = generated_tokens+prompt_tokens;
    push_kafka_data(req_model_name.clone(), config, total_tokens, generated_tokens, 
        prompt_tokens, req_info.user_id.clone(), req_info.service_id.clone(), req_info.start_time);


    Ok(HttpResponse::Ok().json(res))
}


fn get_generate_token(generate_response: GenerateResponse) -> (u32, u32){
     if let Some(details) = generate_response.details {
        (details.generated_tokens, details.prompt_tokens)
    } else {
        // 如果 details 是 None，返回默认值或错误处理
        // 这里返回 (0, 0) 作为默认值
        (0, 0)
    }
}

//非流式构造返回
fn build_res(generate_response: GenerateResponse) -> Value {
    let mut res = json!({
        "generated_text": generate_response.generated_text,
    });

    if let Some(details) = generate_response.details {
        let mut details_json = json!({
            "finish_reason": details.finish_reason,
            "generated_tokens": details.generated_tokens,
            "prompt_tokens": details.prompt_tokens,
            "seed": details.seed,
        });

        // 添加 tokens
        let tokens_json: Vec<Value> = details.tokens.iter().map(|token| {
            json!({
                "id": token.id,
                "logprob": token.logprob,
                "special": token.special,
                "text": token.text,
            })
        }).collect();
        details_json["tokens"] = json!(tokens_json);

        // 添加 prefill（如果存在）
        if let Some(prefill) = details.prefill {
            let prefill_json: Vec<Value> = prefill.iter().map(|token| {
                json!({
                    "id": token.id,
                    "logprob": token.logprob,
                    "special": token.special,
                    "text": token.text,
                })
            }).collect();
            details_json["prefill"] = json!(prefill_json);
        }

        res["details"] = details_json;
    }

    res
}

pub async fn generate_response_stream(
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
                    let json_str = json_str.trim_end();
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
    
                        let generate_response: GenerateStreamResponse = match serde_json::from_value(json_value.clone()) {
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
                        let chunk = json!({});
    
                        // Check for usage chunk
                        if let Some(details) = &generate_response.details {
                            let config = &*GLOBAL_CONFIG;
                            push_kafka_data(
                                req_model_name.clone(),
                                config,
                                details.generated_tokens + details.prompt_tokens,
                                details.generated_tokens,
                                details.prompt_tokens,
                                req_info.user_id.clone(),
                                req_info.service_id.clone(),
                                req_info.start_time,
                            );

                            let _total_tokens = details.generated_tokens + details.prompt_tokens;
                        }
    
                        let chunk = get_chunk_data(generate_response, chunk)
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
        //无需结束标记
    };

    Ok(HttpResponse::Ok()
        .content_type("text/event-stream")
        .streaming(combined_stream))
}

async fn get_chunk_data(
    generate_response: GenerateStreamResponse,
    mut chunk: Value,
) -> Result<Value, Box<dyn std::error::Error>> {

    chunk["token"] = json!({
        "id": generate_response.token.id,
        "text": generate_response.token.text,
        "logprob": generate_response.token.logprob,
        "special": generate_response.token.special,
    });

    chunk["generated_text"] = json!(generate_response.generated_text);

    if let Some(details) = generate_response.details {
        chunk["details"] = json!({
            "prompt_tokens": details.prompt_tokens,
            "finish_reason": details.finish_reason,
            "generated_tokens": details.generated_tokens,
            "seed": details.seed,
        });
    }else{
        chunk["details"] = json!(generate_response.details)
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
