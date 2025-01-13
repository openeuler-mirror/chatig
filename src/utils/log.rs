use std::io::Write;
use actix_web::HttpRequest;
use log::{Level, LevelFilter, info, error};
use env_logger::{Builder, Env};
use chrono::Local;

// Init log
pub fn convert_log_level(log_level: String) -> LevelFilter {
    match log_level.as_str() {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Off,
    }
}

pub fn init_logger(log_level: &str, module_filter: &str) {
    let mut builder = Builder::from_env(Env::default().default_filter_or(log_level));
    builder
        .filter_module(module_filter, convert_log_level(log_level.to_string()))
        .filter(None, log::LevelFilter::Off);
    builder.format(|buf, record| {
        let level = record.level();
        let args = record.args();
        match level {
            Level::Error => writeln!(buf, "[ERROR] {}", args),
            Level::Warn => writeln!(buf, "[WARN] {}", args),
            Level::Info => writeln!(buf, "[INFO] {}", args),
            Level::Debug => writeln!(buf, "[DEBUG] {}", args),
            Level::Trace => writeln!(buf, "[TRACE] {}", args),
        }
    });
    builder.init();
}

// Get info from the request
#[allow(dead_code)]
#[derive(Clone)]
pub struct RequestInfo {
    pub referer: String,
    pub user_agent: String,
    pub client_ip: String,
    pub request_method: String,
    pub request_uri: String,
    pub http_version: String
}

#[allow(dead_code)]
pub fn get_info_from_request(req: HttpRequest) -> RequestInfo {
    let referer = req.headers()
       .get("Referer")
       .and_then(|v| v.to_str().ok())
       .unwrap_or("-")
       .to_string();
    let user_agent = req.headers()
       .get("User-Agent")
       .and_then(|v| v.to_str().ok())
       .unwrap_or("unknown")
       .to_string();
    let client_ip = req.peer_addr().map(|addr| addr.ip().to_string()).unwrap_or_else(|| "unknown".to_string());
    let request_method = req.method().as_str().to_string();
    let request_uri = req.uri().to_string();
    let http_version = format!("{:?}", req.version());
    let req_info = RequestInfo {
        referer,
        user_agent,
        client_ip,
        request_method,
        request_uri,
        http_version,
    };
    req_info
}

// Log for response
#[allow(dead_code)]
pub async fn log_response(
    req: RequestInfo,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        "{client_ip} - - [{time}] \"{request_method} {request_uri} {http_version}\" {status_code} \"{referer}\" \"{user_agent}\"",
        client_ip = req.client_ip,
        time = Local::now().format("%d/%b/%Y:%H:%M:%S %z"),
        request_method = &req.request_method,
        request_uri = &req.request_uri,
        http_version = format!("{:?}", req.http_version),
        status_code = 200,
        // response_size = response_size,
        referer = req.referer,
        user_agent = req.user_agent,
    );
    Ok(())
}

// Log for error
#[allow(dead_code)]
pub async fn log_error(
    req: RequestInfo,
    error_message: &str,
    status_code: u16
) -> Result<(), Box<dyn std::error::Error>> {
    error!(
        "{client_ip} - - [{time}] \"{request_method} {request_uri} {http_version}\" {status_code} \"{referer}\" \"{user_agent}\" \"{error_message}\"",
        client_ip = req.client_ip,
        time = Local::now().format("%d/%b/%Y:%H:%M:%S %z"),
        request_method = &req.request_method,
        request_uri = &req.request_uri,
        http_version = format!("{:?}", req.http_version),
        status_code = status_code,
        referer = req.referer,
        user_agent = req.user_agent,
        error_message = error_message
    );
    Ok(())
}