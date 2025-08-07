use actix_web::{App, HttpServer};
use actix_cors::Cors;
use std::time::Duration;
use std::{fs::File, io::BufReader};
use log4rs::config::{init_raw_config, RawConfig};

mod apis;
mod cores;
mod configs;
mod meta;
mod utils;
mod schema;

use crate::configs::settings::GLOBAL_CONFIG;
use crate::meta::init::setup_database;
use crate::utils::log::get_log_config;
use crate::cores::control::health::monitor_model_health;

#[cfg(test)]
mod test;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = &*GLOBAL_CONFIG;

    // Get log config and init log
    let log_config_content = get_log_config()?;
    let log_config: RawConfig = serde_yaml::from_str(&log_config_content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to parse log config: {}", e)))?;
    init_raw_config(log_config).unwrap();

    setup_database().await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Database setup failed: {}", e)))?;
    meta::connection::setup_database().await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Database setup failed: {}", e))).unwrap();

    // Start models health monitoring service in a new task
    let check_interval = Duration::from_secs(config.model_health_check_interval);
    tokio::spawn(async move {
        if let Err(e) = monitor_model_health(check_interval).await {
            eprintln!("Model health monitoring error: {}", e);
        }
    });

    // Set the port number
    let port = config.port;
    println!("Starting server on port {}", port);

    // Start the HTTP server
    let main_server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin() // cors
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec!["Content-Type", "Authorization", "User-Agent"])
            .max_age(3600);

        App::new()
            .wrap(cors)
            //.wrap(ApiKeyCheck::new(Rc::new(db_pool.clone())))
            .configure(|cfg| apis::models_api::chat::configure(cfg))
            .configure(|cfg| apis::models_api::generate::configure(cfg))
            .configure(|cfg| apis::models_api::embeddings::configure(cfg))
            .configure(|cfg| apis::models_api::rerank::configure(cfg))
            //.configure(apis::models_api::image::configure)
            //.configure(apis::funcs_api::file_chat::configure)
            //.configure(apis::funcs_api::rag::configure)
            .configure(|cfg| apis::control_api::models::configure(cfg))
            .configure(|cfg| apis::control_api::files::configure(cfg))
            //.configure(apis::control_api::projects::configure)
            //.configure(apis::control_api::users::configure)
            .configure(|cfg| apis::control_api::services::configure(cfg))
            .configure(|cfg| apis::control_api::services_detail::configure(cfg))
            //.service(SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", ApiDoc::openapi()))
    });

    // HTTPS or HTTP setup
    let _main_res = if config.https_enabled {
        // HTTPS setup
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

        // HTTPS 主服务 + HTTP 指标服务
        let mut builder = main_server.bind_rustls_0_23(("0.0.0.0", port), tls_config.clone())?;

        if config.ipv6_enabled {
            builder = builder.bind_rustls_0_23(("::", config.ipv6_port), tls_config.clone())?;
            println!("ipv6 enabled");
        }

        let _main_future = builder.run();
    } else {
        // HTTP 双服务
        let mut server_builder = main_server.bind(("0.0.0.0", port))?;

        if config.ipv6_enabled {
            server_builder = server_builder.bind(("::", config.ipv6_port))?;
            println!("ipv6 enabled");
        }

        let _main_future = server_builder.run();
    };
    Ok(())
}