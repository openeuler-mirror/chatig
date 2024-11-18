use actix_web::{App, HttpServer, web};
use crate::configs::settings::load_config;
use crate::servers::api_schemas::AppState;
use crate::database::init::setup_database;
use actix_cors::Cors;

mod servers;
mod models;
mod configs;
mod utils;
mod database;

#[cfg(test)]
mod test;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Set up the AppState struct
    let config = load_config()
    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to load config: {}", e)))?;
    let db_pool = setup_database(config.clone().database).await
    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Database setup failed: {}", e)))?;
    let app_state = AppState { config: config.clone(), db_pool: db_pool.clone() };

    // Set the port number
    let port = config.port;
    println!("Starting server on port {}", port);

    // Start the HTTP server
    HttpServer::new(move || {
        let cors = Cors::default()
        .allow_any_origin() // cors
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
        .allowed_headers(vec!["Content-Type", "Authorization", "User-Agent"])
        .max_age(3600);
        App::new()
        .wrap(cors)
        .app_data(web::Data::new(app_state.clone()))
        .service(servers::server::health)
        .service(servers::server::rag_chat_completions)   // Register new POST service
        .service(servers::server::chat_completions)       
        .service(servers::models::models)
        .service(servers::models::model_info)
        .service(servers::models::delete_model)
        .service(servers::files::upload_file)
        .service(servers::files::delete_file)         
        .service(servers::files::list_file)
        .service(servers::files::get_file)
        .service(servers::files::get_file_content)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}