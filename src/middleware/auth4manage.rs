use actix_web::{dev::{Service, ServiceRequest, ServiceResponse, Transform}, Error};
use std::{task::{Context, Poll}, sync::{Arc, Mutex}};
use futures::future::{ok, LocalBoxFuture, Ready};
use actix_web::error::{ErrorUnauthorized, ErrorForbidden, ErrorInternalServerError};
use crate::configs::settings::GLOBAL_CONFIG;
use crate::meta::middleware::traits::UserKeysTrait;
use crate::meta::middleware::impls::UserKeysImpl;
use crate::middleware::auth_cache::AuthCache;
use std::time::Duration;

#[derive(Clone)]
pub struct Auth4ManageMiddleware {
    userkeys: Arc<dyn UserKeysTrait>, 
    cache: Arc<Mutex<AuthCache>>,
}

impl Auth4ManageMiddleware {
    pub fn new() -> Self {
        let userkeys = Arc::new(UserKeysImpl);
        let cache = Arc::new(Mutex::new(AuthCache::new()));
        Self { userkeys, cache }
    }
}

impl<S, B> Transform<S, ServiceRequest> for Auth4ManageMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = Auth4ManageAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(Auth4ManageAuthMiddleware {
            service,
            userkeys: self.userkeys.clone(),
            cache: self.cache.clone(),
        })
    }
}

pub struct Auth4ManageAuthMiddleware<S> {
    service: S,
    userkeys: Arc<dyn UserKeysTrait>,  // 共享的用户验证逻辑
    cache: Arc<Mutex<AuthCache>>,
}

impl<S, B> Service<ServiceRequest> for Auth4ManageAuthMiddleware<S>
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

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let config = &*GLOBAL_CONFIG;
        let userkeys = self.userkeys.clone();
        let user_key_header = req.headers()
            .get("Authorization")
            .and_then(|hv| hv.to_str().ok())
            .map(|s| s.to_string());

        // 克隆缓存以便在闭包中使用
        let cache = self.cache.clone();
        // 检查本地缓存
        // let key = format!("{}:{}", req.headers().get("X-Api-Key").unwrap().to_str().unwrap(), req.match_info().get("model").unwrap());
        let cache_result = match user_key_header {
            Some(ref key) => self.cache.lock().unwrap().check_cache(key),
            None => None,
        };

        if let Some(user_id) = cache_result {
            // 缓存命中，返回成功
            println!("Cache result: {:?}", cache_result);
            let fut = self.service.call(req);
            return Box::pin(fut);
        }

        // 移动req到fut中
        let fut = self.service.call(req);

        Box::pin(async move {
            if !config.auth_local_enabled && !config.auth_remote_enabled {
                return fut.await;
            }
            
            // 本地鉴权逻辑
            if config.auth_local_enabled {
                let userkey = match user_key_header {
                    Some(s) => s,
                    None => return Err(ErrorUnauthorized("Missing userkey header")),
                };

                match userkeys.check_userkey(&userkey).await {
                    Ok(true) => {
                        // 本地鉴权成功，缓存用户ID
                        cache.lock().unwrap().set_cache(userkey.clone(), Duration::from_secs(3600));

                        return fut.await;
                    },
                    Ok(false) => {
                        return Err(ErrorForbidden("Invalid userkey"));
                    },
                    Err(err) => {
                        eprintln!("check_userkey error: {}", err);
                        return Err(ErrorInternalServerError("check_userkey error"));
                    }
                }
            }

            // 远程鉴权逻辑
            if config.auth_remote_enabled {
                let url = format!("{}/validate", config.auth_remote_server);
                let client = reqwest::Client::new();
                let response = client.post(&url)
                    .header("X-User-Key", user_key_header.unwrap_or_default())
                    .send()
                    .await;

                match response {
                    Ok(resp) if resp.status().is_success() => {
                        // 远程鉴权成功，缓存用户ID
                        // self.cache.lock().unwrap().set_cache(&key, "user_id".to_string(), Duration::from_secs(3600));
                        return fut.await;
                    }
                    _ => return Err(ErrorForbidden("Remote validation failed")),
                }
            }

            Err(ErrorForbidden("No valid authentication method"))
        })
    }
}
