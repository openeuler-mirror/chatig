use actix_web::{delete, error, get, post, put, web, Error, HttpResponse, Responder};

use serde::{Deserialize, Serialize};
use crate::cores::control::services::ServiceManager;

use crate::utils::response::ApiResponse;
use crate::utils::response::ApiError::{InternalServerError, NotFound};

use crate::meta::services::traits::Services;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServicesRequest {
    pub id: String,
    pub servicetype: String,
    pub status: String, // active or inactive
    pub url: String,
    pub model_name: String,
    pub active_model: String,
    pub api_key: String,
    pub context_length: f32,
    pub tags: Option<Vec<String>>,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v1/services")
            .service(load_services)
            .service(create_service)
            .service(get_service)
            .service(get_all_services)
            .service(update_service)
            .service(delete_service),
    );
}

#[post("/load")]
pub async fn load_services() -> impl Responder {
    let service_manager = ServiceManager::default();
    match service_manager.load_services_table().await {
        Ok(_) => {
            let success_res = ApiResponse::<()>::success("Services loaded successfully from YAML.", ());
            HttpResponse::Ok().json(success_res)
        }
        Err(err) => {
            let error_res = ApiResponse::<String>::error(
                InternalServerError("Failed to load services from YAML.".to_string()), Some(format!("{}", err)));
            HttpResponse::InternalServerError().json(error_res)
        }
    }
}

#[post("")]
async fn create_service(
    service_request: web::Json<ServicesRequest>,
) -> Result<impl Responder, Error> {
    let service_manager = ServiceManager::default();
    let service_request_inner = service_request.into_inner();

    let service = Services {
        id: service_request_inner.id,
        servicetype: service_request_inner.servicetype,
        status: service_request_inner.status,
        url: service_request_inner.url,
        model_name: service_request_inner.model_name,
        active_model: service_request_inner.active_model,
        api_key: service_request_inner.api_key,
        context_length: service_request_inner.context_length,
        tags: Some(service_request_inner.tags.unwrap_or_default().join(",")),
    };

    service_manager.create_service(&service)
        .await
        .map(|_| {
            let success_res = ApiResponse::<()>::success("Service created successfully.", ());
            HttpResponse::Created().json(success_res)
        })
        .map_err(|e| {
            let err_res = ApiResponse::<String>::error(
                InternalServerError("Failed to create service.".to_string()), Some(format!("{}", e)));
            error::ErrorInternalServerError(err_res)
        })
}

#[get("/{id}")]
async fn get_service(
    id: web::Path<String>,
) -> Result<impl Responder, Error> {
    let service_manager = ServiceManager::default();
    service_manager.get_service(&id)
        .await
        .map(|service| match service {
            Some(service) => {
                let services_request = ServicesRequest{
                    id: service.id,
                    servicetype: service.servicetype,
                    status: service.status,
                    url: service.url,
                    model_name: service.model_name,
                    active_model: service.active_model,
                    api_key: service.api_key,
                    context_length: service.context_length,
                    tags: Some(service.tags.unwrap_or_default().split(",").map(String::from).collect()),
                };
                let success_res = ApiResponse::<ServicesRequest>::success("Service get successfully.", services_request);
                HttpResponse::Ok().json(success_res)
            },
            None => {
                let not_found_res = ApiResponse::<()>::error(NotFound("Service not found.".to_string()), None);
                HttpResponse::NotFound().json(not_found_res)
            }
        })
        .map_err(|e| {
            let err_res = ApiResponse::<String>::error(
                InternalServerError("Failed to get service.".to_string()), Some(format!("{}", e)));
            error::ErrorInternalServerError(err_res)
        })
}

#[get("")]
async fn get_all_services() -> Result<impl Responder, Error> {
    let service_manager = ServiceManager::default();
    service_manager.get_all_services()
        .await
        .map(|services| {
            let mut requests = Vec::new(); // 初始化目标 Vec
            for service in services {
                requests.push(ServicesRequest {
                    id: service.id,
                    servicetype: service.servicetype,
                    status: service.status,
                    url: service.url,
                    model_name: service.model_name,
                    active_model: service.active_model,
                    api_key: service.api_key,
                    context_length: service.context_length,
                    tags: Some(service.tags.unwrap_or_default().split(",").map(String::from).collect()),
                });
            }
    
            let success_res = ApiResponse::<Vec<ServicesRequest>>::success("All Services get successfully.", requests);
            HttpResponse::Ok().json(success_res)
        })
        .map_err(|e| {
            let err_res = ApiResponse::<String>::error(
                InternalServerError("Failed to get all services.".to_string()), Some(format!("{}", e)));
            error::ErrorInternalServerError(err_res)
        })
}

#[put("/{id}")]
async fn update_service(
    id: web::Path<String>,
    service: web::Json<ServicesRequest>,
) -> Result<impl Responder, Error> {
    let updated_service = service.into_inner();
    let service = Services {
        id: id.clone(),
        servicetype: updated_service.servicetype,
        status: updated_service.status,
        url: updated_service.url,
        model_name: updated_service.model_name,
        active_model: updated_service.active_model,
        api_key: updated_service.api_key,
        context_length: updated_service.context_length,
        tags: Some(updated_service.tags.unwrap_or_default().join(",")),
    };

    let service_manager = ServiceManager::default();
    service_manager.update_service(&service)
        .await
        .map(|rows_updated| {
            if rows_updated > 0 {
                let success_res = ApiResponse::<()>::success("Service updated successfully.", ());
                HttpResponse::Ok().json(success_res)
            } else {
                let not_found_res = ApiResponse::<()>::error(NotFound("Service not found.".to_string()), None);
                HttpResponse::NotFound().json(not_found_res)
            }
        })
        .map_err(|e| {
            let err_res = ApiResponse::<String>::error(
                InternalServerError("Failed to update service.".to_string()), Some(format!("{}", e)));
            error::ErrorInternalServerError(err_res)
        })
}

#[delete("/{id}")]
async fn delete_service(
    id: web::Path<String>,
) -> Result<impl Responder, Error> {
    let service_manager = ServiceManager::default();
    service_manager.delete_service(&id)
        .await
        .map(|delete_num| 
            if delete_num == 0 {
                let not_found_res = ApiResponse::<()>::error(NotFound("Service not found.".to_string()), None);
                HttpResponse::NotFound().json(not_found_res)
            } else {
                let success_res = ApiResponse::<()>::success("Service deleted successfully.", ());
                HttpResponse::Ok().json(success_res)
            }
        ).map_err(|e| {
            let err_res = ApiResponse::<String>::error(
                InternalServerError("Failed to delete service.".to_string()), Some(format!("{}", e)));
            error::ErrorInternalServerError(err_res)
        })
}