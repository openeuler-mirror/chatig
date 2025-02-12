use serde_json::json;
use std::error::Error;
use async_trait::async_trait;

use crate::meta::qos::traits::{Limits, LimitsTrait};
use crate::meta::connection::DBCrud;

pub struct LimitsImpl;

#[async_trait]
impl LimitsTrait for LimitsImpl {
    async fn add_limits_object(&self, limits: Limits) -> Result<(), Box<dyn Error>>{
        let model_limits_object = json!({
            "model_name": limits.model_name,
            "max_requests": limits.max_requests,
            "max_tokens": limits.max_tokens,
        });
        DBCrud::create("model_limits", &model_limits_object).await?;

        Ok(())
    }

    async fn delete_limits_object(&self, model_name: &str) -> Result<(), Box<dyn Error>>{
        let limits_conditions = &[("model_name", json!(model_name))];
        DBCrud::delete("model_limits", Some(limits_conditions)).await?;

        Ok(())
    }

    async fn update_limits_object(&self, limits: Limits) -> Result<u64, Box<dyn Error>>{
        let updates = &[
            ("model_name", json!(limits.model_name)),
            ("max_requests", json!(limits.max_requests)),
            ("max_tokens", json!(limits.max_tokens)),
        ];

        let conditions = &[("model_name", json!(limits.model_name))];
        let rows_updated = DBCrud::update("model_limits", updates, Some(conditions)).await?;

        Ok(rows_updated)
    }

    async fn get_limits_object(&self, model_name: &str) -> Result<Option<Limits>, Box<dyn Error>>{
        let limits: Option<Limits> = DBCrud::get("model_limits", "model_name", &json!(model_name)).await?;

        Ok(limits)
    }

    async fn get_all_limits_objects(&self) -> Result<Vec<Limits>, Box<dyn Error>>{
        let model_limits: Vec<Limits> = DBCrud::get_all("model_limits").await?;

        Ok(model_limits)
    }
}