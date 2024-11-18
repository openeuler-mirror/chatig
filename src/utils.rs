use actix_web::HttpRequest;

use crate::configs::settings::Config;

// Check if the API Key in the request headers matches the config
pub fn check_api_key(headers: HttpRequest, config: &Config) -> bool {
    
    let provided_api_key = headers
        .headers()
        .get("Authorization")
        .and_then(|auth_header| auth_header.to_str().ok())
        .map(|auth_str| auth_str.replace("Bearer ", ""))
        .unwrap_or_default();
    provided_api_key == config.apikey.value
}