use std::error::Error;

use crate::meta::services_detail::traits::{ServicesDetail, ServicesDetailTrait};
use crate::meta::services_detail::impls::ServicesDetailImpl;

pub struct ServicesDetailManager {
    services_detail: Box<dyn ServicesDetailTrait>,
}

// Default implementation for ServicesDetailManager
impl Default for ServicesDetailManager {
    fn default() -> Self {
        ServicesDetailManager {
            services_detail: Box::new(ServicesDetailImpl),
        }
    }
}
impl ServicesDetailManager {
    pub fn _new(services_detail: Box<dyn ServicesDetailTrait>) -> Self {
        ServicesDetailManager { services_detail }
    }

    pub async fn create_service_detail(&self, service_detail: &ServicesDetail) -> Result<(), Box<dyn Error>> {
        self.services_detail.create_service_detail(service_detail).await
    }

    pub async fn delete_service_detail(&self, id: &str) -> Result<u64, Box<dyn Error>> {
        self.services_detail.delete_service_detail(id).await
    }

    pub async fn update_service_detail(&self, service_detail: &ServicesDetail) -> Result<u64, Box<dyn Error>> {
        self.services_detail.update_service_detail(service_detail).await
    }

    pub async fn get_service_detail(&self, service_id: &str) -> Result<Option<ServicesDetail>, Box<dyn Error>> {
        self.services_detail.get_service_detail(service_id).await
    }

    pub async fn get_all_service_details(&self) -> Result<Vec<ServicesDetail>, Box<dyn Error>> {
        self.services_detail.get_all_service_details().await
    }
}