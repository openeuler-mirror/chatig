use actix_web::{get, post, web, Error, HttpRequest, HttpResponse, Responder};
use std::collections::HashMap;

use crate::cores::control::projects::Projectmanager;
use crate::meta::projects::traits::Project;

#[allow(dead_code)]
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/v1/organization/projects")
            .service(list_projects)
            .service(create_project)
            .service(retrieve_project)
            .service(modify_project)
            .service(archive_project)
    );
}

// list projects
#[get("")]
async fn list_projects(headers: HttpRequest) -> Result<impl Responder, Error> {
    // 1. get parameters from query string
    let query = headers.query_string();
    let params: HashMap<String, String> = serde_urlencoded::from_str(query).unwrap_or_default();
    let after = params.get("after").cloned();
    let limit = params.get("limit").and_then(|s| s.parse::<i64>().ok()).unwrap_or(20);
    let include_archived = params.get("include_archived").map(|s| s == "true").unwrap_or(false);

    // 2. list project objects from the database
    let project_manager = Projectmanager::default();
    let projects = project_manager.list_projects(limit, after, include_archived).await
        .map_err(|e| {
            let error_response = format!("Failed to list project objects: {}", e);
            actix_web::error::ErrorInternalServerError(format!("{:?}", error_response))
        })?;

    // 3. return the list of projects
    Ok(HttpResponse::Ok().json(projects))
}

// create project
#[post("")]
async fn create_project(project_name: web::Json<HashMap<String, String>>) -> Result<impl Responder, Error> {
    // 1. create project object
    let name = project_name.get("name").cloned().unwrap_or_default();
    let created_at = chrono::Utc::now().timestamp();
    let id = format!("{}_{}", name, created_at);
    let project = Project{
        id: id,
        object: "organization.project".to_string(),
        name: project_name.get("name").cloned().unwrap_or_default(),
        created_at: created_at,
        archived_at: None,
        status: "active".to_string(),
    };

    let project_manager = Projectmanager::default();
    project_manager.create_project(project.clone()).await
        .map_err(|e| {
            let error_response = format!("Failed to create project object: {}", e);
            actix_web::error::ErrorInternalServerError(format!("{:?}", error_response))
        })?;

    // 2. return the created project object
    Ok(HttpResponse::Ok().json(project))
}

// retrieve project
#[get("/{project_id}")]
async fn retrieve_project(project_id: web::Path<String>) -> Result<impl Responder, Error> {
    // 1. retrieve project object
    let project_id = project_id.into_inner();
    let project_manager = Projectmanager::default();
    let project = project_manager.retrieve_project(project_id).await
        .map_err(|e| {
            let error_response = format!("Failed to retrieve project object: {}", e);
            actix_web::error::ErrorInternalServerError(format!("{:?}", error_response))
        })?;

    // 2. return the retrieved project object
    Ok(HttpResponse::Ok().json(project))
}

// modify project
#[post("/{project_id}")]
async fn modify_project(project_id: web::Path<String>, project_name: web::Json<HashMap<String, String>>) -> Result<impl Responder, Error> {
    // 1. modify project object
    let project_id = project_id.into_inner();
    let name = project_name.get("name").cloned().unwrap_or_default();
    let project_manager = Projectmanager::default();
    let project = project_manager.modify_project(project_id, name).await
        .map_err(|e| {
            let error_response = format!("Failed to modify project object: {}", e);
            actix_web::error::ErrorInternalServerError(format!("{:?}", error_response))
        })?;

    // 2. return the modified project object
    Ok(HttpResponse::Ok().json(project))
}

// archive project
#[post("/{project_id}/archive")]
async fn archive_project(project_id: web::Path<String>) -> Result<impl Responder, Error> {
    // 1. archive project object
    let project_id = project_id.into_inner();
    let project_manager = Projectmanager::default();
    let project = project_manager.archive_project(project_id).await
        .map_err(|e| {
            let error_response = format!("Failed to archive project object: {}", e);
            actix_web::error::ErrorInternalServerError(format!("{:?}", error_response))
        })?;

    // 2. return the archived project object
    Ok(HttpResponse::Ok().json(project))
}