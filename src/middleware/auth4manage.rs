use actix_web::{dev::{Service, ServiceRequest, ServiceResponse, Transform}, error::ErrorInternalServerError, Error};
use std::{task::{Context, Poll}, sync::Arc};
use futures::future::{ok, LocalBoxFuture, Ready};
use actix_web::error::{ErrorUnauthorized, ErrorForbidden};
use crate::configs::settings::GLOBAL_CONFIG;
use crate::meta::middleware::traits::UserKeysTrait;
use crate::meta::middleware::impls::UserKeysImpl;

#[derive(Clone)]
pub struct Auth4ManageMiddleware {
    userkeys: Arc<dyn UserKeysTrait>, 
}

impl Auth4ManageMiddleware {
    pub fn new() -> Self {
        let userkeys = Arc::new(UserKeysImpl);
        Self { userkeys }
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
        })
    }
}

pub struct Auth4ManageAuthMiddleware<S> {
    service: S,
    userkeys: Arc<dyn UserKeysTrait>,  // 共享的用户验证逻辑
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

        // 移动req到fut中
        let fut = self.service.call(req);

        Box::pin(async move {
            if !config.auth_local_enabled {
                return fut.await;
            }

            let userkey = match user_key_header {
                Some(s) => s,
                None => return Err(ErrorUnauthorized("Missing userkey header")),
            };

            match userkeys.check_userkey(&userkey).await {
                Ok(true) => fut.await,  
                Ok(false) => {
                    Err(ErrorForbidden("Invalid userkey"))
                }
                Err(err) => {
                    eprintln!("check_userkey error: {}", err);
                    Err(ErrorInternalServerError("check_userkey error"))
                }
            }
        })
    }
}
