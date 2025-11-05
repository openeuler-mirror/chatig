use actix_web::web;
use serde_json::{Value, json};
use reqwest::Response;
use actix_web::{HttpResponse};
use crate::cores::models::rerank::rerank_controller::{StdRerankRequest, LlamaBoxRerankResponse, 
    VLLMRerankRespnse, MindieRerankResponse};


// Build request body based on inference engine type and request body
// Currently supported inference engines: LlamaBox, VLLM, Mindie, Alibaba
pub fn get_request_body(
    req_body: web::Json<StdRerankRequest>,
    model_name: String,
    engines_type: String,
) -> Result<Value, String> {
    match engines_type.as_str() {
        "llamabox" => {
            let mut request_body = json!({
                "model": model_name,
                "query": req_body.query,
                "documents": req_body.documents
            });

            if let Some(top_n) = req_body.top_n {
                request_body["top_n"] = json!(top_n);
            }

            Ok(request_body)
        }
        "vllm" => {
            let request_body = json!({
                "model": model_name,
                "query": req_body.query,
                "documents": req_body.documents
            });
            Ok(request_body)
        }
        "mindie" => {
            let request_body = json!({
                "query": req_body.query,
                "texts": req_body.documents
            });

            Ok(request_body)
        }
        "alibaba" => {
            let mut request_body = json!({
                "model": model_name,
                "query": req_body.query,
                "documents": req_body.documents
            });

            if let Some(parameters) = req_body.parameters {
                if let Some(top_n) = parameters.top_n {
                    request_body["top_n"] = json!(top_n);
                }
                if let Some(return_documents) = parameters.return_documents {
                    request_body["return_documents"] = json!(return_documents);
                }
            }

            Ok(request_body)
        }
        _ => {
            Err(format!("Unsupported engine type: {}", engines_type))
        }
    }
}


// Build standard response body based on the inference engine type
pub async fn build_std_rerank_response(
    response: Response,
    engine_type: String,
) -> Result<Vec<Value>, String> {
    let response_text = response
        .text()
        .await
        .map_err(|err| format!("Failed to read response: {}", err))?;

    let json_value: Value = serde_json::from_str(&response_text).map_err(|err| {
        format!("Failed to parse unescaped JSON: {}, {}", err, response_text)
    })?;
    // 快速路径：如果根就是数组（标准化后的 [{index, score}]），直接返回
    if let Some(arr) = json_value.as_array() {
        return Ok(arr.clone());
    }
    
    // 一个小工具：从各种可能的包裹层里挖出 results 数组
    fn extract_results_array(root: &Value) -> Option<&Vec<Value>> {
        // 1) 兼容 { code, message, body: {...} }
        let body = root.get("body").unwrap_or(root);

        // 2) 兼容 { output: { results: [...] } } 或 { results: [...] }
        let results_val = body
            .pointer("/output/results")
            .or_else(|| body.get("results"));

        results_val.and_then(|v| v.as_array())
    }
    let arr = extract_results_array(&json_value).ok_or_else(|| {
    format!("Missing results array in response: {}", json_value)
    })?;
    let mut results = Vec::new();
    match engine_type.as_str() {
        "vllm" | "llamabox" | "alibaba" => {
            let arr = extract_results_array(&json_value).ok_or_else(|| {
                format!("Missing results array in response: {}", json_value)
            })?;

            for (i, item) in arr.iter().enumerate() {
                // index：没有就用序号兜底
                let idx = item.get("index")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(i as u64) as u32;

                // score：兼容 relevance_score / score
                let score = item.get("relevance_score")
                    .and_then(|v| v.as_f64())
                    .or_else(|| item.get("score").and_then(|v| v.as_f64()))
                    .unwrap_or(0.0) as f32;
                
                results.push(json!({ "index": idx, "score": score }));
            }
        }
        "mindie" => {
            let mindie_response: Vec<MindieRerankResponse> = serde_json::from_value(json_value).map_err(|err| {
                format!("Failed to deserialize the Response: {}", err)
            })?;

            for result in mindie_response {
                results.push(json!({
                    "index": result.index,
                    "score": result.score
                }));
            }
        }
        _ => {
            return Err(format!("Unsupported engine type: {}", engine_type));
        }
    }

    Ok(results)

}




