use serde_json::json;
use std::error::Error;
use async_trait::async_trait;

use crate::meta::services_detail::traits::{ServicesDetail, ServicesDetailTrait};
use crate::meta::connection::DBCrud;

pub struct ServicesDetailImpl;

#[async_trait]
impl ServicesDetailTrait for ServicesDetailImpl {
    /// Create a new service configuration
    async fn create_service_detail(&self, service_detail: &ServicesDetail) -> Result<(), Box<dyn Error>> {
        // Insert into `services_detail` table
        let service_data = json!({
            "service_id": service_detail.service_id,
            "metrics_url": service_detail.metrics_url,
            "health_check_url": service_detail.health_check_url,
        });

        if let Err(err) = DBCrud::create("services_detail", &service_data).await {
            eprintln!("Failed to insert service detail: {}", err);
            return Err(err.into());
        }

        Ok(())
    }

    /// Delete a service configuration
    async fn delete_service_detail(&self, service_id: &str) -> Result<u64, Box<dyn Error>> {
        // Delete service configuration from `services_detail` table
        let conditions = &[("service_id", json!(service_id))];
        let delete_num = DBCrud::delete("services_detail", Some(conditions)).await?;

        Ok(delete_num)
    }

    /// Update a service configuration
    async fn update_service_detail(&self, service_detail: &ServicesDetail) -> Result<u64, Box<dyn Error>> {
        // Update service configuration in `services_detail` table
        let updates = &[
            ("metrics_url", json!(service_detail.metrics_url)),
            ("health_check_url", json!(service_detail.health_check_url)),
        ];
        let conditions = &[("service_id", json!(service_detail.service_id))];

        let rows_updated = DBCrud::update("services_detail", updates, Some(conditions)).await?;

        Ok(rows_updated)
    }

    /// Get a service configuration
    async fn get_service_detail(&self, service_id: &str) -> Result<Option<ServicesDetail>, Box<dyn Error>> {
        // Get service configuration from `services_detail` table
        let services_detail = DBCrud::get("services_detail", "service_id", &json!(service_id)).await?;

        Ok(services_detail)
    }

    /// Get all service configurations
    async fn get_all_service_details(&self) -> Result<Vec<ServicesDetail>, Box<dyn Error>> {
        // Get all service configurations from `services_detail` table
        let services_details = DBCrud::get_all("services_detail").await?;

        Ok(services_details)
    }
}
