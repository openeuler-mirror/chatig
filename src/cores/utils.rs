use serde_json::Value;
use reqwest::{Client, Response};
use chrono_tz::Asia::Shanghai;
use std::time::Duration;
use chrono::{Utc, DateTime};
use chrono_tz::Tz;

use crate::GLOBAL_CONFIG;
use crate::meta::services::traits::Services;


pub async fn get_response(
    request_body: Value,
    service: &mut Services,
    req_model_name: String,
) -> Result<(Response, DateTime<Tz>), String> {
    let config = &*GLOBAL_CONFIG;
    let client = Client::builder()
        .timeout(Duration::from_secs(config.request_timeout))
        .connect_timeout(Duration::from_secs(config.connection_timeout))
        .build()
        .map_err(|err| format!("Failed to build client: {}", err))?;

    let mut retry_count = 0;
    loop {
        let start_time = Utc::now().with_timezone(&Shanghai);
        match client
            .post(service.url.clone())
            .header("Content-Type", "application/json")
            .header("Authorization", service.api_key.clone())
            .json(&request_body)
            .send()
            .await
        {
            Ok(resp) => match resp.status() {
                status if status.is_success() => {
                    return Ok((resp, start_time));
                }
                status if status.is_server_error() => {
                    retry_count += 1;
                    if retry_count <= config.request_retry {
                        println!(
                            "Request failed for model {}, service id: {}: {}. Retrying ({}/{}). ",
                            req_model_name,
                            status,
                            service.id,
                            retry_count,
                            config.request_retry
                        );
                        continue;
                    } else {
                        return Err(format!("Request failed: {} model request failed, status: {}", req_model_name, status));
                    }
                }
                status => {
                    return Err(format!("Request failed: {} model request failed, status: {}", req_model_name, status));
                }
            },
            Err(_) => {
                retry_count += 1;
                if retry_count <= config.request_retry {
                    println!(
                        "Request failed for model {}, service id: {}. Retrying ({}/{})",
                        req_model_name,
                        service.id,
                        retry_count,
                        config.request_retry
                    );
                    continue;
                } else {
                    return Err(format!("Request failed: {} model request failed.", req_model_name));
                }
            }
        };
    }
}
