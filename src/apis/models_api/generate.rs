use actix_web::{
    post, web, Error, HttpResponse, Responder, HttpRequest, HttpMessage,
};
use actix_web::error::ErrorBadRequest;
use log::{info, error};
use std::collections::HashMap;

use crate::cores::models::generate::generate_controller::{GenerateRequest, Generate};
use crate::cores::models::generate::support_engines::std_generate;
use crate::utils::log::log_request;
use crate::configs::settings::GLOBAL_CONFIG;

pub fn configure(
    cfg: &mut web::ServiceConfig,
) {
    cfg.service(
        web::scope("/v1/tgi")
            .service(generate)
            .service(generate_stream)
    );  
}

/// 定义接口层，调用大模型 completions 方法
struct GenerateModel {
    model: Box<dyn Generate>,
}

impl GenerateModel {
    fn new(model: Box<dyn Generate>) -> Self {
        GenerateModel { model }
    }

    async fn generate(
        &self,
        req_body: web::Json<GenerateRequest>,
        user_name: String,
        user_id: String,
        service_name: String,
        service_id: String,
        user_type: String,
        user_level: String,
        pay_status: String,
        aicpid: String,
        is_stream: bool
    ) -> Result<HttpResponse, Error> {
        self.model.generate(req_body, user_name, user_id, service_name, service_id, user_type, user_level, pay_status, aicpid, is_stream).await
    }
}

#[post("/generate")]
pub async fn generate(
    req: HttpRequest,
    req_body: web::Json<GenerateRequest>,
) -> Result<impl Responder, Error> {
    // 1. Validate that required fields exist in the request data
    if req_body.model.is_empty()  {
        let error_response = "Invalid request: model  cannot be empty.";
        return Ok(HttpResponse::BadRequest().json(error_response));
    }

    // 2. Get the user ID and service name from the request extensions
    let _config = &*GLOBAL_CONFIG;
    let username = req.extensions().get::<HashMap<&str, String>>().and_then(|data| data.get("user_name").cloned()).unwrap_or_else(|| "".to_string());
    let userid = req.extensions().get::<HashMap<&str, String>>().and_then(|data| data.get("user_id").cloned()).unwrap_or_else(|| "".to_string());
    let servicename = req.extensions().get::<HashMap<&str, String>>().and_then(|data| data.get("service_name").cloned()).unwrap_or_else(|| "".to_string());
    let serviceid = req.extensions().get::<HashMap<&str, String>>().and_then(|data| data.get("service_id").cloned()).unwrap_or_else(|| "".to_string());
    let user_type = req.extensions().get::<HashMap<&str, String>>().and_then(|data| data.get("user_type").cloned()).unwrap_or_else(|| "".to_string());
    let user_level = req.extensions().get::<HashMap<&str, String>>().and_then(|data| data.get("user_level").cloned()).unwrap_or_else(|| "".to_string());
    let pay_status = req.extensions().get::<HashMap<&str, String>>().and_then(|data| data.get("pay_status").cloned()).unwrap_or_else(|| "".to_string());
    let aicpid = req.extensions().get::<HashMap<&str, String>>().and_then(|data| data.get("aicpid").cloned()).unwrap_or_else(|| "[]".to_string());

    // 3. Parse the model name and series from the model field (e.g., Qwen/Qwen2.5-7B-Instruct)
    let (model_series, model_name) = match req_body.model.split_once('/') {
        Some((series, name)) => (series, name),
        None => ("std", req_body.model.as_str()),
    };

    // 5. Call the underlying API and return a unified data format
    let model: GenerateModel = match model_series {
        "Qwen" | "GLM" | "meta-llama" | "Bailian" | "deepseek-ai" | "std" => GenerateModel::new(Box::new(
            std_generate::StdGenerateModel {
                active_model: model_name.to_string(),
            },
        )),
        _ => return Err(ErrorBadRequest(format!("Unsupported {} model series!", model_series))),
    };

    // 6. Send the request to the model service
    let is_stream = false;
    let response = model.generate(req_body, username, userid, servicename, serviceid, user_type, user_level, pay_status, aicpid, is_stream).await;
    match response {
        Ok(resp) => {
            info!(target: "access_log", "{}", log_request(req.clone(), resp.status().as_u16(), None).await.unwrap());
            Ok(resp)
        }
        Err(err) => {
            error!(
                target: "error_log", "{}", log_request(req.clone(), err.as_response_error().status_code().as_u16(), Some(&format!("{}", err)))
                .await.unwrap()
            );
            Err(err)
        }
    }
}

#[post("/generate_stream")]
pub async fn generate_stream(
    req: HttpRequest,
    req_body: web::Json<GenerateRequest>,
) -> Result<impl Responder, Error> {
    // 1. Validate that required fields exist in the request data
    if req_body.model.is_empty()  {
        let error_response = "Invalid request: model  cannot be empty.";
        return Ok(HttpResponse::BadRequest().json(error_response));
    }

    // 2. Get the user ID and service name from the request extensions
    let _config = &*GLOBAL_CONFIG;
    let username = req.extensions().get::<HashMap<&str, String>>().and_then(|data| data.get("user_name").cloned()).unwrap_or_else(|| "".to_string());
    let userid = req.extensions().get::<HashMap<&str, String>>().and_then(|data| data.get("user_id").cloned()).unwrap_or_else(|| "".to_string());
    let servicename = req.extensions().get::<HashMap<&str, String>>().and_then(|data| data.get("service_name").cloned()).unwrap_or_else(|| "".to_string());
    let serviceid = req.extensions().get::<HashMap<&str, String>>().and_then(|data| data.get("service_id").cloned()).unwrap_or_else(|| "".to_string());
    let user_type = req.extensions().get::<HashMap<&str, String>>().and_then(|data| data.get("user_type").cloned()).unwrap_or_else(|| "".to_string());
    let user_level = req.extensions().get::<HashMap<&str, String>>().and_then(|data| data.get("user_level").cloned()).unwrap_or_else(|| "".to_string());
    let pay_status = req.extensions().get::<HashMap<&str, String>>().and_then(|data| data.get("pay_status").cloned()).unwrap_or_else(|| "".to_string());
    let aicpid = req.extensions().get::<HashMap<&str, String>>().and_then(|data| data.get("aicpid").cloned()).unwrap_or_else(|| "[]".to_string());

    // 3. Parse the model name and series from the model field (e.g., Qwen/Qwen2.5-7B-Instruct)
    let (model_series, model_name) = match req_body.model.split_once('/') {
        Some((series, name)) => (series, name),
        None => ("std", req_body.model.as_str()),
    };


    // 5. Call the underlying API and return a unified data format
    let model: GenerateModel = match model_series {
        "Qwen" | "GLM" | "meta-llama" | "Bailian" | "deepseek-ai" | "std" => GenerateModel::new(Box::new(
            std_generate::StdGenerateModel {
                active_model: model_name.to_string(),
            },
        )),
        _ => return Err(ErrorBadRequest(format!("Unsupported {} model series!", model_series))),
    };

    // 6. Send the request to the model service
    let is_stream = true;
    let response = model.generate(req_body, username, userid, servicename, serviceid, user_type, user_level, pay_status, aicpid,is_stream).await;
    match response {
        Ok(resp) => {
            info!(target: "access_log", "{}", log_request(req.clone(), resp.status().as_u16(), None).await.unwrap());
            Ok(resp)
        }
        Err(err) => {
            error!(
                target: "error_log", "{}", log_request(req.clone(), err.as_response_error().status_code().as_u16(), Some(&format!("{}", err)))
                .await.unwrap()
            );
            Err(err)
        }
    }
}