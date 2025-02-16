use actix_web::{delete, get, post, put, web, Error, HttpResponse, Responder};
use actix_web::error::ErrorInternalServerError;
use serde_json::json;
use std::sync::Arc;

use crate::cores::control::model_limits::LimitsManager;
use crate::meta::qos::traits::Limits;
use crate::middleware::auth4manage::Auth4ManageMiddleware;

pub fn configure(cfg: &mut web::ServiceConfig, auth_middleware: Arc<Auth4ManageMiddleware>) {
    cfg.service(
        web::scope("/v1/limits")
            .wrap(auth_middleware) // 应用中间件
            .service(create_model_limits)
            .service(delete_model_limits)
            .service(update_model_limits)
            .service(get_all_model_limits)
            .service(get_model_limits),
    );
}

#[utoipa::path(
    post,
    path = "/v1/limits",
    responses(
        (status = 200, body = HttpResponse),
        (status = 404, body = ErrorResponse),
        (status = 500, body = ErrorInternalServerError),
    )
)]

#[post("")]
pub async fn create_model_limits(
    limits: web::Json<Limits>,
) -> Result<impl Responder, Error> {
    let limits_manager = LimitsManager::default();
    limits_manager.add_limits_object(limits.into_inner()).await
    .map(|_| {
        let create_model_limits_response = json!({
            "code": 200,
            "message": "Model limits object created successfully.",
            "body": null
        });
        HttpResponse::Created().json(create_model_limits_response)
    })
    .map_err(|e| {
        let error_response = json!({
            "code": 500,
            "message": "Failed to create Model limits object.",
            "body": format!("{}", e),
        });
        ErrorInternalServerError(error_response)
    })
}

#[utoipa::path(
    delete,
    path = "/v1/limits/{model_name}",
    responses(
        (status = 200, body = HttpResponse),
        (status = 404, body = ErrorResponse),
        (status = 500, body = ErrorResponse),
    )
)]

#[delete("/{model_name}")]
async fn delete_model_limits(
    model_name: web::Path<String>,
) -> Result<impl Responder, Error> {
    let limits_manager = LimitsManager::default();
    limits_manager.delete_limits_object(&model_name).await
    .map(|_| {
        let delete_model_limits_response = json!({
            "code": 200,
            "message": "Model limits object deleted successfully.",
            "body": null
        });
        HttpResponse::Ok().json(delete_model_limits_response)
    })
    .map_err(|e| {
        let error_response = json!({
            "code": 500,
            "message": "Failed to delete Model limits object.",
            "body": format!("{}", e),
        });
        ErrorInternalServerError(error_response)
    })
}

#[put("/{model_name:.*}")]
async fn update_model_limits(
    limits: web::Json<Limits>,
) -> Result<impl Responder, Error> {
    let limits_manager = LimitsManager::default();
    limits_manager.update_limits_object(limits.into_inner()).await
    .map(|rows_updated| {
        if rows_updated > 0{
            let update_model_limits_response = json!({
                "code": 200,
                "message": "Model limits object updated successfully.",
                "body": null
            });
            HttpResponse::Ok().json(update_model_limits_response)
        } else {
            let error_response = json!({
                "code": 404,
                "message": "Model limits object not found.",
                "body": null,
            });
            HttpResponse::NotFound().json(error_response)
        }
    })
    .map_err(|e| {
        let error_response = json!({
            "code": 500,
            "message": "Failed to update Model limits object.",
            "body": format!("{}", e),
        });
        ErrorInternalServerError(error_response)
    })
}

#[utoipa::path(
    get,
    path = "/v1/limits",
    responses(
        (status = 200, body = HttpResponse),
        (status = 500, body = ErrorResponse),
    )
)]

// get https://***/v1/limits
#[get("")]
async fn get_all_model_limits() -> Result<impl Responder, Error> {
    let limits_manager = LimitsManager::default();
    limits_manager.get_all_limits_objects().await
    .map(|limits| {
        let response = json!({
            "code": 200,
            "message": "All Model limits objects fetched successfully.",
            "body": limits,
        });
        HttpResponse::Ok().json(response)
    })
    .map_err(|e| {
        let error_response = json!({
            "code": 500,
            "message": "Failed to fetch all Model limits objects.",
            "body": format!("{}", e),
        });
        ErrorInternalServerError(error_response)
    })
}

#[utoipa::path(
    get,
    path = "/v1/limits/{model_name}",
    responses(
        (status = 200, body = HttpResponse),
        (status = 404, body = ErrorResponse),
        (status = 500, body = ErrorResponse),
    )
)]

// get https://***/v1/limits/{model_name}
#[get("/{model_name:.*}")]
async fn get_model_limits(
    model_name: web::Path<String>,
) -> Result<impl Responder, Error> {
    let limits_manager = LimitsManager::default();
    limits_manager.get_limits_object(&model_name).await
    .map(|limit|match limit {
        Some(limit) => {
            let response = json!({
                "code": 200,
                "message": "Model limits object fetched successfully.",
                "body": limit,
            });
            HttpResponse::Ok().json(response)
        },
        None => {
            let error_response = json!({
                "code": 404,
                "message": "Model limits object not found.",
                "body": null,
            });
            HttpResponse::NotFound().json(error_response)
        }
    })
    .map_err(|e| {
        let error_response = json!({
            "code": 500,
            "message": "Failed to fetch Model limits object.",
            "body": format!("{}", e),
        });
        ErrorInternalServerError(error_response)
    })
}


