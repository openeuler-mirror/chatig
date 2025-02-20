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
use log::info;

use actix_web::error::PayloadError;
// 引入 BoxedPayloadStream 定义
pub type BoxedPayloadStream = std::pin::Pin<Box<dyn futures_core::Stream<Item = Result<Bytes, PayloadError>>>>;

use crate::{configs::settings::GLOBAL_CONFIG, cores::control::model_limits::LimitsManager};
use crate::GLOBAL_MULTI_SERVER_CLIENT;

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
        let auth_local_enabled = config.auth_local_enabled;
        let localuserid = config.localuserid.clone();

        let fut = async move {
            let (chat_request, body_clone) = read_payload_fut.await?;
            let model = chat_request.model;

            // 将请求体重新放回 ServiceRequest
            let body_bytes = Bytes::from(body_clone);
            let stream = once(async { Ok::<_, PayloadError>(body_bytes) });
            let boxed_stream: BoxedPayloadStream = Box::pin(stream);
            let payload = actix_web::dev::Payload::from(boxed_stream);
            req.set_payload(payload);

            if coil_enabled && auth_local_enabled{
                let userid = localuserid;
                req.extensions_mut().insert(userid.clone());
              
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
            } else if coil_enabled && auth_remote_enabled {
            // if coil_enabled {
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
    // let client: reqwest::Client = reqwest::Client::new();
    let (client, base_url) = {
        let global_client = GLOBAL_MULTI_SERVER_CLIENT.lock().unwrap(); // 直接加锁
        let (client, base_url) = global_client.get_client_and_base_url(&apikey, &model);
        // 克隆必要的数据，避免持有 MutexGuard
        (client.clone(), base_url.to_string())
    };

    let limit_manager = LimitsManager::default();
    let limits = limit_manager.get_limits_object(&model)
        .await
        .map_err(|e| ErrorBadRequest(format!("Failed to get model limits: {}", e)))?;
    let limits = match limits {
        Some(limits) => limits,
        None => return Err(ErrorBadRequest(format!("{} model is not supported", model))),
    };

    // 修改请求路径
    // let url = format!("http://{}/query_and_consume", ip);
    let url = format!("{}/query_and_consume", base_url);

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
    // let client: reqwest::Client = reqwest::Client::new();
    let (client, base_url) = {
        let global_client = GLOBAL_MULTI_SERVER_CLIENT.lock().unwrap(); // 直接加锁
        let (client, base_url) = global_client.get_client_and_base_url(&apikey, &model);
        // 克隆必要的数据，避免持有 MutexGuard
        (client.clone(), base_url.to_string())
    };

    let url = format!("{}/consume", base_url);

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
    // let client: reqwest::Client = reqwest::Client::new();
    let (client, base_url) = {
        let global_client = GLOBAL_MULTI_SERVER_CLIENT.lock().unwrap(); // 直接加锁
        let (client, base_url) = global_client.get_client_and_base_url(&apikey, &model);
        // 克隆必要的数据，避免持有 MutexGuard
        (client.clone(), base_url.to_string())
    };

    // 修改请求路径
    // let url = format!("http://{}/throttled", ip);
    let url = format!("{}/throttled", base_url);

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

use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::BTreeMap;
use tokio::net::TcpStream;
use reqwest::Client;
use tokio::time::{self, Duration};
use futures::future::join_all;
use std::sync::Mutex;

// 客户端组
pub struct ClientGroup {
    clients: Vec<Client>,
    index: AtomicUsize,
    base_url: String,
}

impl ClientGroup {
    pub fn new(base_url: &str) -> Self {
        let config = &*GLOBAL_CONFIG;
        let mut clients = Vec::with_capacity(config.connections_per_server);
        for _ in 0..config.connections_per_server {
            let client = Client::new();
            clients.push(client);
        }
        ClientGroup {
            clients,
            index: AtomicUsize::new(0),
            base_url: base_url.to_string(),
        }
    }

    fn get_client(&self) -> &Client {
        let idx = self.index.fetch_add(1, Ordering::Relaxed) % self.clients.len();
        &self.clients[idx]
    }

    fn get_base_url(&self) -> &str {
        &self.base_url
    }
}

// 多服务端客户端
pub struct MultiServerClient {
    pub client_groups: BTreeMap<String, ClientGroup>,
    default_client: Client,
    default_base_url: String,
}

impl MultiServerClient {
    pub fn new() -> Self {
        let config = &*GLOBAL_CONFIG;
        let mut client_groups = BTreeMap::new();
        for ip in &config.multi_ip {
            let base_url = format!("http://{}", ip);
            let group = ClientGroup::new(&base_url);
            client_groups.insert(ip.clone(), group);
        }
        // 默认空的给找不到的时候用
        let default_client = Client::new();
        let default_base_url = "http://127.0.0.1:8011".to_string();
        MultiServerClient { 
            client_groups,
            default_client,
            default_base_url,
        }
    }

    pub fn get_client_and_base_url(&self, user: &str, item: &str) -> (&Client, &str) {
        if self.client_groups.is_empty() {
            return (&self.default_client, &self.default_base_url);
        }
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        user.hash(&mut hasher);
        item.hash(&mut hasher);
        let hash = hasher.finish();
        let keys: Vec<&String> = self.client_groups.keys().collect();
        let idx = (hash % keys.len() as u64) as usize; // 直接对 keys 的长度取模
        let ip = keys[idx];
        let group = self.client_groups.get(ip).unwrap();
        (group.get_client(), group.get_base_url())
    }
    
    pub async fn is_address_available(ip: &str) -> bool {
        match time::timeout(Duration::from_secs(1), TcpStream::connect(ip)).await {
            Ok(Ok(_)) => true,
            _ => false,
        }
    }
}

pub async fn check_and_remove_unavailable_clients(multi_server_client_clone: Arc<Mutex<MultiServerClient>>) {
    let client = multi_server_client_clone.lock().unwrap();
    // 提取需要的数据到局部变量
    let config = &*GLOBAL_CONFIG;
    let multi_ip = config.multi_ip.clone();
    drop(client); // 提前释放锁

    let mut tasks = vec![];
    // 为每个地址创建一个检查任务
    for ip in multi_ip.clone() {
        let ip_clone = ip.clone();
        let task = tokio::spawn(async move {
            (ip_clone.clone(), MultiServerClient::is_address_available(&ip_clone).await)
        });
        tasks.push(task);
    }

    // 等待所有检查任务完成
    let results = join_all(tasks).await;

    // 再次获取锁来更新状态
    let mut client = multi_server_client_clone.lock().unwrap();

    for result in results {
        if let Ok((ip, is_available)) = result {
            if is_available {
                if!client.client_groups.contains_key(&ip) {
                    let base_url = format!("http://{}", ip);
                    let group = ClientGroup::new(&base_url);
                    client.client_groups.insert(ip.clone(), group);
                    info!(target: "access_log", "Address {} is available. Adding clients...", ip);
                    // println!("Address {} is available. Adding clients...", ip);
                }
            } else {
                if client.client_groups.contains_key(&ip) {
                    client.client_groups.remove(&ip);
                    info!(target: "access_log", "Address {} is not available. Removing clients...", ip);
                    // println!("Address {} is not available. Removing clients...", ip);
                }
            }
        }
    }
}