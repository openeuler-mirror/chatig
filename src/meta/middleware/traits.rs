use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::error::Error;

use async_trait::async_trait;

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct UserKeys {
    pub userkey: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct UserKeysModels {
    pub userkey: String,
    pub model: String,
}

#[async_trait]
pub trait UserKeysTrait: Send + Sync {
    async fn check_userkey(&self, userkey: &str) -> Result<bool, Box<dyn Error>>;
    async fn check_userkey_model(&self, userkey: &str, model: &str) -> Result<bool, Box<dyn Error>>;
}