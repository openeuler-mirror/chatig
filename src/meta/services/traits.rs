use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::error::Error;

use async_trait::async_trait;

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct Services {
    pub id: String,
    pub servicetype: String,
    pub status: String, // active or inactive
    pub url: String,
    pub model_name: String,
    pub active_model: String,
    pub api_key: String,
    pub context_length: f32,
    pub tags: Option<String>
}

#[async_trait]
pub trait ServicesTrait: Send + Sync {
    async fn load_services_table(&self) -> Result<(), Box<dyn Error>>;
    async fn create_service(&self, service: &Services) -> Result<(), Box<dyn Error>>;
    async fn delete_service(&self, service_id: &str) -> Result<u64, Box<dyn Error>>;
    async fn update_service(&self, service: &Services) -> Result<u64, Box<dyn Error>>;
    async fn get_service(&self, service_id: &str) -> Result<Option<Services>, Box<dyn Error>>;
    async fn get_service_by_model(&self, active_model: &str, input_tokens: Option<f32>, aicpid_json: String) -> Result<Option<Services>, Box<dyn Error>>;
    async fn get_all_services(&self) -> Result<Vec<Services>, Box<dyn Error>>;
    //async fn get_id_by_model(&self, active_model: &str) -> Result<String, Box<dyn Error>>;
}