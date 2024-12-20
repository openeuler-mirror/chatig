use actix_web::{App, HttpServer, web};
use crate::configs::settings::load_config;
use crate::servers::api_schemas::AppState;
use crate::database::init::setup_database;
use crate::servers::invitation_code::generate_and_save_invitation_codes;
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
        .app_data(web::Data::new(app_state.clone()))
        .configure(servers::server::configure)
        .configure(servers::models::configure)
        .configure(servers::files::configure)
        .configure(servers::projects::configure)
        .configure(servers::invitation_code::configure)
        .configure(servers::users::configure)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}