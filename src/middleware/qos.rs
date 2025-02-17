use actix_service::{Service, Transform};
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    Error,
    HttpMessage,
};
use actix_web::error::ErrorBadRequest;
use futures::future::{ok, LocalBoxFuture, Ready};
use serde::Deserialize;
use serde::Serialize;
use bytes::Bytes;
use futures::StreamExt;
use bytes::BytesMut;
use std::sync::Arc;
use futures::stream::once;
use futures_core::Stream;
use tokio::join;
use log::error;

use actix_web::error::PayloadError;
// 引入 BoxedPayloadStream 定义
pub type BoxedPayloadStream = std::pin::Pin<Box<dyn futures_core::Stream<Item = Result<Bytes, PayloadError>>>>;

use crate::{configs::settings::GLOBAL_CONFIG, cores::control::model_limits::LimitsManager};

// 假设的 ChatCompletionRequest 结构体
#[derive(Deserialize)]
struct ChatCompletionRequest {
    model: String,
    // 可以添加其他字段
}

// middleware structure
pub struct Qos {
}

// The constructor function
impl Qos {
    pub fn new() -> Self {
        Self {}
    }
}

// Transform trait implementation, used for middleware wrapping
impl<S, B> Transform<S, ServiceRequest> for Qos
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
    // B: 'static + actix_web::body::MessageBody,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = QosMiddleware<Arc<S>>; // 使用 Arc 包装 S
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(QosMiddleware {
            service: Arc::new(service),
        })
    }
}

// Middleware implementation
pub struct QosMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for QosMiddleware<Arc<S>>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let payload = req.take_payload();
        let body = BytesMut::new();

        async fn read_payload(mut payload: impl Stream<Item = Result<Bytes, PayloadError>> + Unpin, mut body: BytesMut) -> Result<(ChatCompletionRequest, BytesMut), Error> {
            while let Some(chunk) = payload.next().await {
                let chunk = chunk.map_err(|e| {
                    actix_web::error::ErrorInternalServerError(format!("Failed to read payload: {}", e))
                })?;
                body.extend_from_slice(&chunk);
            }
            let body_clone = body.clone();
            let chat_request = serde_json::from_slice::<ChatCompletionRequest>(&body_clone)
               .map_err(|e| ErrorBadRequest(format!("Failed to parse JSON: {}", e)))?;
            Ok((chat_request, body_clone))
        }

        let read_payload_fut = async move {
            read_payload(payload, body).await
        };

        let service = self.service.clone(); // Arc 实现了 Clone 特性

        let config = &*GLOBAL_CONFIG;
        let coil_enabled = config.coil_enabled;
        let auth_remote_enabled = config.auth_remote_enabled;

        let fut = async move {
            let (chat_request, body_clone) = read_payload_fut.await?;
            let model = chat_request.model;

            // 将请求体重新放回 ServiceRequest
            let body_bytes = Bytes::from(body_clone);
            let stream = once(async { Ok::<_, PayloadError>(body_bytes) });
            let boxed_stream: BoxedPayloadStream = Box::pin(stream);
            let payload = actix_web::dev::Payload::from(boxed_stream);
            req.set_payload(payload);

            if coil_enabled && auth_remote_enabled {
                let userid = req.extensions().get::<String>().cloned().unwrap_or_else(|| "".to_string());

                let userid_clone = userid.clone();
                let model_clone = model.clone();
                let (valid_tokens, valid) = join!(
                    throttled(userid_clone, model_clone),
                    query_and_consume(userid, model)
                );
                let valid_tokens = valid_tokens?;
                let valid = valid?;

                if valid && valid_tokens {
                    service.call(req).await
                } else {
                    Err(actix_web::error::ErrorTooManyRequests("Throttle for request"))
                }       
            } else {
                    service.call(req).await
            }
        };

        Box::pin(fut)
    }
}

#[derive(Deserialize)]
struct ResponseData {
    throttled: bool,
    backoff_ns: u64,
}

// 定义请求体的结构体
#[derive(Serialize)]
struct RequestBody {
    user: String,
    item: String,
    request_amount: String,
    limit: String,
}

async fn query_and_consume(apikey: String, model: String) -> Result<bool, Error> {
    let client: reqwest::Client = reqwest::Client::new();

    let config = &*GLOBAL_CONFIG;
    let ip = &config.coil_ip;

    let limit_manager = LimitsManager::default();
    let limits = limit_manager.get_limits_object(&model)
        .await
        .map_err(|e| ErrorBadRequest(format!("Failed to get model limits: {}", e)))?;
    let limits = match limits {
        Some(limits) => limits,
        None => return Err(ErrorBadRequest(format!("{} model is not supported", model))),
    };

    // 修改请求路径
    let url = format!("http://{}/query_and_consume", ip);

    // 构建请求体
    let request_body = RequestBody {
        user: apikey,
        item: model,
        request_amount: "1".to_string(),
        limit: limits.max_requests,
    };

    // 发送 POST 请求并带上请求体
    let response = match client.post(url)
       .json(&request_body)
       .send()
       .await {
        Ok(resp) => resp,
        Err(err) => {
            // 记录错误日志
            error!(target: "error_log", "{}", err);
            return Ok(true);
        }
    };

    let body_text = match response.text().await {
        Ok(text) => text,
        Err(err) => {
            return Err(actix_web::error::ErrorInternalServerError(format!(
                "Failed to read response body: {}",
                err
            )))
        }
    };

    if body_text.trim().is_empty() {
        return Ok(true);
    }

    if body_text.trim() == "{}" {
        // 空对象表示未限流
        return Ok(true);
    }

    let body = match serde_json::from_str::<ResponseData>(&body_text) {
        Ok(body) => body,
        Err(err) => {
            return Err(actix_web::error::ErrorInternalServerError(format!(
                "Failed to parse response: {}",
                err
            )))
        }
    };

    Ok(!body.throttled)
}

// 请求消耗的token
#[derive(Serialize)]
struct RequestConsumeBody {
    user: String,
    item: String,
    request_amount: String,
}

#[derive(Deserialize)]
struct ResponseConsumeData {
    status: String,
}

pub async fn consume(apikey: String, model: String, tokens: u32) -> Result<String, Error> {
    let client: reqwest::Client = reqwest::Client::new();

    let config = &*GLOBAL_CONFIG;
    let ip = &config.coil_ip;

    // 修改请求路径
    let url = format!("http://{}/consume", ip);

    // 用户和rpm的不一样
    let tokens_apikey = format!("tokens{}", apikey);

    // 构建请求体
    let request_body = RequestConsumeBody {
        user: tokens_apikey,
        item: model,
        request_amount: tokens.to_string(),
    };

    // 发送 POST 请求并带上请求体
    let response = match client.post(url)
       .json(&request_body)
       .send()
       .await {
        Ok(resp) => resp,
        Err(err) => {
            // 记录错误日志
            error!(target: "error_log", "{}", err);
            return Ok("success".to_string());
        }
    };

    let body_text = match response.text().await {
        Ok(text) => text,
        Err(err) => {
            return Err(actix_web::error::ErrorInternalServerError(format!(
                "Failed to read response body: {}",
                err
            )))
        }
    };

    let body = match serde_json::from_str::<ResponseConsumeData>(&body_text) {
        Ok(body) => body,
        Err(err) => {
            return Err(actix_web::error::ErrorInternalServerError(format!(
                "Failed to parse response: {}",
                err
            )))
        }
    };

    Ok(body.status)
}

// 测试token是否达到阈值
pub async fn throttled(apikey: String, model: String) -> Result<bool, Error> {
    let client: reqwest::Client = reqwest::Client::new();

    let config = &*GLOBAL_CONFIG;
    let ip = &config.coil_ip;

    // 修改请求路径
    let url = format!("http://{}/throttled", ip);

    let limit_manager = LimitsManager::default();
    let limits = limit_manager.get_limits_object(&model)
        .await
        .map_err(|e| ErrorBadRequest(format!("Failed to get model limits: {}", e)))?;
    let limits = match limits {
        Some(limits) => limits,
        None => return Err(ErrorBadRequest(format!("{} model is not supported", model))),
    };

    // 用户和rpm的不一样
    let tokens_apikey = format!("tokens{}", apikey);

    // 构建请求体
    let request_body = RequestBody {
        user: tokens_apikey,
        item: model,
        request_amount: "100".to_string(),
        limit: limits.max_tokens,
    };

    // 发送 POST 请求并带上请求体
    let response = match client.post(url)
        .json(&request_body)
        .send()
        .await {
         Ok(resp) => resp,
         Err(err) => {
            // 记录错误日志
            error!(target: "error_log", "{}", err);
            return Ok(true);
        }
     };
 
     let body_text = match response.text().await {
         Ok(text) => text,
         Err(err) => {
             return Err(actix_web::error::ErrorInternalServerError(format!(
                 "Failed to read response body: {}",
                 err
             )))
         }
     };
 
     if body_text.trim().is_empty() {
         return Ok(true);
     }
  
     if body_text.trim() == "{}" {
         // 空对象表示未限流
         return Ok(true);
     }

     let body = match serde_json::from_str::<ResponseData>(&body_text) {
         Ok(body) => body,
         Err(err) => {
             return Err(actix_web::error::ErrorInternalServerError(format!(
                 "Failed to parse response: {}",
                 err
             )))
         }
     };
     let _ = body.backoff_ns;
 
     Ok(!body.throttled)
}