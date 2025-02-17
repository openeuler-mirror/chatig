use std::error::Error;

use crate::meta::services::traits::{ServiceConfig, ServicesTrait};
use crate::meta::services::impls::ServicesImpl;

pub struct ServiceManager {
    services: Box<dyn ServicesTrait>,
}

// Default implementation for ServiceManager
impl Default for ServiceManager {
    fn default() -> Self {
        ServiceManager {
            services: Box::new(ServicesImpl),
        }
    }
}

impl ServiceManager{
    pub fn _new(services: Box<dyn ServicesTrait>) -> Self {
        ServiceManager { services }
    }

    pub async fn load_services_table(&self) -> Result<(), Box<dyn Error>> {
        self.services.load_services_table().await
    }

    pub async fn create_service(&self, service: &ServiceConfig) -> Result<(), Box<dyn Error>> {
        self.services.create_service(service).await
    }

    pub async fn delete_service(&self, service_id: &str) -> Result<u64, Box<dyn Error>> {
        self.services.delete_service(service_id).await
    }

    pub async fn update_service(&self, service: &ServiceConfig) -> Result<u64, Box<dyn Error>> {
        self.services.update_service(service).await
    }

    pub async fn get_service(&self, service_id: &str) -> Result<Option<ServiceConfig>, Box<dyn Error>> {
        self.services.get_service(service_id).await
    }

    pub async fn get_service_by_model(&self, model_name: &str) -> Result<Option<ServiceConfig>, Box<dyn Error>> {
        self.services.get_service_by_model(model_name).await
    }

    pub async fn get_all_services(&self) -> Result<Vec<ServiceConfig>, Box<dyn Error>> {
        self.services.get_all_services().await
    }
}