use actix_web::{App, HttpServer};
use actix_cors::Cors;
use std::time::Duration;
use std::{fs::File, io::BufReader};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use log4rs::config::{init_raw_config, RawConfig};
use std::sync::Arc;
use actix_web::rt::time;
use std::sync::Mutex;
use crate::middleware::auth4manage::Auth4ManageMiddleware;
use crate::middleware::auth4model::Auth4ModelMiddleware;
use crate::middleware::qos::Qos;

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
use crate::middleware::rate_limit::RateLimitMiddleware;
use crate::apis::api_doc::ApiDoc;
use crate::utils::log::get_log_config;
use crate::middleware::qos::MultiServerClient;
use crate::middleware::qos::check_and_remove_unavailable_clients;
use lazy_static::lazy_static;

lazy_static! {
    static ref GLOBAL_MULTI_SERVER_CLIENT: Arc<Mutex<MultiServerClient>> = Arc::new(Mutex::new(MultiServerClient::new()));
}

#[cfg(test)]
mod test;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = &*GLOBAL_CONFIG;

    if config.coil_enabled {
        let multi_server_client_clone = GLOBAL_MULTI_SERVER_CLIENT.clone();
        let mut interval = time::interval(Duration::from_secs(3600));
        tokio::spawn(async move {
            loop {
                interval.tick().await;
                check_and_remove_unavailable_clients(multi_server_client_clone.clone()).await;
            }
        });
    }

    // Get log config and init log
    let log_config_content = get_log_config()?;
    let log_config: RawConfig = serde_yaml::from_str(&log_config_content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to parse log config: {}", e)))?;
    init_raw_config(log_config).unwrap();

    let db_pool = setup_database().await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Database setup failed: {}", e)))?;
    meta::connection::setup_database().await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Database setup failed: {}", e))).unwrap();

    generate_and_save_invitation_codes(&db_pool).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Database setup failed: {}", e)))?;

    // Set the port number
    let port = config.port;
    println!("Starting server on port {}", port);

    //Https set
    // println!("{:?}",config);
    let mut server_cert_file = BufReader::new(File::open(config.server_cert_file.clone()).unwrap());
    let mut chain_cert_file = BufReader::new(File::open(config.chain_cert_file.clone()).unwrap()); // 中间证书链
    let mut key_file = BufReader::new(File::open(config.key_file.clone()).unwrap());

    let server_certs = rustls_pemfile::certs(&mut server_cert_file)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let chain_certs = rustls_pemfile::certs(&mut chain_cert_file)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let mut tls_certs = server_certs;
    tls_certs.extend(chain_certs);
    let tls_key = rustls_pemfile::private_key(&mut key_file).unwrap().unwrap();
    // set up TLS config options
    let tls_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(tls_certs, tls_key)
        .unwrap();

    let rate_limiter = RateLimitMiddleware::new(config.rate_limit_tps, config.rate_limit_bucket_capacity, Duration::from_millis(config.rate_limit_refill_interval));
    let auth_manage = Arc::new(Auth4ManageMiddleware::new());
    let auth_model = Arc::new(Auth4ModelMiddleware::new());
    let qos = Arc::new(Qos::new());

    // Start the HTTP server
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin() // cors
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec!["Content-Type", "Authorization", "User-Agent"])
            .max_age(3600);

        App::new()
            .wrap(cors)
            //.wrap(ApiKeyCheck::new(Rc::new(db_pool.clone())))
            .wrap(rate_limiter.clone())
            .configure(|cfg| apis::models_api::chat::configure(cfg, auth_model.clone(), qos.clone()))
            // .configure(|cfg| apis::models_api::embeddings::configure(cfg, auth_model.clone()))
            //.configure(apis::models_api::image::configure)
            //.configure(apis::funcs_api::file_chat::configure)
            //.configure(apis::funcs_api::rag::configure)
            .configure(|cfg| apis::control_api::models::configure(cfg, auth_manage.clone()))
            .configure(|cfg| apis::control_api::files::configure(cfg, auth_manage.clone()))
            //.configure(apis::control_api::projects::configure)
            //.configure(apis::control_api::invitation_code::configure)
            //.configure(apis::control_api::users::configure)
            .configure(|cfg| apis::control_api::services::configure(cfg, auth_manage.clone(), auth_model.clone()))
            .configure(|cfg| apis::control_api::model_limits::configure(cfg, auth_manage.clone()))
            .service(SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", ApiDoc::openapi()))
    }) 
    .bind_rustls_0_23(("0.0.0.0", port), tls_config)?
    // .bind(("0.0.0.0", port))? // http
    .run()
    .await
}