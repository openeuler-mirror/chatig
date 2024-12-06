use actix_web::{get, delete, post, web, Error, HttpRequest, HttpResponse, Responder};
use std::collections::HashMap;

use crate::utils::check_api_key;
use crate::servers::api_schemas::{AppState, ErrorResponse};
use crate::database::users::{insert_user_object, list_user_objects, modify_user_object, retrieve_user_object, delete_user_object, UserObjectDto};

use serde::Serialize;

#[derive(Serialize)]
struct DeleteUserResponse {
    id: String,
    object: String,
    deleted: bool,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(list_users)
        .service(modify_user)
        .service(retrieve_user)
        .service(delete_user)
        .service(create_user);
}

// create user
#[post("/v1/organization/users")]
async fn create_user(headers: HttpRequest, data: web::Data<AppState>, user: web::Json<UserObjectDto>) -> Result<impl Responder, Error> {
    // 0. Check if the API Key in the request headers matches the config
    let config = &data.config;
    if !check_api_key(headers.clone(), config) {
        let error_response = ErrorResponse {
            error: "Invalid API Key.".into(),
        };
        return Ok(HttpResponse::Unauthorized().json(error_response));
    }

    // 1. create user object in the database
    let pool = &data.db_pool;

    let user = insert_user_object(pool, user.into_inner()).await
        .map_err(|e| {
            let error_response = ErrorResponse {
                error: format!("Failed to create user object: {}", e),
            };
            actix_web::error::ErrorInternalServerError(format!("{:?}", error_response))
        })?;

    // 2. return success
    Ok(HttpResponse::Ok().json(user))
}

// list users
#[get("/v1/organization/users")]
async fn list_users(headers: HttpRequest, data: web::Data<AppState>) -> Result<impl Responder, Error> {
    // 0. Check if the API Key in the request headers matches the config
    let config = &data.config;
    if !check_api_key(headers.clone(), config) {
        let error_response = ErrorResponse {
            error: "Invalid API Key.".into(),
        };
        return Ok(HttpResponse::Unauthorized().json(error_response));
    }

    // 1. get parameters from query string
    let query = headers.query_string();
    let params: HashMap<String, String> = serde_urlencoded::from_str(query).unwrap_or_default();
    let after = params.get("after").cloned();
    let limit = params.get("limit").and_then(|s| s.parse::<i64>().ok()).unwrap_or(20);

    // 2. list user objects from the database
    let pool = &data.db_pool;
    let users = list_user_objects(pool, limit, after).await
        .map_err(|e| {
            let error_response = ErrorResponse {
                error: format!("Failed to list user objects: {}", e),
            };
            actix_web::error::ErrorInternalServerError(format!("{:?}", error_response))
        })?;

    // 3. return the list of users
    Ok(HttpResponse::Ok().json(users))
}


// modify user
#[post("/v1/organization/users/{user_id}")]
async fn modify_user(headers: HttpRequest, data: web::Data<AppState>, user_id: web::Path<String>, role: web::Json<HashMap<String, String>>) -> Result<impl Responder, Error> {
    // 0. Check if the API Key in the request headers matches the config
    let config = &data.config;
    if !check_api_key(headers.clone(), config) {
        let error_response = ErrorResponse {
            error: "Invalid API Key.".into(),
        };
        return Ok(HttpResponse::Unauthorized().json(error_response));
    }

    // 1. modify user object in the database
    let pool = &data.db_pool;
    let user_id = user_id.into_inner();
    let role = role.get("role").cloned().unwrap_or_default();

    let user = modify_user_object(pool, user_id, role).await
        .map_err(|e| {
            let error_response = ErrorResponse {
                error: format!("Failed to modify user object: {}", e),
            };
            actix_web::error::ErrorInternalServerError(format!("{:?}", error_response))
        })?;

    // 2. return success
    Ok(HttpResponse::Ok().json(user))
}


// retrieve user
#[get("/v1/organization/users/{user_id}")]
async fn retrieve_user(headers: HttpRequest, data: web::Data<AppState>, user_id: web::Path<String>) -> Result<impl Responder, Error> {
    // 0. Check if the API Key in the request headers matches the config
    let config = &data.config;
    if !check_api_key(headers.clone(), config) {
        let error_response = ErrorResponse {
            error: "Invalid API Key.".into(),
        };
        return Ok(HttpResponse::Unauthorized().json(error_response));
    }

    // 1. retrieve user object from the database
    let pool = &data.db_pool;
    let user_id = user_id.into_inner();
    let user = retrieve_user_object(pool, user_id).await
        .map_err(|e| {
            let error_response = ErrorResponse {
                error: format!("Failed to retrieve user object: {}", e),
            };
            actix_web::error::ErrorInternalServerError(format!("{:?}", error_response))
        })?;

    // 2. return the user object
    Ok(HttpResponse::Ok().json(user))
}

// delete user
#[delete("/v1/organization/users/{user_id}")]
async fn delete_user(headers: HttpRequest, data: web::Data<AppState>, user_id: web::Path<String>) -> Result<impl Responder, Error> {
    // 0. Check if the API Key in the request headers matches the config
    let config = &data.config;
    if !check_api_key(headers.clone(), config) {
        let error_response = ErrorResponse {
            error: "Invalid API Key.".into(),
        };
        return Ok(HttpResponse::Unauthorized().json(error_response));
    }

    // 1. delete user object from the database
    let pool = &data.db_pool;
    let user_id = user_id.into_inner();
    delete_user_object(pool, user_id.clone()).await
        .map_err(|e| {
            let error_response = ErrorResponse {
                error: format!("Failed to delete user object: {}", e),
            };
            actix_web::error::ErrorInternalServerError(format!("{:?}", error_response))
        })?;

    // 2. return success
    Ok(HttpResponse::Ok().json(DeleteUserResponse {
        id: user_id,
        object: "organization.user".into(),
        deleted: true,
    }))
}


  