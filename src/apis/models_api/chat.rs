use actix_web::{get, post, web, Error, HttpResponse, Responder, HttpRequest};
use actix_web::error::ErrorBadRequest;
use log::{info, error};

use crate::apis::models_api::schemas::ChatCompletionRequest;
use crate::apis::schemas::ErrorResponse;

use crate::cores::chat_models::chat_controller::Completions;
use crate::cores::chat_models::qwen::Qwen;
use crate::cores::chat_models::glm::GLM;
use crate::utils::log::log_request;
use crate::cores::chat_models::llama::Llama;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(health)
       .service(completions);
}

///获取健康检查
/// 
/// # 健康检查
/// ```rust
/// get /health
/// 如果接口没有问题，则返回"OK"
/// ```
#[utoipa::path(
    get,  // 请求方法
    path = "/health",  // 路径
    responses((status = 200, body = String))  // 响应内容
)]

#[get("/health")]
pub async fn health() -> impl Responder {
    "OK"
}

// define an interface layer that calls the completions method of the large model
struct LLM {
    model: Box<dyn Completions>,
}

impl LLM {
    fn new(model: Box<dyn Completions>) -> Self {
        LLM { model }
    }

    async fn completions(&self, req_body: web::Json<ChatCompletionRequest>) -> Result<HttpResponse, Error> {
        self.model.completions(req_body).await
    }
}

#[utoipa::path(
    post,  // 请求方法
    path = "/v1/chat/completions",  // 路径
    request_body = ChatCompletionRequest,
    responses(
        (status = 200, body = CompletionsResponse), //还没有写完
        (status = 400, body = ErrorResponse),
        (status = 500, body = ErrorResponse),
    )  // 响应内容
)]

#[post("/v1/chat/completions")]
pub async fn completions(req: HttpRequest, req_body: web::Json<ChatCompletionRequest>) -> Result<impl Responder, Error> {
    // 1. Validate that required fields exist in the request data
    if req_body.model.is_empty() || req_body.messages.is_empty() {
        let error_response = ErrorResponse {
            error: "Invalid request: model or messages cannot be empty.".into(),
        };
        return Ok(HttpResponse::BadRequest().json(error_response));
    }

    // 2. Call the underlying API and return a unified data format
    let model_name = req_body.model.clone();
    let model_series = model_name.split("/").next().unwrap_or("");
    let model: LLM = match model_series {
        "Qwen" => LLM::new(Box::new(Qwen {})),
        "GLM" => LLM::new(Box::new(GLM {})),
        "meta-llama" => LLM::new(Box::new(Llama {})),
        _ => return Err(ErrorBadRequest(format!("Unsupported {} model series!", model_series))),
    };

    // 3. Send the request to the model service
    let response = model.completions(req_body).await;
    match response {
        Ok(resp) => {
            info!(target: "access_log", "{}", log_request(req.clone(),  resp.status().as_u16(), None).await.unwrap());
            Ok(resp)
        }
        Err(err) => {
            error!(target: "error_log", "{}", log_request(req.clone(), err.as_response_error().status_code().as_u16(), Some(&format!("{}", err))).await.unwrap());
            Err(err)
        }
    }  
}
