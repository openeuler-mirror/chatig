use crate::meta::connection::DBCrud;
use crate::meta::middleware::traits::{UserKeysTrait, UserKeys, UserKeysModels};
use async_trait::async_trait;
use serde_json::json;
use std::error::Error;

pub struct UserKeysImpl;

#[async_trait]
impl UserKeysTrait for UserKeysImpl {
    async fn check_userkey(&self, userkey: &str) -> Result<bool, Box<dyn Error>> {
        let record = DBCrud::get::<UserKeys>(
            "UserKeys",
            "userkey",
            &json!(userkey),
        )
        .await?;
        Ok(record.is_some())
    }

    async fn check_userkey_model(&self, userkey: &str, model: &str) -> Result<bool, Box<dyn Error>> {
        let all_records = DBCrud::get_all::<UserKeysModels>("UserKeysModels").await?;
        let found = all_records
            .iter()
            .any(|record| record.userkey == userkey && record.model == model);
        Ok(found)
    }
}