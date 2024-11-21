use actix_web::{HttpRequest, web, Error};
use crate::configs::settings::Config;
use actix_web::error::ErrorInternalServerError;

use crate::servers::invitation_code::check_invitation_code_exists;
use crate::servers::api_schemas::AppState;

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

pub async fn check_api_key_db(headers: HttpRequest, data: web::Data<AppState>) -> Result<bool, Error> {
    let provided_api_key = headers
       .headers()
       .get("Authorization")
       .and_then(|auth_header| auth_header.to_str().ok())
       .map(|auth_str| auth_str.replace("Bearer ", ""))
       .unwrap_or_default();
    let exists = check_invitation_code_exists(&data.db_pool, &provided_api_key).await;
    match exists {
        Ok(result) => Ok(result),
        Err(err) => Err(ErrorInternalServerError(format!("Failed to check invitation code existence: {}", err))),
    }
}