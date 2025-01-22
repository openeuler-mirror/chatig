use actix_web::{delete, get, post, put, web, Error, HttpResponse, Responder};
use actix_web::error::ErrorInternalServerError;
use serde_json::json;

use crate::cores::control::files::FileManager;
use crate::meta::files::traits::File;


pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(create_file)
        .service(delete_file)
        .service(get_all_files)
        .service(get_file);
}

#[utoipa::path(
    post,  // 请求方法
    path = "/v1/files",  // 路径
    responses(
        (status = 200, body = HttpResponse),
        (status = 404, body = ErrorResponse),
        (status = 500, body = ErrorInternalServerError),
    )  // 响应内容
)]

#[post("v1/files")]
pub async fn create_file(
    file: web::Json<File>,
) -> Result<impl Responder, Error> {
    let file_manager = FileManager::default();
    file_manager.add_file_object(file.into_inner()).await
    .map(|_| {
        let create_file_response = json!({
            "code": 200,
            "message": "File object created successfully.",
            "body": null
        });
        HttpResponse::Created().json(create_file_response)
    })
    .map_err(|e| {
        let error_response = json!({
            "code": 500,
            "message": "Failed to create file object.",
            "body": format!("{}", e),
        });
        ErrorInternalServerError(error_response)
    })
}

#[utoipa::path(
    delete,  // 请求方法
    path = "/v1/files/{file_id}",  // 路径
    responses(
        (status = 200, body = HttpResponse),
        (status = 404, body = ErrorResponse),
        (status = 500, body = ErrorResponse),
    )  // 响应内容
)]

// delete https://***/v1/files/{file_id}
#[delete("v1/files/{file_id}")]
async fn delete_file(
    file_id: web::Path<String>,
) -> Result<impl Responder, Error> {
    let file_manager = FileManager::default();
    file_manager.delete_file_object(&file_id).await
    .map(|_| {
        let delete_file_response = json!({
            "code": 200,
            "message": "File object deleted successfully.",
            "body": null
        });
        HttpResponse::Ok().json(delete_file_response)
    })
    .map_err(|e| {
        let error_response = json!({
            "code": 500,
            "message": "Failed to delete file object.",
            "body": format!("{}", e),
        });
        ErrorInternalServerError(error_response)
    })
}

#[put("v1/files/{file_id}")]
async fn update_file(
    file: web::Json<File>,
) -> Result<impl Responder, Error> {
    let file_manager = FileManager::default();
    file_manager.update_file_object(file.into_inner()).await
    .map(|rows_updated| {
        if rows_updated > 0{
            let update_file_response = json!({
                "code": 200,
                "message": "File object updated successfully.",
                "body": null
            });
            HttpResponse::Ok().json(update_file_response)
        } else {
            let error_response = json!({
                "code": 404,
                "message": "File object not found.",
                "body": null,
            });
            HttpResponse::NotFound().json(error_response)
        }
    })
    .map_err(|e| {
        let error_response = json!({
            "code": 500,
            "message": "Failed to update file object.",
            "body": format!("{}", e),
        });
        ErrorInternalServerError(error_response)
    })
}

#[utoipa::path(
    get,  // 请求方法
    path = "/v1/files",  // 路径
    responses(
        (status = 200, body = HttpResponse),
        (status = 500, body = ErrorResponse),
    )  // 响应内容
)]

// get https://***/v1/files
#[get("v1/files")]
async fn get_all_files() -> Result<impl Responder, Error> {
    let file_manager = FileManager::default();
    file_manager.get_all_file_objects().await
    .map(|files| {
        let response = json!({
            "code": 200,
            "message": "All file objects fetched successfully.",
            "body": files,
        });
        HttpResponse::Ok().json(response)
    })
    .map_err(|e| {
        let error_response = json!({
            "code": 500,
            "message": "Failed to fetch all file objects.",
            "body": format!("{}", e),
        });
        ErrorInternalServerError(error_response)
    })
}

#[utoipa::path(
    get,  // 请求方法
    path = "/v1/files/{file_id}",  // 路径
    responses(
        (status = 200, body = HttpResponse),
        (status = 404, body = ErrorResponse),
        (status = 500, body = ErrorResponse),
    )  // 响应内容
)]

// get https://***/v1/files/{file_id}
#[get("v1/files/{file_id}")]
async fn get_file(
    file_id: web::Path<String>,
) -> Result<impl Responder, Error> {
    let file_manager = FileManager::default();
    file_manager.get_file_object(&file_id).await
    .map(|file|match file {
        Some(file) => {
            let response = json!({
                "code": 200,
                "message": "File object fetched successfully.",
                "body": file,
            });
            HttpResponse::Ok().json(response)
        },
        None => {
            let error_response = json!({
                "code": 404,
                "message": "File object not found.",
                "body": null,
            });
            HttpResponse::NotFound().json(error_response)
        }
    })
    .map_err(|e| {
        let error_response = json!({
            "code": 500,
            "message": "Failed to fetch file object.",
            "body": format!("{}", e),
        });
        ErrorInternalServerError(error_response)
    })
}


