use serde_json::{json, Value};
use chrono::{Utc, DateTime};
use chrono_tz::Asia::Shanghai;
use chrono_tz::Tz;
use reqwest::Client;
use std::time::Duration;

use crate::cores::models::embedding::embedding_controller::{EmbeddingRequest, 
    EmbeddingResponse, EmbeddingData, RawEmbeddingArray, Usage};
    
pub fn build_embedding_request(
    model_name: &str,
    req: &EmbeddingRequest
) -> Value {
    // let mut request_body = json!({
    //     "input": req.input,
    //     "model": model_name
    // });
    let mut request_body = json!({
        "model": model_name
    });

    // 优先使用input数组格式
    if let Some(ref input) = req.input {
        request_body["input"] = json!(input);
    } else if let Some(ref inputs) = req.inputs {
        request_body["inputs"] = json!(inputs);
    } else {
        panic!("embeddings 请求必须提供 input 或 inputs 字段");
    }

    if let Some(dimensions) = req.dimensions {
        if dimensions != 0 {
            request_body["dimensions"] = json!(dimensions);
        }
    }
    if let Some(encoding_format) = &req.encoding_format {
        if!encoding_format.is_empty() {
            request_body["encoding_format"] = json!(encoding_format);
        }
    }
    if let Some(user) = &req.user {
        if!user.is_empty() {
            request_body["user"] = json!(user);
        }
    }
    if let Some(normalize) = req.normalize {
        request_body["normalize"] = json!(normalize);
    }
    if let Some(truncate) = req.truncate {
        request_body["truncate"] = json!(truncate);
    }
    if let Some(truncation_direction) = &req.truncation_direction {
        request_body["truncation_direction"] = json!(truncation_direction);
    }
    if let Some(prompt_name) = &req.prompt_name {
        request_body["prompt_name"] = json!(prompt_name);
    }

    request_body
}

pub async fn get_embedding_response(
    request_body: Value,
    url: String,
    api_key: String
) -> Result<(EmbeddingResponse, DateTime<Tz>), String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(300))
        .connect_timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Client build failed: {}", e))?;

    let start_time = Utc::now().with_timezone(&Shanghai);
    let response = client.post(&url)
        .header("Content-Type", "application/json")
        .header("Authorization", api_key)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!("API error: {}", status));
    }

    // 读取原始JSON值
    let json_value: Value = response.json().await
        .map_err(|e| format!("Failed to parse response as JSON: {}", e))?;

    // 尝试解析为标准格式
    match serde_json::from_value::<EmbeddingResponse>(json_value.clone()) {
        Ok(resp) => {
            Ok((resp, start_time))
        },
        Err(_) => {
            // 尝试解析为数组格式
            let raw_array = serde_json::from_value::<RawEmbeddingArray>(json_value)
                .map_err(|e| format!("Unsupported response format: {}", e))?;

            // 转换为标准结构
            let data = raw_array.0.into_iter()
                .enumerate()
                .map(|(index, embedding)| EmbeddingData {
                    object: "embedding".into(),
                    embedding,
                    index,
                })
                .collect();

            Ok((EmbeddingResponse {
                object: "list".into(),
                data,
                model: request_body["input"].to_string(),
                usage: Usage {
                    prompt_tokens: 0,
                    completion_tokens: None,
                    total_tokens: 0,
                },
            }, start_time))
        }
    }
}

// pub async fn get_embedding_response(
//     request_body: Value,
//     url: String
// ) -> Result<(Response, DateTime<Tz>), String> {
//     let client = Client::builder()
//         .timeout(Duration::from_secs(300))
//         .connect_timeout(Duration::from_secs(10))
//         .build()
//         .map_err(|e| format!("Client build failed: {}", e))?;

//     let start_time = Utc::now().with_timezone(&Shanghai);
//     let response = client.post(&url)
//         .header("Content-Type", "application/json")
//         .header("Authorization", "Bearer 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")
//         .json(&request_body)
//         .send()
//         .await
//         .map_err(|e| format!("Request failed: {}", e))?;

//     if !response.status().is_success() {
//         return Err(format!("API error: {}", response.status()));
//     }

//     Ok((response, start_time))
// }