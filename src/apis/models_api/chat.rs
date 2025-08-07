use actix_web::{
    get, post, web, Error, HttpResponse, Responder, HttpRequest, HttpMessage,
};
use actix_web::error::ErrorBadRequest;
use log::{info, error};
use std::collections::HashMap;

use crate::cores::models::chat::chat_controller::{ChatCompletionRequest, Completions};
use crate::cores::models::chat::support_engines::std_chat;
use crate::utils::log::log_request;
use crate::configs::settings::GLOBAL_CONFIG;

pub fn configure(
    cfg: &mut web::ServiceConfig,
) {
    cfg.service(
        web::scope("/v1/chat")
            .service(health)
            .service(completions)
            .service(completions2),
    );
}

/// 获取健康检查
///
/// # 健康检查
/// - get /health
/// - 如果接口没有问题，则返回 "OK"
#[utoipa::path(
    get,
    path = "/health",
    responses((status = 200, body = String))
)]
#[get("/health")]
pub async fn health() -> impl Responder {
    "OK"
}

/// 定义接口层，调用大模型 completions 方法
struct LLM {
    model: Box<dyn Completions>,
}

impl LLM {
    fn new(model: Box<dyn Completions>) -> Self {
        LLM { model }
    }

    async fn completions(
        &self,
        req_body: web::Json<ChatCompletionRequest>,
        user_name: String,
        user_id: String,
        service_name: String,
        service_id: String,
        user_type: String,
        user_level: String,
        pay_status: String,
        aicpid: String,
    ) -> Result<HttpResponse, Error> {
        self.model.completions(req_body, user_name, user_id, service_name, service_id, user_type, user_level, pay_status, aicpid).await
    }
}

#[utoipa::path(
    post,
    path = "/v1/chat/completions",
    request_body = ChatCompletionRequest,
    responses(
        (status = 200, body = CompletionsResponse),
        (status = 400, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    )
)]
#[post("/completions")]
pub async fn completions(
    req: HttpRequest,
    req_body: web::Json<ChatCompletionRequest>,
) -> Result<impl Responder, Error> {
    // 1. Validate that required fields exist in the request data
    if req_body.model.is_empty() || req_body.messages.is_empty() {
        let error_response = "Invalid request: model or messages cannot be empty.";
        return Ok(HttpResponse::BadRequest().json(error_response));
    }

    // 2. Get the user ID and service name from the request extensions
    let _config = &*GLOBAL_CONFIG;
    let username = req.extensions().get::<HashMap<&str, String>>().and_then(|data| data.get("user_name").cloned()).unwrap_or_else(|| "".to_string());
    let userid = req.extensions().get::<HashMap<&str, String>>().and_then(|data| data.get("user_id").cloned()).unwrap_or_else(|| "".to_string());
    let servicename = req.extensions().get::<HashMap<&str, String>>().and_then(|data| data.get("service_name").cloned()).unwrap_or_else(|| "".to_string());
    let serviceid = req.extensions().get::<HashMap<&str, String>>().and_then(|data| data.get("service_id").cloned()).unwrap_or_else(|| "".to_string());
    let user_type = req.extensions().get::<HashMap<&str, String>>().and_then(|data| data.get("user_type").cloned()).unwrap_or_else(|| "".to_string());
    let user_level = req.extensions().get::<HashMap<&str, String>>().and_then(|data| data.get("user_level").cloned()).unwrap_or_else(|| "".to_string());
    let pay_status = req.extensions().get::<HashMap<&str, String>>().and_then(|data| data.get("pay_status").cloned()).unwrap_or_else(|| "".to_string());
    let aicpid = req.extensions().get::<HashMap<&str, String>>().and_then(|data| data.get("aicpid").cloned()).unwrap_or_else(|| "[]".to_string());

    // 3. Parse the model name and series from the model field (e.g., Qwen/Qwen2.5-7B-Instruct)
    let (model_series, model_name) = match req_body.model.split_once('/') {
        Some((series, name)) => (series, name),
        None => ("std", req_body.model.as_str()),
    };

    // Call the underlying API and return a unified data format
    let model: LLM = match model_series {
        "Qwen" | "GLM" | "meta-llama" | "Bailian" | "deepseek-ai" | "std" => LLM::new(Box::new(
            std_chat::StdChatModel {
                active_model: model_name.to_string(),
            },
        )),
        _ => return Err(ErrorBadRequest(format!("Unsupported {} model series!", model_series))),
    };

    // Send the request to the model service
    let response = model.completions(req_body, username, userid, servicename, serviceid, user_type, user_level, pay_status, aicpid).await;
    match response {
        Ok(resp) => {
            info!(target: "access_log", "{}", log_request(req.clone(), resp.status().as_u16(), None).await.unwrap());
            Ok(resp)
        }
        Err(err) => {
            error!(
                target: "error_log", "{}", log_request(req.clone(), err.as_response_error().status_code().as_u16(), Some(&format!("{}", err)))
                .await.unwrap()
            );
            Err(err)
        }
    }
}



use bytes::BytesMut;
use futures::StreamExt; // 确保引入 StreamExt

#[post("/completions_debug")]
pub async fn completions2(_req: HttpRequest, mut payload: web::Payload) -> Result<impl Responder, Error> {
    // 获取原始请求体
    let mut body = BytesMut::new();
    while let Some(chunk) = payload.next().await {
        body.extend_from_slice(&chunk?);
    }

    // 将请求体转换为字符串并打印
    let body_str = String::from_utf8(body.to_vec())
        .unwrap_or_else(|_| "Failed to decode body as UTF-8".to_string());
    println!("Received raw request body: {}\n", body_str);

    // 尝试反序列化（可选，用于确认问题）
    let chat_request: ChatCompletionRequest = serde_json::from_str(&body_str)
        .map_err(|e| {
            println!("Deserialization error: {}", e);
            actix_web::error::ErrorBadRequest(e)
        })?;

    println!("Received request: {:?}\n", chat_request);

    // 正常处理逻辑
    Ok(HttpResponse::Ok().json("Received"))
}