use actix_web::{App, HttpServer};
use actix_cors::Cors;
use std::rc::Rc;

mod apis;
mod cores;
mod configs;
mod meta;
mod middleware;
mod utils;
mod schema;

use crate::configs::settings::GLOBAL_CONFIG;
use crate::meta::init::setup_database;
use crate::apis::control_api::invitation_code::generate_and_save_invitation_codes;
use crate::middleware::api_key::ApiKeyCheck;

#[cfg(test)]
mod test;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Set up the AppState struct
    let config = &*GLOBAL_CONFIG;

    let db_pool = setup_database().await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Database setup failed: {}", e)))?;
    meta::connection::setup_database().await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Database setup failed: {}", e))).unwrap();

    generate_and_save_invitation_codes(&db_pool).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Database setup failed: {}", e)))?;

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
            .wrap(ApiKeyCheck::new(Rc::new(db_pool.clone())))
            .configure(apis::models_api::chat::configure)
            .configure(apis::models_api::embeddings::configure)
            .configure(apis::control_api::models::configure)
            .configure(apis::control_api::files::configure)
            .configure(apis::control_api::projects::configure)
            .configure(apis::control_api::invitation_code::configure)
            .configure(apis::control_api::users::configure)
    }) 
    .bind(("0.0.0.0", port))?
    .run()
    .await
}