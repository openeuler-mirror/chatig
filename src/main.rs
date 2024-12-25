use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use configs::settings::GLOBAL_CONFIG;
use database::diesel::{establish_connection, run_migrations};
use crate::servers::api_schemas::AppState;
use crate::database::init::setup_database;
use crate::servers::invitation_code::generate_and_save_invitation_codes;
use crate::middleware::api_key::ApiKeyCheck;
use std::rc::Rc;

mod servers;
mod models;
mod configs;
mod database;
mod schema;
mod middleware;

#[cfg(test)]
mod test;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Set up the AppState struct
    let config = &*GLOBAL_CONFIG;

    let pool = establish_connection();
    let mut conn = pool.get().expect("Failed to get connection from pool");
    run_migrations(&mut conn);

    let db_pool = setup_database(config.clone().database).await
    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Database setup failed: {}", e)))?;
    let app_state = web::Data::new(AppState { config: config.clone(), db_pool: db_pool.clone() });

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
            // .wrap(ApiKeyCheck::new(db_pool.clone()))
            .app_data(app_state.clone())
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