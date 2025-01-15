use actix_web::HttpRequest;
use chrono::Local;
use serde::{Deserialize, Serialize};

// Log info for tokens
#[derive(Deserialize, Serialize, Debug)]
pub struct Tokens {
    pub timestamp: i64,
    pub fields: FieldsInfo,
    pub tags: TagsInfo
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FieldsInfo {
    pub completion_tokens: u32,
    pub prompt_tokens: u32,
    pub total_tokens: u32
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TagsInfo {
    pub user_name: String,
    pub model_name: String,
}

// Function for access log and error log
pub async fn log_request(
    req: HttpRequest,
    status_code: u16,
    error_message: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
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

    let log_message = if let Some(msg) = error_message {
        // Error log format
        format!(
            "{client_ip} - - [{time}] \"{request_method} {request_uri} {http_version}\" {status_code} \"{referer}\" \"{user_agent}\" \"{error_message}\"",
            client_ip = client_ip,
            time = Local::now().format("%d/%b/%Y:%H:%M:%S %z"),
            request_method = request_method,
            request_uri = request_uri,
            http_version = http_version,
            status_code = status_code,
            referer = referer,
            user_agent = user_agent,
            error_message = msg
        )
    } else {
        // Access log format
        format!(
            "{client_ip} - - [{time}] \"{request_method} {request_uri} {http_version}\" {status_code} \"{referer}\" \"{user_agent}\"",
            client_ip = client_ip,
            time = Local::now().format("%d/%b/%Y:%H:%M:%S %z"),
            request_method = request_method,
            request_uri = request_uri,
            http_version = http_version,
            status_code = status_code,
            referer = referer,
            user_agent = user_agent,
        )
    };

    Ok(log_message)
}