use actix_web::{get, delete, web, HttpResponse, Responder};
use serde_json::json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::cores::models::{get_model, get_models};

#[derive(Deserialize,Serialize,ToSchema)]
pub struct ModelErrorDetails {
    pub error: String,
    pub details: String,
}

#[derive(Deserialize,Serialize,ToSchema)]
pub struct ModelErrorName {
    pub error: String,
    pub model_name: String,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(models)
       .service(model_info)
       .service(delete_model);
}

#[utoipa::path(
    get,  // 请求方法
    path = "/v1/models",  // 路径
    responses(
        (status = 200, body = Vec<Model>),
        (status = 500, body = ModelErrorDetails)
    )  // 响应内容
)]

// Lists the currently available models, and provides basic information about each one such as the owner and availability.
#[get("/v1/models")]
pub async fn models() -> impl Responder {
    match get_models().await {
        Ok(models) => {
            // 成功获取模型数据，返回 JSON 响应
            HttpResponse::Ok().json(models)
        }
        Err(err) => {
            // 处理错误，返回 500 状态码和错误信息
            HttpResponse::InternalServerError().json(json!({
                "error": "Failed to fetch models",
                "details": err.to_string()
            }))
        }
    }
}

#[utoipa::path(
    get,  // 请求方法
    path = "/v1/models/{model}",  // 路径
    responses(
        (status = 200, body = Model),
        (status = 404, body = ModelErrorName),
        (status = 500, body = ModelErrorDetails)
    )  // 响应内容
    //params(("model_name",),)
)]

// Retrieves a model instance, providing basic information about the model such as the owner and permissioning.
#[get("/v1/models/{model}")]
pub async fn model_info(path: web::Path<String>) -> impl Responder {
    let model_name = path.into_inner(); // 提取路径参数
    // 调用封装的函数查询指定模型
    match get_model(&model_name).await {
        Ok(Some(model)) => {
            // 查询成功，返回模型信息
            HttpResponse::Ok().json(model)
        }
        Ok(None) => {
            // 查询结果为空，返回 404
            HttpResponse::NotFound().json(json!({
                "error": "Model not found",
                "model_name": model_name
            }))
        }
        Err(err) => {
            // 查询出现错误，返回 500
            HttpResponse::InternalServerError().json(json!({
                "error": "Failed to fetch model info",
                "details": err.to_string()
            }))
        }
    }
}

#[utoipa::path(
    delete,  // 请求方法
    path = "/v1/models/{model}",  // 路径
    responses(
        (status = 501, description = "Not implemented", body = String)
    )  // 响应内容
    //params(("model_name",),)
)]

// Delete a fine-tuned model. You must have the Owner role in your organization to delete a model.
// And we don't support this feature now.
#[delete("/v1/models/{model}")]
pub async fn delete_model(path: web::Path<String>) -> impl Responder {
    let model_name = path.into_inner();
    // log the model name
    println!("Deleting model: {}", model_name);
    HttpResponse::NotImplemented().body("Not implemented")
}