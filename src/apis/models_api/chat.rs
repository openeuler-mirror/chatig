use actix_web::{get, post, web, Error, HttpResponse, Responder, HttpRequest};
use actix_web::error::ErrorBadRequest;
use log::{info, error};
use serde_yaml::Value;
use std::time::Duration;
use std::sync::{Mutex, Arc};

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
use crate::middleware::qos::QosAuthCache;
use crate::configs::settings::GLOBAL_CONFIG;


pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v1/chat") 
            .wrap(Auth4ModelMiddleware::new())  // 在这个作用域内应用中间件
            .wrap(Qos::new())
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

    async fn completions(&self, req_body: web::Json<ChatCompletionRequest>, apikey: String, curl_mode: String) -> Result<HttpResponse, Error> {
        self.model.completions(req_body, apikey, curl_mode).await
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

    // 获取 Authorization 头部的值，应该传入到具体的模型函数中
    let auth_header = req.headers().get("Authorization");
    let apikey = match auth_header {
        Some(header_value) => {
            let auth_str = header_value.to_str().map_err(|_| ErrorBadRequest("Invalid Authorization header"))?;
            if let Some(token_str) = auth_str.strip_prefix("Bearer ") {
                token_str.to_string()
            } else {
                return Err(ErrorBadRequest("Authorization header does not contain 'Bearer '"));
            }
        }
        None => {
            return Err(ErrorBadRequest("Authorization header is missing"));
        }
    };

    //  缓存 ///////////////////////////////////////////
    // 获取 appkey 头部的值
    let appkey_header = req.headers().get("app_key");
    let appkey = match appkey_header {
        Some(header_value) => {
            header_value.to_str().map_err(|_| ErrorBadRequest("Invalid app_key header"))?.to_string()
        }
        None => {
            // 没有app_key就没有，不影响
            "".to_string()
        }
    };

    let cache: Arc<Mutex<QosAuthCache>> = Arc::new(Mutex::new(QosAuthCache::new()));
    let cache_key = format!("{}{}{}", apikey, appkey, req_body.model.clone());
    let mut userid = "".to_string();
    if let Some(user_id) = cache.lock().unwrap().check_cache_model(&cache_key) {
        userid = user_id;
    } 

    let config = &*GLOBAL_CONFIG;
    if userid == "" {
        let url = format!("{}/v1/apiInfo/check", config.auth_remote_server);
        let client = reqwest::Client::new();
        let response = client.post(&url)
            .json(&serde_json::json!({
                "apiKey": apikey.clone(),
                "appKey": appkey.clone(),
                "modelName": req_body.model.clone(),
                // "cloudRegonId": config.cloud_region_id
            }))
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                if let Some(user_id) = resp.json::<Value>().await.ok().and_then(|json| json.get("userId").and_then(|u| u.as_str()).map(|u| u.to_string())) {
                    userid = user_id.clone();
                    cache.lock().unwrap().set_cache_model(&cache_key, user_id, Duration::from_secs(3600));
                }
            }
            _ => {
                // 只获取值，不校验返回错误
            }
        }
    }
    /////////////////////////////////////////////////////////////

    // 打印获取到的令牌，方便调试
    let curl_model = req_body.model.clone();

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
    if userid == "" {
        let response = model.completions(req_body, apikey, curl_model).await;
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
    } else {
        let response = model.completions(req_body, userid, curl_model).await;
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

}
