use actix_web::{delete, get, post, put, web, Error, HttpResponse, Responder};
use actix_web::error::ErrorInternalServerError;

use crate::utils::response::ApiResponse;
use crate::utils::response::ApiError::{InternalServerError, NotFound};

use crate::cores::control::files::FileManager;
use crate::meta::files::traits::File;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v1/files")
            .service(create_file)
            .service(delete_file)
            .service(get_all_files)
            .service(get_file),
    );
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

#[post("")]
pub async fn create_file(
    file: web::Json<File>,
) -> Result<impl Responder, Error> {
    let file_manager = FileManager::default();
    file_manager.add_file_object(file.into_inner()).await
    .map(|_| {
        let success_res = ApiResponse::<()>::success("File object created successfully.", ());
        HttpResponse::Created().json(success_res)
    })
    .map_err(|e| {
        let error_res = ApiResponse::<String>::error(
            InternalServerError("Failed to create file object.".to_string()), Some(format!("{}", e)));
        ErrorInternalServerError(error_res)
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
#[delete("/{file_id}")]
async fn delete_file(
    file_id: web::Path<String>,
) -> Result<impl Responder, Error> {
    let file_manager = FileManager::default();
    file_manager.delete_file_object(&file_id).await
    .map(|_| {
        let success_res = ApiResponse::<()>::success("File object deleted successfully.", ());
        HttpResponse::Ok().json(success_res)
    })
    .map_err(|e| {
        let error_res = ApiResponse::<String>::error(
            InternalServerError("Failed to delete file object.".to_string()), Some(format!("{}", e)));
        ErrorInternalServerError(error_res)
    })
}

#[put("/{file_id}")]
async fn update_file(
    file: web::Json<File>,
) -> Result<impl Responder, Error> {
    let file_manager = FileManager::default();
    file_manager.update_file_object(file.into_inner()).await
    .map(|rows_updated| {
        if rows_updated > 0{
            let success_res = ApiResponse::<()>::success("File object updated successfully.", ());
            HttpResponse::Ok().json(success_res)
        } else {
            let not_found_res = ApiResponse::<()>::error(
                NotFound("File object not found.".to_string()), None);
            HttpResponse::NotFound().json(not_found_res)
        }
    })
    .map_err(|e| {
        let error_res = ApiResponse::<String>::error(
            InternalServerError("Failed to update file object.".to_string()), Some(format!("{}", e)));
        ErrorInternalServerError(error_res)
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
#[get("")]
async fn get_all_files() -> Result<impl Responder, Error> {
    let file_manager = FileManager::default();
    file_manager.get_all_file_objects().await
    .map(|files| {
        let success_res = ApiResponse::<Vec<File>>::success("All file objects fetched successfully.", files);
        HttpResponse::Ok().json(success_res)
    })
    .map_err(|e| {
        let error_res = ApiResponse::<String>::error(
            InternalServerError("Failed to fetch all file objects.".to_string()), Some(format!("{}", e)));
        ErrorInternalServerError(error_res)
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
#[get("/{file_id}")]
async fn get_file(
    file_id: web::Path<String>,
) -> Result<impl Responder, Error> {
    let file_manager = FileManager::default();
    file_manager.get_file_object(&file_id).await
    .map(|file|match file {
        Some(file) => {
            let success_res = ApiResponse::<File>::success("File object fetched successfully.", file);
            HttpResponse::Ok().json(success_res)
        },
        None => {
            let error_res = ApiResponse::<()>::error(
                NotFound("File object not found.".to_string()), None);
            HttpResponse::NotFound().json(error_res)
        }
    })
    .map_err(|e| {
        let error_res = ApiResponse::<String>::error(
            InternalServerError("Failed to fetch file object.".to_string()), Some(format!("{}", e)));
        ErrorInternalServerError(error_res)
    })
}


