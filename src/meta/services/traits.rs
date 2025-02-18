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
}

// 记录多模型集群里，集群支持的模型类型
#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct ModelsService {
    pub serviceid: String,
    pub modelid: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct ServiceConfig {
    pub id: String,
    pub servicetype: String,
    pub status: String,
    pub url: String,
    pub model_name: String,
    pub active_model: String,
    pub models: Vec<String>,
}

#[derive(Deserialize)]
pub struct InvalidateCacheRequest {
    pub key: String,
    pub cache_type: String, // 指定是清除哪一类缓存, 可以是 "manage" 或 "model"
}

#[async_trait]
pub trait ServicesTrait: Send + Sync {
    async fn load_services_table(&self) -> Result<(), Box<dyn Error>>;
    async fn create_service(&self, service: &ServiceConfig) -> Result<(), Box<dyn Error>>;
    async fn delete_service(&self, service_id: &str) -> Result<u64, Box<dyn Error>>;
    async fn update_service(&self, service: &ServiceConfig) -> Result<u64, Box<dyn Error>>;
    async fn get_service(&self, service_id: &str) -> Result<Option<ServiceConfig>, Box<dyn Error>>;
    async fn get_service_by_model(&self, active_model: &str) -> Result<Option<ServiceConfig>, Box<dyn Error>>;
    async fn get_all_services(&self) -> Result<Vec<ServiceConfig>, Box<dyn Error>>;
}