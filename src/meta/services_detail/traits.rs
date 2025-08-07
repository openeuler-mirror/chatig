use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::error::Error;

use async_trait::async_trait;

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct ServicesDetail {
    pub service_id: String, // 关联到 services 表的服务实例 ID，作为主键
    pub metrics_url: Option<String>, // 用于服务指标的监控
    pub health_check_url: Option<String>, // 用于检查模型是否健康运行的 URL
}

#[async_trait]
pub trait ServicesDetailTrait: Send + Sync {
    async fn create_service_detail(&self, service_detail: &ServicesDetail) -> Result<(), Box<dyn Error>>;
    async fn delete_service_detail(&self, service_id: &str) -> Result<u64, Box<dyn Error>>;
    async fn update_service_detail(&self, service_detail: &ServicesDetail) -> Result<u64, Box<dyn Error>>;
    async fn get_service_detail(&self, service_id: &str) -> Result<Option<ServicesDetail>, Box<dyn Error>>;
    async fn get_all_service_details(&self) -> Result<Vec<ServicesDetail>, Box<dyn Error>>;
}