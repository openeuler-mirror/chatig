use actix_web::{get, delete, web, HttpResponse, Responder};

use crate::apis::control_api::schemas::Model;


pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(models)
       .service(model_info)
       .service(delete_model);
}

pub fn get_models() -> Vec<Model> {
    let static_models: Vec<Model> = vec![
        Model {
        id: "chatchat".to_string(),
        object: "model".to_string(),
        created: 1686935002,  
        owned_by: "culinux".to_string(),
        },
        Model {
            id: "copilot".to_string(),
            object: "model".to_string(),
            created: 1686935002,
            owned_by: "openeuler".to_string(),
        },
    ];
    static_models
}

// Lists the currently available models, and provides basic information about each one such as the owner and availability.
#[get("/v1/models")]
pub async fn models() -> impl Responder {
    let models = get_models();
    HttpResponse::Ok().json(models)
}

// Retrieves a model instance, providing basic information about the model such as the owner and permissioning.
#[get("/v1/models/{model}")]
pub async fn model_info(path: web::Path<String>) -> impl Responder {
    let model_name = path.into_inner();
    //let model = SUPPORTED_MODELS.iter().find(|&m| m == &model_name);
    let all_models = get_models();
    let model = all_models.iter().find(|&m| m.id == model_name);
    match model {
        Some(model) => {
            let json_model = serde_json::to_value(model).unwrap();
            HttpResponse::Ok().json(json_model)
        }
        None => HttpResponse::NotFound().body("Model not found"),
    }
}

// Delete a fine-tuned model. You must have the Owner role in your organization to delete a model.
// And we don't support this feature now.
#[delete("/v1/models/{model}")]
pub async fn delete_model(path: web::Path<String>) -> impl Responder {
    let model_name = path.into_inner();
    // log the model name
    println!("Deleting model: {}", model_name);
    HttpResponse::NotImplemented().body("Not implemented")
}