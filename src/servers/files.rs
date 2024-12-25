use actix_web::{get, post, web, delete, Error, HttpResponse, Responder};
use actix_multipart::Multipart;

use crate::models::chatchat::upload_temp_docs;
use crate::servers::api_schemas::{AppState, ErrorResponse};
use crate::database::files::{list_file_objects, get_file_object_by_id, delete_file_object};

use serde::Serialize;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(upload_file)
       .service(delete_file)
       .service(list_file)
       .service(get_file)
       .service(get_file_content);
}

#[derive(Serialize)]
struct DeleteFileResponse {
    id: String,
    object: String,
    deleted: bool,
}

// post https://***/v1/files
#[post("v1/files")]
async fn upload_file(data: web::Data<AppState>, payload: Multipart) -> Result<impl Responder, Error> {
    // 2. Call the underlying API and return a unified data format
    let response = upload_temp_docs(payload, data).await;

    // 3. Construct the response body based on the API's return result
    match response {
        Ok(resp) => {
            Ok(resp)
        }
        Err(err) => {
            let error_response = ErrorResponse {
                error: format!("Failed to get response from upload_temp_docs: {}", err),
            };
            Ok(HttpResponse::InternalServerError().json(error_response))
        }
    }
}

// delete https://***/v1/files/{file_id}
#[delete("v1/files/{file_id}")]
async fn delete_file(data: web::Data<AppState>, file_id: web::Path<i32>) -> Result<impl Responder, Error> {
    let pool = &data.db_pool;
    let file_id = file_id.into_inner();

    // Check if the file exists before attempting to delete
    let file_object = get_file_object_by_id(pool, file_id).await
        .map_err(|e| {
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

    let _result = delete_file_object(pool, file_id).await
        .map_err(|e| {
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

// get https://***/v1/files
#[get("v1/files")]
async fn list_file(data: web::Data<AppState>) -> Result<impl Responder, Error> {
    let pool = &data.db_pool;
    let file_objects = list_file_objects(pool).await
        .map_err(|e| {
            let error_response = ErrorResponse {
                error: format!("Failed to list file objects: {}", e),
            };
            actix_web::error::ErrorInternalServerError(format!("{:?}", error_response))
        })?;
    
    Ok(HttpResponse::Ok().json(file_objects))
}

// get https://***/v1/files/{file_id}
#[get("v1/files/{file_id}")] 
async fn get_file(data: web::Data<AppState>, file_id: web::Path<i32>) -> Result<impl Responder, Error> {
    let pool = &data.db_pool;
    let file_id = file_id.into_inner();
    let file_object = get_file_object_by_id(pool, file_id).await
        .map_err(|e| {
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

// get https://***/v1/files/{file_id}/content
#[get("v1/files/{file_id}/content")]
async fn get_file_content(data: web::Data<AppState>, file_id: web::Path<i32>) -> Result<impl Responder, Error> {
    let pool = &data.db_pool;
    let file_id = file_id.into_inner();
    let file_object = get_file_object_by_id(pool, file_id).await
        .map_err(|e| {
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
            let file = std::fs::read(file_path)
                .map_err(|e| {
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