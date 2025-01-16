use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct Services {
    pub id: String,
    pub servicetype: String,
    pub status: String, // active or inactive
    pub url: String,
    pub max_token: i64,
}

// 记录多模型集群里，集群支持的模型类型
#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct ModelsService {
    pub serviceid: String,
    pub modelid: String,
}