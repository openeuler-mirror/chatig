use std::error::Error;

use crate::meta::qos::traits::{Limits, LimitsTrait};
use crate::meta::qos::impls::LimitsImpl;


pub struct LimitsManager {
    limits: Box<dyn LimitsTrait>,
}

// Default implementation for FileManager
impl Default for LimitsManager {
    fn default() -> Self {
        LimitsManager {
            limits: Box::new(LimitsImpl),
        }
    }
}

impl LimitsManager{
    pub fn _new(limits: Box<dyn LimitsTrait>) -> Self {
        LimitsManager { limits }
    }

    pub async fn add_limits_object(&self, limits: Limits) -> Result<(), Box<dyn Error>> {
        self.limits.add_limits_object(limits).await
    }

    pub async fn delete_limits_object(&self, model_name: &str) -> Result<(), Box<dyn Error>> {
        self.limits.delete_limits_object(model_name).await
    }

    pub async fn update_limits_object(&self, limits: Limits) -> Result<u64, Box<dyn Error>> {
        self.limits.update_limits_object(limits).await
    }

    pub async fn get_limits_object(&self, model_name: &str) -> Result<Option<Limits>, Box<dyn Error>> {
        self.limits.get_limits_object(model_name).await
    }

    pub async fn get_all_limits_objects(&self) -> Result<Vec<Limits>, Box<dyn Error>> {
        self.limits.get_all_limits_objects().await
    }
}