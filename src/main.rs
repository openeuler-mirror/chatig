use actix_web::{App, HttpServer};
use actix_cors::Cors;
use std::time::Duration;
use std::{rc::Rc, fs::File, io::BufReader};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use log4rs::init_file;

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
use crate::middleware::rate_limit::RateLimitMiddleware;
use crate::apis::api_doc::ApiDoc;

#[cfg(test)]
mod test;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = &*GLOBAL_CONFIG;

    let config_path = format!("{}/src/configs/log4rs.yaml", env!("CARGO_MANIFEST_DIR"));
    init_file(&config_path, Default::default()).unwrap();

    let db_pool = setup_database().await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Database setup failed: {}", e)))?;
    meta::connection::setup_database().await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Database setup failed: {}", e))).unwrap();

    generate_and_save_invitation_codes(&db_pool).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Database setup failed: {}", e)))?;

    // Set the port number
    let port = config.port;
    println!("Starting server on port {}", port);

    //Https set
    let mut certs_file = BufReader::new(File::open("docs/https/server.crt").unwrap());
    let mut key_file = BufReader::new(File::open("docs/https/server.key").unwrap());

    let tls_certs = rustls_pemfile::certs(&mut certs_file)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let tls_key = rustls_pemfile::private_key(&mut key_file).unwrap().unwrap();

    // set up TLS config options
    let tls_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(tls_certs, tls_key)
        .unwrap();

    let rate_limiter = RateLimitMiddleware::new(config.rate_limit_tps, config.rate_limit_bucket_capacity, Duration::from_millis(config.rate_limit_refill_interval));
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
            .wrap(rate_limiter.clone())
            .configure(apis::models_api::chat::configure)
            .configure(apis::models_api::embeddings::configure)
            .configure(apis::funcs_api::file_chat::configure)
            .configure(apis::funcs_api::rag::configure)
            .configure(apis::control_api::models::configure)
            .configure(apis::control_api::files::configure)
            .configure(apis::control_api::projects::configure)
            .configure(apis::control_api::invitation_code::configure)
            .configure(apis::control_api::users::configure)
            .configure(apis::control_api::services::configure)
            .service(SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", ApiDoc::openapi()))
    }) 
    .bind_rustls_0_23(("0.0.0.0", port), tls_config)?
    .run()
    .await
}