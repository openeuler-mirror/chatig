use actix_web::{
    body::MessageBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::StatusCode,
    HttpResponse, Error as ActixError,
};
use futures_util::future::{ok, Ready};
use std::{
    future::{ready, Future},
    pin::Pin,
    rc::Rc,
    cell::RefCell,
    task::{Context, Poll},
    sync::Arc,
};

#[derive(Clone)]
pub struct Auth4Manage {
    userkeys: Arc<dyn UserKeysTrait>,
}

impl Auth4Manage {
    pub fn new(userkeys: Arc<dyn UserKeysTrait>) -> Self {
        Self { userkeys }
    }
}

impl<S, B> Transform<S, ServiceRequest> for Auth4Manage
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = ActixError;
    type Transform = Auth4ManageAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(Auth4ManageAuthMiddleware {
            service: Rc::new(RefCell::new(service)),
            userkeys: Arc::clone(&self.userkeys),
        })
    }
}

pub struct Auth4ManageAuthMiddleware<S> {
    service: Rc<RefCell<S>>,
    userkeys: Arc<dyn UserKeysTrait>,
}

impl<S, B> Service<ServiceRequest> for Auth4ManageAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = ActixError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.borrow_mut().poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let config = &*GLOBAL_CONFIG;
        if !config.auth_local_enabled {
            let fut = self.service.borrow_mut().call(req);
            return Box::pin(async move { fut.await });
        }

        let userkey = match req.headers().get("X-User-Key") {
            Some(val) => match val.to_str() {
                Ok(s) => s.to_string(),
                Err(_) => {
                    let resp = HttpResponse::Unauthorized().body("Invalid userkey header");
                    return Box::pin(async move {
                        Ok(req.into_response(resp.map_into_right_body()))
                    });
                }
            },
            None => {
                let resp = HttpResponse::Unauthorized().body("Missing userkey header");
                return Box::pin(async move { Ok(req.into_response(resp.map_into_right_body())) });
            }
        };

        let userkeys = Arc::clone(&self.userkeys);
        let mut srv = self.service.clone();

        Box::pin(async move {
            match userkeys.check_userkey(&userkey).await {
                Ok(true) => {
                    let fut = srv.borrow_mut().call(req);
                    fut.await
                }
                Ok(false) => {
                    let resp = HttpResponse::Forbidden().body("Invalid userkey");
                    Ok(ServiceResponse::new(req.into_parts().0, resp.map_into_right_body()))
                }
                Err(err) => {
                    eprintln!("check_userkey error: {}", err);
                    let resp = HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR);
                    Ok(ServiceResponse::new(req.into_parts().0, resp.map_into_right_body()))
                }
            }
        })
    }
}
