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
    cache: Arc<Mutex<AuthCache>>,
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
            .map(|s| s.to_string());

        let app_key = req.headers()
            .get("app_key")
            .and_then(|hv| hv.to_str().ok())
            .map(|s| s.to_string());

        let model_name = req.match_info().get("model").map(|m| m.to_string());

        // 克隆缓存以便在闭包中使用
        let cache = self.cache.clone();
        // 构造缓存的key
        let cache_key = format!("{}:{}:{}", user_key_header.clone().unwrap_or_default(), app_key.clone().unwrap_or_default(), model_name.unwrap_or_default());

        // 检查缓存
        let cache_result = self.cache.lock().unwrap().check_cache_model(&cache_key);

        if let Some(user_id) = cache_result {
            // 缓存命中，返回成功
            println!("Cache hit for user_id: {:?}", user_id);
            let fut = self.service.call(req);
            return Box::pin(fut);
        }

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
            if !config.auth_local_enabled {
                return service.call(req).await;
            }

            // 如果没有启用鉴权，直接继续请求
            if !config.auth_local_enabled && !config.auth_remote_enabled {
                return service.call(req).await;
            }

            let userkey = match user_key_header {
                Some(s) => s,
                None => return Err(ErrorUnauthorized("Missing userkey header")),
            };

            // 如果启用了本地鉴权
            if config.auth_local_enabled {
                match userkeys.check_userkey(&userkey).await {
                    Ok(true) => {
                        if let Some(model_value) = model.clone() {
                            match userkeys.check_userkey_model(&userkey, &model_value).await {
                                Ok(true) => {
                                    return service.call(req).await;
                                }
                                Ok(false) => {
                                    return Err(ErrorForbidden("Invalid userkey and model combination"));
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
                        return Err(ErrorForbidden("Invalid userkey"));
                    }
                    Err(err) => {
                        eprintln!("check_userkey error: {}", err);
                        return Err(ErrorInternalServerError("check_userkey error"));
                    }
                }
            }

             // 如果启用了远程鉴权
             if config.auth_remote_enabled {
                let url = format!("{}/v1/apiInfo/check", config.auth_remote_server);
                let client = reqwest::Client::new();
                let response = client.post(&url)
                    .json(&serde_json::json!({
                        "apiKey": userkey.clone(),
                        "appKey": app_key.clone(),
                        "modelName": model.unwrap_or_default(),
                        // "cloudRegonId": config.cloud_region_id
                    }))
                    .send()
                    .await;

                match response {
                    Ok(resp) if resp.status().is_success() => {
                        // 获取远程校验通过后的用户ID，缓存它
                        if let Some(user_id) = resp.json::<Value>().await.ok().and_then(|json| json.get("userId").and_then(|u| u.as_str()).map(|u| u.to_string())) {
                            cache.lock().unwrap().set_cache_model(&cache_key, user_id, Duration::from_secs(3600));
                        }
                        
                        return service.call(req).await;
                    }
                    _ => {
                        eprintln!("Remote check failed for userkey: {}", userkey);
                        return Err(ErrorForbidden("Remote validation failed"));
                    }
                }
            }

            Err(ErrorForbidden("Authentication failed"))
        })
    }
}
