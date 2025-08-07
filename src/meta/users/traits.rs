use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use sqlx::FromRow;

// user_object table structure
#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct UserObject {
    pub id: String,                 // The identifier, which can be referenced in API endpoints
    pub object: String,             // The object type, which is always organization.user
    pub name: String,               // The name of the user
    pub email: String,              // The email address of the user
    pub role: String,               // owner or reader
    pub added_at: i64,              // The Unix timestamp (in seconds) of when the user was added.
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct UserObjectDto {
    pub name: String,               // The name of the user
    pub email: String,              // The email address of the user
    pub role: String,               // owner or reader
}

#[async_trait]
pub trait UserObjectTrait {
    async fn insert_user_object(&self, user: UserObjectDto) -> Result<UserObject, Box<dyn std::error::Error>>;
    async fn list_user_objects(&self, limit: i64, after: Option<String>) -> Result<Vec<UserObject>, Box<dyn std::error::Error>>;
    async fn modify_user_object(&self, id: String, role: String) -> Result<UserObject, Box<dyn std::error::Error>>;
    async fn retrieve_user_object(&self, id: String) -> Result<Option<UserObject>, Box<dyn std::error::Error>>;
    async fn delete_user_object(&self, id: String) -> Result<u64, Box<dyn std::error::Error>>;
}
