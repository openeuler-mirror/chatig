use std::time::Duration;
use tokio::time;
use reqwest::Client;
use log;

use crate::cores::control::services::ServiceManager;
use crate::cores::control::services_detail::ServicesDetailManager;

/// Monitors the health status of models continuously.
///
/// This function runs in an infinite loop, periodically checking the health status of the models.
pub async fn monitor_model_health(check_interval: Duration) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Started model health monitoring service");
    let service_detail_manager = ServicesDetailManager::default();
    let service_manager = ServiceManager::default();

    loop {
        let llm_service_details = match service_detail_manager.get_all_service_details().await {
            Ok(details) => details,
            Err(e) => {
                println!("Failed to fetch service_details table: {}, stop model health monitoring service!", e);
                return Ok(());
            }
        };

        for llm_service_detail in llm_service_details {
            // Skip if the model service does not provide a health check interface
            let Some(health_check_url) = &llm_service_detail.health_check_url else {
                continue;
            };

            let llm_service = match service_manager.get_service(&llm_service_detail.service_id).await {
                Ok(Some(service)) => service,
                Ok(None) => {
                    println!("Service {} not found", llm_service_detail.service_id);
                    continue;
                },
                Err(e) => {
                    println!("Failed to fetch service {}: {}", llm_service_detail.service_id, e);
                    continue;
                }
            };

            match check_model_health(&llm_service.id.clone(), &health_check_url.clone(), &llm_service.servicetype.clone()).await {
                Ok(is_healthy) => {
                    let mut llm_service = llm_service.clone(); // Clone the service configuration
                    let current_status = llm_service.status.clone();
                    let new_status = if is_healthy { "active" } else { "inactive" };

                    if current_status != new_status {
                        llm_service.status = new_status.to_string();
                        println!("Model {} status changed to {}", llm_service.active_model, new_status);
    
                        if let Err(e) = service_manager.update_service(&llm_service).await {
                            println!("Failed to update service status: {}", e);
                        }
                    }
    
                    if !is_healthy {
                        // TODO: Implement alert logic
                    }
                },
                Err(e) => {
                    println!("Failed to perform service: {} health check: {}", llm_service.id, e);
                }
            }
        }

        // Wait for the specified interval before checking again
        time::sleep(check_interval).await;
    }
}

/// Implements the actual model health check.
async fn check_model_health(
    service_id: &str,
    health_check_url: &str,
    servicetype: &str,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let response = client.get(health_check_url).send().await?;
    if !response.status().is_success() {
        log::warn!(
            "Health check for service {} failed with status: {}",
            service_id,
            response.status()
        );
    }

    match servicetype {
        "vllm" => Ok(response.status().is_success()),
        "mindie" => {
            let body = response.text().await?;
            let json: serde_json::Value = serde_json::from_str(&body)?;
            Ok(json["status"] == "healthy")
        },
        "ollama" => {
            // TODO: Implement health check logic for OLLAMA
            Ok(true)
        },
        _ => {
            log::warn!("Unknown service type: {}", servicetype);
            Ok(false)
        }
    }
}
