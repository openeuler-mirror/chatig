use actix_web::{dev::{Service, ServiceRequest, ServiceResponse, Transform}, error::{ErrorBadRequest, ErrorInternalServerError}, Error, HttpMessage};
use std::{sync::Arc, task::{Context, Poll}};
use futures::{future::{ok, LocalBoxFuture, Ready}, StreamExt};
use actix_web::error::{ErrorUnauthorized, ErrorForbidden};
use crate::configs::settings::GLOBAL_CONFIG;
use crate::meta::middleware::traits::UserKeysTrait;
use crate::meta::middleware::impls::UserKeysImpl;
use serde_json::Value;

#[derive(Clone)]
pub struct Auth4ModelMiddleware {
    userkeys: Arc<dyn UserKeysTrait>, 
}

impl Auth4ModelMiddleware {
    pub fn new() -> Self {
        let userkeys = Arc::new(UserKeysImpl);
        Self { userkeys }
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
        })
    }
}

pub struct Auth4ModelAuthMiddleware<S> {
    service: Arc<S>,
    userkeys: Arc<dyn UserKeysTrait>,
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

            let userkey = match user_key_header {
                Some(s) => s,
                None => return Err(ErrorUnauthorized("Missing userkey header")),
            };

            match userkeys.check_userkey(&userkey).await {
                Ok(true) => {
                if let Some(model_value) = model {
                    match userkeys.check_userkey_model(&userkey, &model_value).await {
                        Ok(true) => {
                            service.call(req).await
                        }
                        Ok(false) => {
                            Err(ErrorForbidden("Invalid userkey and model combination"))
                        }
                        Err(err) => {
                            eprintln!("check_userkey_model error: {}", err);
                            Err(ErrorInternalServerError("check_userkey_model error"))
                        }
                    }
                } else {
                        Err(ErrorBadRequest("Missing model info"))
                    }
                } 
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
