use actix_web::{dev::{Service, ServiceRequest, ServiceResponse, Transform}, error::{ErrorBadRequest, ErrorInternalServerError}, Error, HttpMessage};
use std::{sync::{Arc, Mutex}, task::{Context, Poll}};
use futures::{future::{ok, LocalBoxFuture, Ready}, StreamExt};
use actix_web::error::{ErrorUnauthorized, ErrorForbidden};
use crate::configs::settings::GLOBAL_CONFIG;
use crate::meta::middleware::traits::UserKeysTrait;
use crate::meta::middleware::impls::UserKeysImpl;
use crate::middleware::auth_cache::AuthCache;
use serde_json::Value;
use std::time::Duration;

#[derive(Clone)]
pub struct Auth4ModelMiddleware {
    userkeys: Arc<dyn UserKeysTrait>,
    pub cache: Arc<Mutex<AuthCache>>,
}

impl Auth4ModelMiddleware {
    pub fn new() -> Self {
        let userkeys = Arc::new(UserKeysImpl);
        let cache = Arc::new(Mutex::new(AuthCache::new()));
        Self { userkeys, cache }
    }
}

impl<S, B> Transform<S, ServiceRequest> for Auth4ModelMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = Auth4ModelAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(Auth4ModelAuthMiddleware {
            service: Arc::new(service),
            userkeys: self.userkeys.clone(),
            cache: self.cache.clone(),
        })
    }
}

pub struct Auth4ModelAuthMiddleware<S> {
    service: Arc<S>,
    userkeys: Arc<dyn UserKeysTrait>,
    cache: Arc<Mutex<AuthCache>>,
}

impl<S, B> Service<ServiceRequest> for Auth4ModelAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let config = &*GLOBAL_CONFIG;
        let userkeys = self.userkeys.clone();
        let user_key_header = req.headers()
            .get("Authorization")
            .and_then(|hv| hv.to_str().ok())
            .map(|auth_str| auth_str.replace("Bearer ", ""))
            .map(|s| s.to_string());

        let app_key_header = req.headers()
            .get("appKey")
            .and_then(|hv| hv.to_str().ok())
            .map(|s| s.to_string());

        let cache = self.cache.clone();
        Box::pin(async move {
            let mut body = actix_web::web::BytesMut::new();
            while let Some(chunk) = req.take_payload().next().await {
                let chunk = chunk?;
                body.extend_from_slice(&chunk);
            }

            let model = if let Ok(json) = serde_json::from_slice::<Value>(&body) {
                json.get("model").and_then(|m| m.as_str().map(|s| s.to_string()))
            } else {
                None
            };
            let (_, mut new_payload) = actix_http::h1::Payload::create(true);
            new_payload.unread_data(body.freeze());
            req.set_payload(actix_web::dev::Payload::from(new_payload));

            // 如果没有启用鉴权，直接继续请求
            if !config.auth_local_enabled && !config.auth_remote_enabled {
                return service.call(req).await;
            }

            let api_key = match user_key_header {
                Some(s) => s,
                None => return Err(ErrorUnauthorized("Missing api_key header")),
            };

            let model_name = match model.clone() {
                Some(s) => s,
                None => return Err(ErrorUnauthorized("Missing model header")),
            };

            // 如果启用了本地鉴权
            if config.auth_local_enabled {
                match userkeys.check_userkey(&api_key).await {
                    Ok(true) => {
                        if let Some(model_value) = model.clone() {
                            match userkeys.check_userkey_model(&api_key, &model_value).await {
                                Ok(true) => {
                                    return service.call(req).await;
                                }
                                Ok(false) => {
                                    return Err(ErrorForbidden("Invalid api_key and model combination"));
                                }
                                Err(err) => {
                                    eprintln!("check_userkey_model error: {}", err);
                                    return Err(ErrorInternalServerError("check_userkey_model error"));
                                }
                            }
                        } else {
                            return Err(ErrorBadRequest("Missing model info"));
                        }
                    }
                    Ok(false) => {
                        return Err(ErrorForbidden("Invalid api_key"));
                    }
                    Err(err) => {
                        eprintln!("check_userkey error: {}", err);
                        return Err(ErrorInternalServerError("check_userkey error"));
                    }
                }
            }

            // 如果启用了远程鉴权
             if config.auth_remote_enabled {
                let app_key = match app_key_header {
                    Some(s) => s,
                    None => return Err(ErrorUnauthorized("Missing app_key header")),
                };
                
                // 构造缓存的key
                let cache_key = format!("{}:{}:{}", api_key.clone(), app_key.clone(), model_name.clone());

                // 检查缓存
                // println!("cache_key: {}", cache_key);
                let cache_result = cache.lock().unwrap().check_cache_model(&cache_key);

                if let Some(user_id) = cache_result {
                    // 缓存命中，返回成功
                    // println!("Cache hit for user_id: {:?}", user_id);
                    req.extensions_mut().insert(user_id);
                    return service.call(req).await;
                }

                let url = format!("{}/v1/apiInfo/check", config.auth_remote_server);
                let client = reqwest::Client::new();
                let response = client.post(&url)
                    .json(&serde_json::json!({
                        "apiKey": api_key.clone(),
                        "appKey": app_key.clone(),
                        "modelName": model_name.clone(),
                        "cloudRegionId": config.cloud_region_id
                    }))
                    .send()
                    .await;

                // println!("response: {:?}", response);
                match response {
                    Ok(resp) if resp.status().is_success() => {
                        if let Ok(json) = resp.json::<serde_json::Value>().await {
                            let account_id = json.get("accountId").and_then(|u| u.as_str()).map(|u| u.to_string());
                            // let user_id = json.get("userId").and_then(|u| u.as_str()).map(|u| u.to_string());
                            let is_valid = json.get("isValid").and_then(|v| v.as_bool());
            
                            if let (Some(user_id), Some(true)) = (account_id.clone(), is_valid) {
                                // 获取远程校验通过后的用户ID，缓存它
                                req.extensions_mut().insert(user_id.clone());
                                cache.lock().unwrap().set_cache_model(&cache_key, user_id, Duration::from_secs(config.auth_cache_time)); // 设置缓存时间
                                return service.call(req).await;
                            }
                            // println!("accountId: {:?}, isValid: {:?}, user_id{:?}", account_id, is_valid, user_id);
                        }
                        // 如果 accountId 为空或 isValid 为 false，返回错误
                        return Err(ErrorForbidden("Remote validation failed: accountId is empty or isValid is false"));
                    }
                    _ => {
                        return Err(ErrorForbidden("Remote validation failed"));
                    }
                }
            }

            Err(ErrorForbidden("Authentication failed"))
        })
    }
}
