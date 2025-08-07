use actix_web::{delete, error, get, post, put, web, Error, HttpResponse, Responder};
use serde::{Serialize, Deserialize};

use crate::utils::response::ApiResponse;
use crate::utils::response::ApiError::{InternalServerError, NotFound};

use crate::cores::control::services_detail::ServicesDetailManager;
use crate::meta::services_detail::traits::ServicesDetail;


pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v1/services_detail")
            .service(create_services_detail)
            .service(delete_service_detail)
            .service(update_service_detail)
            .service(get_service_detail)
            .service(get_all_service_details)
    );
}


// Create a new service detail configuration
#[post("")]
async fn create_services_detail(
    service_derail: web::Json<ServicesDetail>,
) -> Result<impl Responder, Error> {
    let service_detail_manager = ServicesDetailManager::default();
    service_detail_manager.create_service_detail(&service_derail.into_inner())
        .await
        .map(|_| {
            let success_res = ApiResponse::<()>::success("Service detail created successfully.", ());
            HttpResponse::Created().json(success_res)
        })
        .map_err(|e| {
            let err_res = ApiResponse::<String>::error(
                InternalServerError("Failed to create service detail.".to_string()), Some(format!("{}", e)));
            error::ErrorInternalServerError(err_res)
        })
}

// Delete a service detail configuration
#[delete("/{service_id}")]
async fn delete_service_detail(
    service_id: web::Path<String>,
) -> Result<impl Responder, Error> {
    let service_detail_manager = ServicesDetailManager::default();


    service_detail_manager.delete_service_detail(&service_id)
        .await
        .map(|_| {
            let success_res = ApiResponse::<()>::success("Service detail deleted successfully.", ());
            HttpResponse::Ok().json(success_res)
        })
        .map_err(|e| {
            let err_res = ApiResponse::<String>::error(
                InternalServerError("Failed to delete service detail.".to_string()), Some(format!("{}", e)));
            error::ErrorInternalServerError(err_res)
        })
}
// Update a service detail configuration
#[derive(Serialize, Deserialize)]
struct UpdateServiceDetail {
    metrics_url: Option<String>,
    health_check_url: Option<String>,
}

#[put("/{service_id}")]
async fn update_service_detail(
    service_id: web::Path<String>,
    update_service_detail: web::Json<UpdateServiceDetail>,
) -> Result<impl Responder, Error> {
    let updated_service_detail = ServicesDetail {
        service_id: service_id.into_inner(),
        metrics_url: update_service_detail.metrics_url.clone(),
        health_check_url: update_service_detail.health_check_url.clone(),
    };

    let service_detail_manager = ServicesDetailManager::default();
    service_detail_manager.update_service_detail(&updated_service_detail)
        .await
        .map(|rows_updated| {
            if rows_updated > 0 {
                let success_res = ApiResponse::<()>::success("Service detail updated successfully.", ());
                HttpResponse::Ok().json(success_res)
            } else {
                let not_found_res = ApiResponse::<()>::error(NotFound("Service detail not found.".to_string()), None);
                HttpResponse::NotFound().json(not_found_res)
            }
        })
        .map_err(|e| {
            let err_res = ApiResponse::<String>::error(
                InternalServerError("Failed to update service detail.".to_string()), Some(format!("{}", e)));
            error::ErrorInternalServerError(err_res)
        })
}

// Get a service detail configuration
#[get("/{service_id}")]
async fn get_service_detail(
    service_id: web::Path<String>,
) -> Result<impl Responder, Error> {
    let service_detail_manager = ServicesDetailManager::default();
    service_detail_manager.get_service_detail(&service_id)
        .await
        .map(|service_detail| match service_detail {
            Some(detail) => {
                let success_res = ApiResponse::<ServicesDetail>::success("Service detail retrieved successfully.", detail);
                HttpResponse::Ok().json(success_res)
            }
            None => {
                let err_res = ApiResponse::<String>::error(
                    NotFound("Service detail not found.".to_string()), None);
                HttpResponse::NotFound().json(err_res)
            }
        })
        .map_err(|e| {
            let err_res = ApiResponse::<String>::error(
                InternalServerError("Failed to retrieve service detail.".to_string()), Some(format!("{}", e)));
            error::ErrorInternalServerError(err_res)
        })
}

// Get all service detail configurations
#[get("")]
async fn get_all_service_details() -> Result<impl Responder, Error> {
    let service_detail_manager = ServicesDetailManager::default();
    service_detail_manager.get_all_service_details()
        .await
        .map(|service_details| {
            let success_res = ApiResponse::<Vec<ServicesDetail>>::success("Service details retrieved successfully.", service_details);
            HttpResponse::Ok().json(success_res)
        })
        .map_err(|e| {
            let err_res = ApiResponse::<String>::error(
                InternalServerError("Failed to retrieve service details.".to_string()), Some(format!("{}", e)));
            error::ErrorInternalServerError(err_res)
        })
}

