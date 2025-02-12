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
use serde_yaml::Value;
use std::sync::Mutex;
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
        // 新增逻辑 ///////////////////////////////
        let appkey = req.headers()
            .get("app_key")
            .and_then(|hv| hv.to_str().ok())
            .map(|s| s.to_string());
        let app_key = appkey.clone().unwrap_or_default();

        // 结束 /////////////////////////////////////////

        let api_key = req
           .headers()
           .get("Authorization")
           .and_then(|auth_header| auth_header.to_str().ok())
           .map(|auth_str| auth_str.replace("Bearer ", ""))
           .unwrap_or_default();

        let payload = req.take_payload();
        let body = BytesMut::new();

        async fn read_payload(mut payload: impl Stream<Item = Result<Bytes, PayloadError>> + Unpin, mut body: BytesMut) -> Result<(ChatCompletionRequest, BytesMut), Error> {
            while let Some(chunk) = payload.next().await {
                let chunk = chunk.map_err(|e| {
                    actix_web::error::ErrorInternalServerError(format!("Failed to read payload: {}", e))
                })?;
                body.extend_from_slice(&chunk);
            }
            // 克隆 body 的内容
            let body_clone = body.clone();
            // 将 serde_json::Error 转换为 actix_web::Error
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

        // 创建 QosAuthCache 实例并使用 Mutex 包装 /////////////////////////////////////////
        let cache: Arc<Mutex<QosAuthCache>> = Arc::new(Mutex::new(QosAuthCache::new()));
        //////////////////////////////////////////////////////////////////////

        let fut = async move {
            let (chat_request, body_clone) = read_payload_fut.await?;
            let model = chat_request.model;

            // 将请求体重新放回 ServiceRequest
            let body_bytes = Bytes::from(body_clone);
            let stream = once(async { Ok::<_, PayloadError>(body_bytes) });
            let boxed_stream: BoxedPayloadStream = Box::pin(stream);
            let payload = actix_web::dev::Payload::from(boxed_stream);
            req.set_payload(payload);

            // 新增检测缓存 ///////////////////////////////////////////
            let cache_key = format!("{}{}{}", api_key, app_key, model);
            let mut userid = "".to_string();
            if let Some(user_id) = cache.lock().unwrap().check_cache_model(&cache_key) {
                userid = user_id;
            } 
            /////////////////////////////////////////////////////////////

            if coil_enabled {
                // 新增获取用户请求，肯定是鉴权通过了的 /////////////////////////////////////
                if userid == "" {
                    let url = format!("{}/v1/apiInfo/check", config.auth_remote_server);
                    let client = reqwest::Client::new();
                    let response = client.post(&url)
                        .json(&serde_json::json!({
                            "apiKey": api_key.clone(),
                            "appKey": app_key.clone(),
                            "modelName": model.clone(),
                            // "cloudRegonId": config.cloud_region_id
                        }))
                        .send()
                        .await;

                    match response {
                        Ok(resp) if resp.status().is_success() => {
                            // 获取远程校验通过后的用户ID，缓存它
                            if let Some(user_id) = resp.json::<Value>().await.ok().and_then(|json| json.get("userId").and_then(|u| u.as_str()).map(|u| u.to_string())) {
                                userid = user_id.clone();
                                cache.lock().unwrap().set_cache_model(&cache_key, user_id, Duration::from_secs(3600));
                            }
                            
                            return service.call(req).await;
                        }
                        _ => {
                            // 不处理校验的
                        }
                    }
                }
                if userid == "" {
                    let api_key_clone = api_key.clone();
                    let model_clone = model.clone();
                    let (valid_tokens, valid) = join!(
                        throttled(api_key_clone, model_clone),
                        query_and_consume(api_key, model)
                    );
                    let valid_tokens = valid_tokens?;
                    let valid = valid?;
    
                    if valid && valid_tokens {
                        service.call(req).await
                    } else {
                        Err(actix_web::error::ErrorTooManyRequests("Throttle for request"))
                    }     
                } else {
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
                }
                // 结束 /////////////////////////////////////////////////////////////////////////    
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

    // 获取apikey，并且根据apikey获取到用户的信息
    // doing

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
        request_amount: "8192".to_string(),
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



//////////////////////////////////////////////////////////////////////////////////
// 增加缓存代码
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct QosAuthCache {
    pub cache_model: HashMap<String, (String, Instant)>,  // 存储 api_key+app_key+model_name -> (user_id, expire_time)
}

impl QosAuthCache {
    // 创建新的缓存实例
    pub fn new() -> Self {
        QosAuthCache {
            cache_model: HashMap::new(),
        }
    }

    // 检查model缓存是否有效
    pub fn check_cache_model(&self, key: &str) -> Option<String> {
        if let Some((user_id, expire_time)) = self.cache_model.get(key) {
            if Instant::now() < *expire_time {
                return Some(user_id.clone());
            }
        }
        None
    }

    // 设置model缓存
    pub fn set_cache_model(&mut self, key: &str, user_id: String, ttl: Duration) {
        let expire_time = Instant::now() + ttl;
        self.cache_model.insert(key.to_string(), (user_id, expire_time));
    }
}
