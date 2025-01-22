use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use std::error::Error;
use utoipa::ToSchema;
use async_trait::async_trait;


// file_object table structure
/*
{
  "id": "file-abc123",
  "object": "file",
  "bytes": 120000,
  "created_at": 1677610602,
  "filename": "salesOverview.pdf",
  "purpose": "assistants",
}
*/
#[derive(Serialize, Deserialize, Debug, ToSchema, Clone, FromRow)]
pub struct File {
    pub id: String,           
    pub bytes: i64,
    pub created_at: i64,
    pub filename: String,
    pub object: String,
    pub purpose: String,
}

#[async_trait]
pub trait FilesTrait: Send + Sync {
    async fn add_file_object(&self, file: File) -> Result<(), Box<dyn Error>>;
    async fn delete_file_object(&self, file_id: &str) -> Result<(), Box<dyn Error>>;
    async fn update_file_object(&self, file: File) -> Result<u64, Box<dyn Error>>;
    async fn get_file_object(&self, file_id: &str) -> Result<Option<File>, Box<dyn Error>>;
    async fn get_all_file_objects(&self) -> Result<Vec<File>, Box<dyn Error>>;
}