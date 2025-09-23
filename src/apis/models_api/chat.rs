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
use crate::utils::convlog::{append_conv_log, ConversationLog, AppState};
use futures::StreamExt;
use chrono::Utc;
use std::time::Instant;
use bytes::BytesMut;

use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use actix_web::body::{to_bytes, BodyStream};
use actix_http::body::{BoxBody, MessageBody};
use actix_utils::future::poll_fn;
use bytes::Bytes;



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
    state: web::Data<AppState>,
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
        _ => return Err(ErrorBadRequest(
            format!("Unsupported {} model series!",
            model_series
        ))),
    };


    // 4. 调用大模型
    // let resp = model.completions(req_body, username, userid, servicename, serviceid, user_type, user_level, pay_status, aicpid).await?;
    // ------- 下面是“会话日志”相关 --------

    // b) 最近一条用户输入
    let last_user_input: String = req_body
        .messages
        .iter()
        .rev()
        .find(|m| m.role.eq_ignore_ascii_case("user"))
        .map(|m| serde_json::to_string(&m.content).unwrap_or_default()) // ← 序列化为 String
        .unwrap_or_default();
    // 把整个 messages 转成 JSON Value
    let history_json = serde_json::to_value(&req_body.messages).unwrap_or(serde_json::Value::Null);   

    // c) 轮数（粗略估算：user出现的次数）
    let rounds = req_body
        .messages
        .iter()
        .filter(|m| m.role.eq_ignore_ascii_case("user"))
        .count() as u32;

    // d) 计时
    let start_at = Utc::now();
    let st = Instant::now();
    // -----------------------------------

    // Send the request to the model service
   let response = model
    .completions(
        req_body,                 
        username,         
        userid,
        servicename,
        serviceid,
        user_type,
        user_level,
        pay_status,
        aicpid,
    )
    .await;

    match response {
        Ok(mut resp) => {
         
            info!(
                target: "access_log",
                "{}",
                log_request(req.clone(), resp.status().as_u16(), None).await.unwrap()
            );

            // 是否流式
            let is_stream = resp
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .map(|ct| ct.starts_with("text/event-stream"))
                .unwrap_or(false);

            if !is_stream {
                // ---------- 非流式：读取 body，解析输出和 id ----------
                let status = resp.status();
                let headers = resp.headers().clone();

                let body_bytes = to_bytes(resp.into_body()).await
                    .map_err(|e| ErrorBadRequest(format!("read body failed: {}", e)))?;

                // 输出文本
                let output_text = extract_output_text_from_bytes(&body_bytes);
                // 从响应体解析 id 作为 conversation_id（若无则写空字符串）
                let conversation_id = extract_id_from_bytes(&body_bytes).unwrap_or_default();

                // 计时点放在读取完之后
                let end_at = Utc::now();
                let duration_ms = st.elapsed().as_millis() as i64;
                eprintln!(">>> about to call append_conv_log, cid={}", conversation_id);

                // 写会话日志
                let conv_log = ConversationLog {
                    conversation_id: conversation_id.clone(),
                    input: last_user_input,
                    output: output_text.clone(),
                    start_time: start_at,
                    end_time: end_at,
                    duration_ms,
                    rounds,
                    history: Some(history_json),
                };
                info!(target: "conv_log_debug",
                "nonstream about to write convlog: cid={}, body_len={}B",
                conversation_id,
                body_bytes.len()
                );

                if let Err(e) = append_conv_log(state.clone(), conv_log).await {
                    eprintln!("chat.rs: about to call append_conv_log, conv_id={}", conversation_id);
                    error!(target:"conv_log","append_conv_log_failed (nonstream): {}", e);
                   
                } else {
                    info!(target:"conv_log_debug","nonstream convlog written");
                }

                // 重建响应返回
                let mut builder = HttpResponse::build(status);
                for (k, v) in headers.iter() {
                    builder.insert_header((k.clone(), v.clone()));
                }
                let rebuilt_resp = builder.body(body_bytes);
                Ok(rebuilt_resp)
            } else {
                // ---------- 流式：tee 转发分片，同时从分片解析 id；在流结束时写日志 ----------
        
                let status = resp.status();
                let headers = resp.headers().clone();

                // upstream 原始 body
                let mut body: BoxBody = resp.into_body();

                // tee 的发送端/接收端
                let (tx, rx) = mpsc::channel::<Result<Bytes, actix_web::Error>>(32);

                // 克隆必要现场用于日志写入
                let state_cloned = state.clone();
                let last_user_input_cloned = last_user_input.clone();
                let history_json_cloned = history_json.clone();
                let rounds_cloned = rounds;
                let start_at_cloned = start_at;
                let st_cloned = st;

                // 会话 id 保存（只来源于响应体）
                let cid_holder = std::sync::Arc::new(std::sync::Mutex::new(None::<String>));
                let cid_holder2 = cid_holder.clone();

                actix_web::rt::spawn(async move {
                    loop {
                        // 通过 poll_fn 等待下一块 bytes
                        let polled = poll_fn(|cx| std::pin::Pin::new(&mut body).poll_next(cx)).await;
                        match polled {
                            Some(Ok(bytes)) => {
                                // 解析 id（若尚未获得）
                                if cid_holder2.lock().unwrap().is_none() {
                                    if let Some(id) = extract_id_from_sse_or_jsonl_bytes(bytes.as_ref()) {
                                        *cid_holder2.lock().unwrap() = Some(id);
                                    }
                                }
                                // 转发给客户端
                                if tx.send(Ok(bytes)).await.is_err() {
                                    // 下游已关闭
                                    break;
                                }
                            }
                            Some(Err(e)) => {
                                let _ = tx.send(Err(actix_web::error::ErrorBadGateway(e))).await;
                                break;
                            }
                            None => {
                                // 流结束：写会话日志（只用响应体提取的 id；若没拿到则空字符串）
                                let end_at = chrono::Utc::now();
                                let duration_ms = st_cloned.elapsed().as_millis() as i64;
                                let conversation_id = cid_holder2.lock().unwrap().clone().unwrap_or_default();

                                let conv_log = ConversationLog {
                                    conversation_id: conversation_id.clone(),
                                    input: last_user_input_cloned,
                                    output: "<streaming>".to_string(),
                                    start_time: start_at_cloned,
                                    end_time: end_at,
                                    duration_ms,
                                    rounds: rounds_cloned,
                                    history: Some(history_json_cloned),
                                };
                                info!(
                                target:"conv_log_debug",
                                "stream about to write convlog (on EOF): cid={}",
                                conversation_id
                                );
                                if let Err(e) = append_conv_log(state_cloned, conv_log).await {
                                    error!(target:"conv_log","append_conv_log_failed (stream): {}", e);
                                } else {
                                    info!(target:"conv_log_debug","stream convlog written");
                                }

                                // 正常结束
                                break;
                            }
                        }
                    }
                });

                // 返回代理后的流式响应；外层不再写日志
                let mut builder = HttpResponse::build(status);
                for (k, v) in headers.iter() {
                    builder.insert_header((k.clone(), v.clone()));
                }
                let proxied = builder.body(BodyStream::new(ReceiverStream::new(rx)));
                Ok(proxied)
        }
    }
        
        Err(err) => {
            error!(
                target: "error_log", "{}", log_request(req.clone(), err.as_response_error().status_code().as_u16(), Some(&format!("{}", err)))
                .await.unwrap() );

            // 失败也记一条“会话日志”
                let end_at = Utc::now();
                let duration_ms = st.elapsed().as_millis() as i64;
                let conv_log = ConversationLog {
                    conversation_id: String::new(), // 失败没有 id 就留空
                    input: last_user_input,
                    output: format!("ERROR: {}", err),
                    start_time: start_at,
                    end_time: end_at,
                    duration_ms,
                    rounds,
                    history: Some(history_json),
                };
                if let Err(e) = append_conv_log(state.clone(), conv_log).await {
                    error!(target:"conv_log","append_conv_log_failed (error branch): {}", e);
                }

               
            Err(err)
        }
    }
}





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
// 非流式：从完整 JSON body 中提取 id
fn extract_id_from_bytes(bytes: &[u8]) -> Option<String> {
    let v: serde_json::Value = serde_json::from_slice(bytes).ok()?;
    // 顶层 id
    if let Some(id) = v.get("id").and_then(|x| x.as_str()) {
        return Some(id.to_string());
    }
    // 兼容 data.id
    if let Some(id) = v.get("data").and_then(|d| d.get("id")).and_then(|x| x.as_str()) {
        return Some(id.to_string());
    }
    // 兼容 choices[0].id
    if let Some(id) = v.get("choices").and_then(|c| c.get(0)).and_then(|c0| c0.get("id")).and_then(|x| x.as_str()) {
        return Some(id.to_string());
    }
    None
}
fn extract_output_text_from_bytes(bytes: &[u8]) -> String {
    if let Ok(val) = serde_json::from_slice::<serde_json::Value>(bytes) {
        if let Some(s) = val.get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|t| t.as_str())
        {
            return s.to_string();
        }
        if let Some(s) = val.get("output").and_then(|v| v.as_str()) {
            return s.to_string();
        }
        if let Some(s) = val.get("text").and_then(|v| v.as_str()) {
            return s.to_string();
        }
        return val.to_string();
    }
    String::from_utf8_lossy(bytes).to_string()
}
/// 既兼容 SSE 帧（以 "data: " 起头的行），也兼容纯 JSON/JSONL 分片
fn extract_id_from_sse_or_jsonl_bytes(bytes: &[u8]) -> Option<String> {
    // 1) 先尝试按 SSE 帧逐行解析
    for line in bytes.split(|&c| c == b'\n') {
        let line = trim_crlf(line);
        if line.starts_with(b"data:") {
            // "data:" 后可能有一个空格
            let payload = line.strip_prefix(b"data:").unwrap_or(line);
            let payload = trim_leading_space(payload);
            if let Some(id) = parse_id_from_json_slice(payload) {
                return Some(id);
            }
        }
    }
    // 2) 再尝试把整块当成一段 JSON/JSONL
    if let Some(id) = parse_id_from_json_slice(bytes) {
        return Some(id);
    }
    None
}

fn parse_id_from_json_slice(slice: &[u8]) -> Option<String> {
    // slice 可能包含多段 JSON（JSONL），尝试逐段解析
    // 简化策略：先整体 parse；失败就按 '\n' 拆开多段尝试
    if let Ok(v) = serde_json::from_slice::<serde_json::Value>(slice) {
        if let Some(id) = pick_id(&v) { return Some(id); }
    } else {
        for part in slice.split(|&c| c == b'\n') {
            if part.is_empty() { continue; }
            if let Ok(v) = serde_json::from_slice::<serde_json::Value>(part) {
                if let Some(id) = pick_id(&v) { return Some(id); }
            }
        }
    }
    None
}

fn pick_id(v: &serde_json::Value) -> Option<String> {
    // 常见位置：顶层 id、data.id、choices[0].id
    if let Some(s) = v.get("id").and_then(|x| x.as_str()) {
        return Some(s.to_string());
    }
    if let Some(s) = v.get("data").and_then(|d| d.get("id")).and_then(|x| x.as_str()) {
        return Some(s.to_string());
    }
    if let Some(s) = v.get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c0| c0.get("id"))
        .and_then(|x| x.as_str())
    {
        return Some(s.to_string());
    }
    None
}

fn trim_crlf(s: &[u8]) -> &[u8] {
    let mut end = s.len();
    while end > 0 && (s[end-1] == b'\r' || s[end-1] == b'\n') { end -= 1; }
    &s[..end]
}
fn trim_leading_space(s: &[u8]) -> &[u8] {
    let mut i = 0;
    while i < s.len() && s[i].is_ascii_whitespace() { i += 1; }
    &s[i..]
}
