use actix_web::web;
use serde_json::{Value, json};
use reqwest::Response;

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

    let mut results = Vec::new();
    match engine_type.as_str() {
        "llamabox" => {
            // Parse response into LlamaBoxRerankResponse
            let llamabox_response: LlamaBoxRerankResponse = serde_json::from_value(json_value).map_err(|err| {
                format!("Failed to deserialize the Response: {}", err)
            })?;

            for result in llamabox_response.results {
                results.push(json!({
                    "index": result.index,
                    "score": result.relevance_score
                }));
            }
        }
        "vllm" => {
            let vllm_response: VLLMRerankRespnse = serde_json::from_value(json_value).map_err(|err| {
                format!("Failed to deserialize the Response: {}", err)
            })?;

            for result in vllm_response.results {
                results.push(json!({
                    "index": result.index,
                    "score": result.relevance_score
                }));
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




