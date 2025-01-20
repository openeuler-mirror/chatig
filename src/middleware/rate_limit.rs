use actix_web::{dev::{Service, ServiceRequest, ServiceResponse, Transform}, Error};
use leaky_bucket::RateLimiter;
use std::{task::{Context, Poll}, time::Duration};
use std::sync::Arc;
use futures::future::{ok, LocalBoxFuture, Ready};
use tokio::sync::Mutex;

use crate::configs::settings::GLOBAL_CONFIG;

// 定义限流中间件
#[derive(Clone)]
pub struct RateLimitMiddleware {
    limiter: Arc<Mutex<RateLimiter>>, // 使用共享的令牌桶
}

impl RateLimitMiddleware {
    pub fn new(rate_per_second: usize, max_capacity: usize, interval: Duration) -> Self {
        let limiter = RateLimiter::builder()
            .initial(max_capacity) // 初始令牌数量
            .refill(rate_per_second)   // 每次补充的令牌数
            .max(max_capacity)         // 最大令牌容量
            .interval(interval) // 补充时间间隔
            .fair(false)      // 是否启用公平分配
            .build();

        Self {
            limiter: Arc::new(Mutex::new(limiter)),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimitMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RateLimitMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RateLimitMiddlewareService {
            service,
            limiter: self.limiter.clone(),
        })
    }
}

pub struct RateLimitMiddlewareService<S> {
    service: S,
    limiter: Arc<Mutex<RateLimiter>>,
}

impl<S, B> Service<ServiceRequest> for RateLimitMiddlewareService<S>
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
        let fut = self.service.call(req);
        let limiter = self.limiter.clone();

        Box::pin(async move {
            if !config.rate_limit_enbled {
                return fut.await;
            }
            let allowed = {
                let limiter = limiter.lock().await;
                limiter.try_acquire(1)
            };

            if allowed {
                fut.await
            } else {
                Err(actix_web::error::ErrorTooManyRequests("Rate limit exceeded"))
            }
        })
    }
}
