use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::error::Error;

use async_trait::async_trait;

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct Limits {
    pub model_name: String,
    pub max_requests: String,
    pub max_tokens: String,
}

#[async_trait]
pub trait LimitsTrait: Send + Sync {
    async fn add_limits_object(&self, limits: Limits) -> Result<(), Box<dyn Error>>;
    async fn delete_limits_object(&self, model_name: &str) -> Result<(), Box<dyn Error>>;
    async fn update_limits_object(&self, limits: Limits) -> Result<u64, Box<dyn Error>>;
    async fn get_limits_object(&self, model_name: &str) -> Result<Option<Limits>, Box<dyn Error>>;
    async fn get_all_limits_objects(&self) -> Result<Vec<Limits>, Box<dyn Error>>;
}