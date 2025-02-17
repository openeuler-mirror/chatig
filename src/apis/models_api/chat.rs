use actix_web::{get, post, web, Error, HttpResponse, Responder, HttpRequest, HttpMessage};
use actix_web::error::ErrorBadRequest;
use log::{info, error};
use std::sync::Arc;

use crate::apis::models_api::schemas::ChatCompletionRequest;
use crate::apis::schemas::ErrorResponse;

use crate::cores::chat_models::chat_controller::Completions;
use crate::cores::chat_models::qwen::Qwen;
use crate::cores::chat_models::glm::GLM;
use crate::middleware::auth4model::Auth4ModelMiddleware;
use crate::middleware::qos::Qos;
use crate::utils::log::log_request;
use crate::cores::chat_models::llama::Llama;
use crate::cores::chat_models::bailian::Bailian;
use crate::cores::chat_models::deepseek::DeepSeek;
use crate::configs::settings::GLOBAL_CONFIG;

pub fn configure(cfg: &mut web::ServiceConfig, auth_middleware: Arc<Auth4ModelMiddleware>, qos: Arc<Qos>) {
    cfg.service(
        web::scope("/v1/chat") 
            .wrap(qos)
            .wrap(auth_middleware)  // 在这个作用域内应用中间件
            .service(health)
            .service(completions)
    );
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

    async fn completions(&self, req_body: web::Json<ChatCompletionRequest>, userid: String, appkey: String) -> Result<HttpResponse, Error> {
        self.model.completions(req_body, userid, appkey).await
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

#[post("/completions")]
pub async fn completions(req: HttpRequest, req_body: web::Json<ChatCompletionRequest>) -> Result<impl Responder, Error> {

    let config = &*GLOBAL_CONFIG;

    let mut appkey = "".to_string();
    if config.auth_remote_enabled {
        // Get appkey
        let appkey_header = req.headers().get("appKey");
        appkey = match appkey_header {
            Some(header_value) => {
                header_value.to_str().map_err(|_| ErrorBadRequest("Invalid appKey header"))?.to_string()
            }
            None => {
                return Err(ErrorBadRequest("App_key is missing"));
            }
        };
    }
    // Get userid
    let userid = req.extensions().get::<String>().cloned().unwrap_or_else(|| "".to_string());

    // 1. Validate that required fields exist in the request data
    if req_body.model.is_empty() || req_body.messages.is_empty() {
        let error_response = ErrorResponse {
            error: "Invalid request: model or messages cannot be empty.".into(),
        };
        return Ok(HttpResponse::BadRequest().json(error_response));
    }

    // 2. Parse the model name and series from the model field (Qwen/Qwen2.5-7B-Instruct)
    let parts: Vec<&str> = req_body.model.split('/').collect();
    if parts.len() != 2 {
        return Err(ErrorBadRequest("Invalid model format"));
    }

    //  3. Call the underlying API and return a unified data format
    let model: LLM = match parts[0] {
        "Qwen" => LLM::new(Box::new(Qwen {model_name: parts[1].to_string()})),
        "GLM" => LLM::new(Box::new(GLM {model_name: parts[1].to_string()})),
        "meta-llama" => LLM::new(Box::new(Llama {model_name: parts[1].to_string()})),
        "Bailian" => LLM::new(Box::new(Bailian {})),
        "deepseek-ai" => LLM::new(Box::new(DeepSeek {model_name: parts[1].to_string()})),
        _ => return Err(ErrorBadRequest(format!("Unsupported {} model series!", parts[0]))),
    };

    // 4. Send the request to the model service
    let response = model.completions(req_body, userid, appkey).await;
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
