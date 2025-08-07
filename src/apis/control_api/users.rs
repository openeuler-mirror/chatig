use actix_web::{delete, get, post, web, Error, HttpRequest, HttpResponse, Responder};
use serde::Serialize;
use std::collections::HashMap;

use crate::cores::control::users::Usermanager;
use crate::meta::users::traits::{UserObject, UserObjectDto};
use crate::utils::response::ApiError::{InternalServerError, NotFound};
use crate::utils::response::ApiResponse;

#[derive(Serialize)]
struct DeleteUserResponse {
    id: String,
    object: String,
    deleted: bool,
}

#[allow(dead_code)]
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v1/organization/users")
            .service(list_users)
            .service(modify_user)
            .service(retrieve_user)
            .service(delete_user)
            .service(create_user)
    );
}

// Create user
#[post("")]
async fn create_user(user_dto: web::Json<UserObjectDto>) -> Result<impl Responder, Error> {
    let user_manager = Usermanager::default();
    user_manager
        .insert_user_object(user_dto.into_inner())
        .await
        .map(|_| {
            let success_res = ApiResponse::<()>::success("User created successfully.", ());
            HttpResponse::Created().json(success_res)
        })
        .map_err(|e| {
            let error_response = format!("Failed to create user object: {}", e);
            actix_web::error::ErrorInternalServerError(format!("{:?}", error_response))
        })
}

// List users
#[get("")]
async fn list_users(headers: HttpRequest) -> Result<impl Responder, Error> {
    let query = headers.query_string();
    let params: HashMap<String, String> = serde_urlencoded::from_str(query).unwrap_or_default();
    let after = params.get("after").cloned();
    let limit = params
        .get("limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(20);

    let user_manager = Usermanager::default();
    user_manager
        .list_user_objects(limit, after)
        .await
        .map(|users| {
            let success_res =
                ApiResponse::<Vec<UserObject>>::success("User objects retrieved successfully.", users);
            HttpResponse::Ok().json(success_res)
        })
        .map_err(|e| {
            let err_res = ApiResponse::<String>::error(
                InternalServerError("Failed to list user objects.".to_string()),
                Some(format!("{}", e)),
            );
            actix_web::error::ErrorInternalServerError(err_res)
        })
}

// Modify user
#[post("/{user_id}")]
async fn modify_user(
    user_id: web::Path<String>,
    role: web::Json<HashMap<String, String>>,
) -> Result<impl Responder, Error> {
    let user_id = user_id.into_inner();
    let role = role.get("role").cloned().unwrap_or_default();

    let user_manager = Usermanager::default();
    user_manager
        .modify_user_object(user_id.clone(), role)
        .await
        .map(|user| {
            let success_res =
                ApiResponse::<UserObject>::success("User object modified successfully.", user);
            HttpResponse::Ok().json(success_res)
        })
        .map_err(|e| {
            let err_res = ApiResponse::<String>::error(
                InternalServerError("Failed to modify user object.".to_string()),
                Some(format!("{}", e)),
            );
            actix_web::error::ErrorInternalServerError(err_res)
        })
}

// Retrieve user
#[get("/{user_id}")]
async fn retrieve_user(user_id: web::Path<String>) -> Result<impl Responder, Error> {
    let user_manager = Usermanager::default();
    let user_id = user_id.into_inner();
    user_manager
        .retrieve_user_object(user_id.clone())
        .await
        .map(|user| match user {
            Some(user) => {
                let success_res =
                    ApiResponse::<UserObject>::success("User object retrieved successfully.", user);
                HttpResponse::Ok().json(success_res)
            }
            None => {
                let not_found_res =
                    ApiResponse::<()>::error(NotFound("User object not found.".to_string()), None);
                HttpResponse::NotFound().json(not_found_res)
            }
        })
        .map_err(|e| {
            let err_res = ApiResponse::<String>::error(
                InternalServerError("Failed to retrieve user object.".to_string()),
                Some(format!("{}", e)),
            );
            actix_web::error::ErrorInternalServerError(err_res)
        })
}

// Delete user
#[delete("/{user_id}")]
async fn delete_user(user_id: web::Path<String>) -> Result<impl Responder, Error> {
    let user_manager = Usermanager::default();
    let user_id = user_id.into_inner();
    let user_id_clone = user_id.clone();

    user_manager
        .delete_user_object(user_id_clone)
        .await
        .map(|delete_num| {
            if delete_num == 0 {
                let not_found_res =
                    ApiResponse::<()>::error(NotFound("User object not found.".to_string()), None);
                HttpResponse::NotFound().json(not_found_res)
            } else {
                let success_res = ApiResponse::<DeleteUserResponse>::success(
                    "User object deleted successfully.",
                    DeleteUserResponse {
                        id: user_id,
                        object: "organization.user".into(),
                        deleted: true,
                    },
                );
                HttpResponse::Ok().json(success_res)
            }
        })
        .map_err(|e| {
            let err_res = ApiResponse::<String>::error(
                InternalServerError("Failed to delete user object.".to_string()),
                Some(format!("{}", e)),
            );
            actix_web::error::ErrorInternalServerError(err_res)
        })
}