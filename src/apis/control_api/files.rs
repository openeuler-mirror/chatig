use actix_web::{delete, get, web, Error, HttpResponse, Responder};
use utoipa::ToSchema;

// use crate::models::chatchat::upload_temp_docs;
use crate::apis::schemas::ErrorResponse;
use crate::meta::files::{delete_file_object, get_file_object_by_id, list_file_objects};

use serde::Serialize;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(delete_file)
        .service(list_file)
        .service(get_file)
        .service(get_file_content);
}

#[derive(Serialize,ToSchema)]
pub struct DeleteFileResponse {
    id: String,
    object: String,
    deleted: bool,
}

#[utoipa::path(
    delete,  // 请求方法
    path = "/v1/files/{file_id}",  // 路径
    responses(
        (status = 200, body = DeleteFileResponse),
        (status = 404, body = ErrorResponse),
        (status = 500, body = ErrorResponse),
    )  // 响应内容
)]

// delete https://***/v1/files/{file_id}
#[delete("v1/files/{file_id}")]
async fn delete_file(
    file_id: web::Path<i32>,
) -> Result<impl Responder, Error> {
    let file_id = file_id.into_inner();

    // Check if the file exists before attempting to delete
    let file_object = get_file_object_by_id(file_id).await.map_err(|e| {
        let error_response = ErrorResponse {
            error: format!("Failed to get file object: {}", e),
        };
        actix_web::error::ErrorInternalServerError(format!("{:?}", error_response))
    })?;

    if file_object.is_none() {
        let error_response = ErrorResponse {
            error: format!("File object with id {} not found", file_id),
        };
        return Ok(HttpResponse::NotFound().json(error_response));
    }

    let _result = delete_file_object(file_id).await.map_err(|e| {
        let error_response = ErrorResponse {
            error: format!("Failed to delete file object: {}", e),
        };
        actix_web::error::ErrorInternalServerError(format!("{:?}", error_response))
    })?;

    Ok(HttpResponse::Ok().json(DeleteFileResponse {
        id: file_id.to_string(),
        object: file_object.unwrap().object,
        deleted: true,
    }))
}

#[utoipa::path(
    get,  // 请求方法
    path = "/v1/files",  // 路径
    responses(
        (status = 200, body = Vec<FileObject>),
        (status = 500, body = ErrorResponse),
    )  // 响应内容
)]

// get https://***/v1/files
#[get("v1/files")]
async fn list_file() -> Result<impl Responder, Error> {
    let file_objects = list_file_objects().await.map_err(|e| {
        let error_response = ErrorResponse {
            error: format!("Failed to list file objects: {}", e),
        };
        actix_web::error::ErrorInternalServerError(format!("{:?}", error_response))
    })?;

    Ok(HttpResponse::Ok().json(file_objects))
}

#[utoipa::path(
    get,  // 请求方法
    path = "/v1/files/{file_id}",  // 路径
    responses(
        (status = 200, body = Vec<FileObject>),
        (status = 404, body = ErrorResponse),
        (status = 500, body = ErrorResponse),
    )  // 响应内容
)]

// get https://***/v1/files/{file_id}
#[get("v1/files/{file_id}")]
async fn get_file(
    file_id: web::Path<i32>,
) -> Result<impl Responder, Error> {
    let file_id = file_id.into_inner();
    let file_object = get_file_object_by_id(file_id).await.map_err(|e| {
        let error_response = ErrorResponse {
            error: format!("Failed to get file object: {}", e),
        };
        actix_web::error::ErrorInternalServerError(format!("{:?}", error_response))
    })?;

    match file_object {
        Some(file_object) => Ok(HttpResponse::Ok().json(file_object)),
        None => {
            let error_response = ErrorResponse {
                error: format!("File object with id {} not found", file_id),
            };
            Ok(HttpResponse::NotFound().json(error_response))
        }
    }
}

#[utoipa::path(
    get,  // 请求方法
    path = "/v1/files/{file_id}/content",  // 路径
    responses(
        (status = 200, body = Vec<u8>),
        (status = 404, body = ErrorResponse),
        (status = 500, body = ErrorResponse),
    )  // 响应内容
)]

// get https://***/v1/files/{file_id}/content
#[get("v1/files/{file_id}/content")]
async fn get_file_content(
    file_id: web::Path<i32>,
) -> Result<impl Responder, Error> {
    let file_id = file_id.into_inner();
    let file_object = get_file_object_by_id(file_id).await.map_err(|e| {
        let error_response = ErrorResponse {
            error: format!("Failed to get file object: {}", e),
        };
        actix_web::error::ErrorInternalServerError(format!("{:?}", error_response))
    })?;

    if file_object.is_none() {
        let error_response = ErrorResponse {
            error: format!("File object with id {} not found", file_id),
        };
        return Ok(HttpResponse::NotFound().json(error_response));
    }

    match file_object {
        Some(file_object) => {
            let file_path = format!("{}", file_object.object);
            let file = std::fs::read(file_path).map_err(|e| {
                let error_response = ErrorResponse {
                    error: format!("Failed to read file: {}", e),
                };
                actix_web::error::ErrorInternalServerError(format!("{:?}", error_response))
            })?;
            Ok(HttpResponse::Ok().body(file))
        }
        None => {
            let error_response = ErrorResponse {
                error: format!("File object with id {} not found", file_id),
            };
            Ok(HttpResponse::NotFound().json(error_response))
        }
    }
}
