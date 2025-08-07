use serde::{Serialize, Deserialize};
use async_trait::async_trait;

// project_object table structure
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    pub id: String,                    // The identifier, which can be referenced in API endpoints
    pub object: String,             // The object type, which is always organization.project
    pub name: String,               // The name of the project. This appears in reporting.
    pub created_at: i64,            // The Unix timestamp (in seconds) of when the project was created.
    pub archived_at: Option<i64>,   // The Unix timestamp (in seconds) of when the project was archived or null.
    pub status: String,             // active or archive
}

#[async_trait]
pub trait ProjectTrait {
    async fn list_projects(&self, limit: i64, after: Option<String>, include_archived: bool)
        -> Result<Vec<Project>, Box<dyn std::error::Error>>;
    async fn create_project(&self, project: Project) -> Result<(), Box<dyn std::error::Error>>;
    async fn retrieve_project(&self, project_id: String) -> Result<Project, Box<dyn std::error::Error>>;
    async fn modify_project(&self, project_id: String, project_name: String) 
        -> Result<Project, Box<dyn std::error::Error>>;
    async fn archive_project(&self, project_id: String) -> Result<Project, Box<dyn std::error::Error>>;
}