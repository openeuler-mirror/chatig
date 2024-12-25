use actix_web::{get, post, web, Error, HttpRequest, HttpResponse, Responder};
use std::collections::HashMap;

use crate::servers::api_schemas::{AppState, ErrorResponse};
use crate::database::projects::{list_project_objects, create_project_object, retrieve_project_object, 
    modify_project_object, archive_project_object, ProjectObject};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(list_projects)
        .service(create_project)
        .service(retrieve_project)
        .service(modify_project)
        .service(archive_project);
}

// list projects
#[get("/v1/organization/projects")]
async fn list_projects(headers: HttpRequest, data: web::Data<AppState>) -> Result<impl Responder, Error> {
    // 1. get parameters from query string
    let query = headers.query_string();
    let params: HashMap<String, String> = serde_urlencoded::from_str(query).unwrap_or_default();
    let after = params.get("after").cloned();
    let limit = params.get("limit").and_then(|s| s.parse::<i64>().ok()).unwrap_or(20);
    let include_archived = params.get("include_archived").map(|s| s == "true").unwrap_or(false);

    // 2. list project objects from the database
    let pool = &data.db_pool;
    let projects = list_project_objects(pool, limit, after, include_archived).await
        .map_err(|e| {
            let error_response = ErrorResponse {
                error: format!("Failed to list project objects: {}", e),
            };
            actix_web::error::ErrorInternalServerError(format!("{:?}", error_response))
        })?;

    // 3. return the list of projects
    Ok(HttpResponse::Ok().json(projects))
}

// create project
#[post("/v1/organization/projects")]
async fn create_project(data: web::Data<AppState>, project_name: web::Json<HashMap<String, String>>) -> Result<impl Responder, Error> {
    // 1. create project object
    let pool = &data.db_pool;
    let name = project_name.get("name").cloned().unwrap_or_default();
    let created_at = chrono::Utc::now().timestamp();
    let id = format!("{}_{}", name, created_at);
    let project = ProjectObject{
        id: id,
        object: "organization.project".to_string(),
        name: project_name.get("name").cloned().unwrap_or_default(),
        created_at: created_at,
        archived_at: None,
        status: "active".to_string(),
    };
    create_project_object(pool, project.clone()).await
        .map_err(|e| {
            let error_response = ErrorResponse {
                error: format!("Failed to create project object: {}", e),
            };
            actix_web::error::ErrorInternalServerError(format!("{:?}", error_response))
        })?;

    // 2. return the created project object
    Ok(HttpResponse::Ok().json(project))
}

// retrieve project
#[get("/v1/organization/projects/{project_id}")]
async fn retrieve_project(data: web::Data<AppState>, project_id: web::Path<String>) -> Result<impl Responder, Error> {
    // 1. retrieve project object
    let pool = &data.db_pool;
    let project_id = project_id.into_inner();
    let project = retrieve_project_object(pool, project_id).await
        .map_err(|e| {
            let error_response = ErrorResponse {
                error: format!("Failed to retrieve project object: {}", e),
            };
            actix_web::error::ErrorInternalServerError(format!("{:?}", error_response))
        })?;

    // 2. return the retrieved project object
    Ok(HttpResponse::Ok().json(project))
}

// modify project
#[post("/v1/organization/projects/{project_id}")]
async fn modify_project(data: web::Data<AppState>, project_id: web::Path<String>, project_name: web::Json<HashMap<String, String>>) -> Result<impl Responder, Error> {
    // 1. modify project object
    let pool = &data.db_pool;
    let project_id = project_id.into_inner();
    let name = project_name.get("name").cloned().unwrap_or_default();
    let project = modify_project_object(pool, project_id, name).await
        .map_err(|e| {
            let error_response = ErrorResponse {
                error: format!("Failed to modify project object: {}", e),
            };
            actix_web::error::ErrorInternalServerError(format!("{:?}", error_response))
        })?;

    // 2. return the modified project object
    Ok(HttpResponse::Ok().json(project))
}

// archive project
#[post("/v1/organization/projects/{project_id}/archive")]
async fn archive_project(data: web::Data<AppState>, project_id: web::Path<String>) -> Result<impl Responder, Error> {
    // 1. archive project object
    let pool = &data.db_pool;
    let project_id = project_id.into_inner();
    let project = archive_project_object(pool, project_id).await
        .map_err(|e| {
            let error_response = ErrorResponse {
                error: format!("Failed to archive project object: {}", e),
            };
            actix_web::error::ErrorInternalServerError(format!("{:?}", error_response))
        })?;

    // 2. return the archived project object
    Ok(HttpResponse::Ok().json(project))
}