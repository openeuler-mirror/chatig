use actix_web::{delete, error, get, post, put, web, Error, HttpResponse, Responder};
use serde_json::json;
// use crate::cores::services::{ServiceConfig, ServiceManager};
use crate::cores::control::services::ServiceManager;
use crate::meta::services::traits::ServiceConfig;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(load_services)
        .service(create_service)
        .service(get_service)
        .service(get_all_services)
        .service(update_service)
        .service(delete_service);
}



#[post("/v1/services/load")]
pub async fn load_services() -> impl Responder {
    let service_manager = ServiceManager::default();
    match service_manager.load_services_table().await {
        Ok(_) => HttpResponse::Ok().json(json!({
            "code": 200,
            "message": "Services loaded successfully from YAML.",
            "body": null
        })),
        Err(err) => {
            eprintln!("Failed to load services: {}", err);
            HttpResponse::InternalServerError().json(json!({
                "code": 500,
                "message": "Failed to load services.",
                "body": format!("{}", err)
            }))
        }
    }
}

#[post("/v1/services")]
async fn create_service(
    service: web::Json<ServiceConfig>,
) -> Result<impl Responder, Error> {
    let service_manager = ServiceManager::default();
    service_manager.create_service(&service.into_inner())
        .await
        .map(|_| HttpResponse::Created().json(json!({
            "code": 200,
            "message": "Service created successfully.",
            "body": null
        })))
        .map_err(|e| {
            error::ErrorInternalServerError(json!({
                "code": 500,
                "message": "Failed to create service.",
                "body": format!("{}", e)
            }))
        })
}

#[get("/v1/services/{id}")]
async fn get_service(
    id: web::Path<String>,
) -> Result<impl Responder, Error> {
    let service_manager = ServiceManager::default();
    service_manager.get_service(&id)
        .await
        .map(|service| match service {
            Some(service) => HttpResponse::Ok().json(json!({
                "code": 200,
                "message": "Service get successfully.",
                "body": service
            })),
            None => HttpResponse::NotFound().json(json!({
                "code": 404,
                "message": "Service not found.",
                "body": null
            })),
        })
        .map_err(|e| {
            error::ErrorInternalServerError(json!({
                "code": 500,
                "message": "Failed to get service.",
                "body": format!("{}", e)
            }))
        })
}

#[get("/v1/services")]
async fn get_all_services() -> Result<impl Responder, Error> {
    let service_manager = ServiceManager::default();
    service_manager.get_all_services()
        .await
        .map(|services| HttpResponse::Ok().json(json!({
            "code": 200,
            "message": "All Services get successfully.",
            "body": services
        })))
        .map_err(|e| {
            error::ErrorInternalServerError(json!({
                "code": 500,
                "message": "Failed to get all services.",
                "body": format!("{}", e)
            }))
        })
}

#[put("/v1/services/{id}")]
async fn update_service(
    id: web::Path<String>,
    service: web::Json<ServiceConfig>,
) -> Result<impl Responder, Error> {
    let mut updated_service = service.into_inner();
    updated_service.id = id.clone();

    let service_manager = ServiceManager::default();
    service_manager.update_service(&updated_service)
        .await
        .map(|rows_updated| {
            if rows_updated > 0 {
                HttpResponse::Ok().json(json!({
                    "code": 200,
                    "message": "Service updated successfully.",
                    "body": null
                }))
            } else {
                HttpResponse::NotFound().json(json!({
                    "code": 404,
                    "message": "Service not found.",
                    "body": null
                }))
            }
        })
        .map_err(|e| {
            error::ErrorInternalServerError(json!({
                "code": 500,
                "message": "Failed to update service.",
                "body": format!("{}", e)
            }))
        })
}

#[delete("/v1/services/{id}")]
async fn delete_service(
    id: web::Path<String>,
) -> Result<impl Responder, Error> {
    let service_manager = ServiceManager::default();
    service_manager.delete_service(&id)
        .await
        .map(|_| HttpResponse::Ok().json(json!({
            "code": 200,
            "message": "Service deleted successfully.",
            "body": null
        })))
        .map_err(|e| {
            error::ErrorInternalServerError(json!({
                "code": 500,
                "message": "Failed to delete service.",
                "body": format!("{}", e)
            }))
        })
}
