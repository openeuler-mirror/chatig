use actix_service::{Service, Transform};
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    Error, 
};
use futures::future::{ok, LocalBoxFuture, Ready};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;
use std::rc::Rc;

use crate::servers::invitation_code::check_invitation_code_exists;

// 中间件结构体
pub struct ApiKeyCheck {
    db_pool: Rc<Pool<PostgresConnectionManager<NoTls>>>, // 数据库连接池
}

// 实现构造函数
impl ApiKeyCheck {
    pub fn new(db_pool: Rc<Pool<PostgresConnectionManager<NoTls>>>) -> Self {
        Self { db_pool }
    }
}

// Transform trait 实现，用于中间件包装
impl<S, B> Transform<S, ServiceRequest> for ApiKeyCheck
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ApiKeyCheckMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ApiKeyCheckMiddleware {
            service,
            db_pool: self.db_pool.clone(),
        })
    }
}

// 中间件实现
pub struct ApiKeyCheckMiddleware<S> {
    service: S,
    db_pool: Rc<Pool<PostgresConnectionManager<NoTls>>>,
}

impl<S, B> Service<ServiceRequest> for ApiKeyCheckMiddleware<S>
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

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let db_pool = self.db_pool.clone();

        let api_key = req
            .headers()
            .get("Authorization")
            .and_then(|auth_header| auth_header.to_str().ok())
            .map(|auth_str| auth_str.replace("Bearer ", ""))
            .unwrap_or_default();

        // 检查 api-key 是否有效
        let fut = self.service.call(req);
        Box::pin(async move {
            let valid = check_invitation_code_exists(&db_pool, &api_key).await;
            match valid {
                Ok(true) => fut.await,
                Ok(false) => Err(actix_web::error::ErrorUnauthorized("Invalid or missing API key")),
                Err(_) => Err(actix_web::error::ErrorInternalServerError("Database error")),
            }
        })
    }
}