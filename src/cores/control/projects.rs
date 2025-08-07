use crate::meta::projects::traits::{ProjectTrait, Project};
use crate::meta::projects::impls::ProjectImpl;

pub struct Projectmanager {
    projects: Box<dyn ProjectTrait>,
}

impl Default for Projectmanager {
    fn default() -> Self {
        Projectmanager {
            projects: Box::new(ProjectImpl),
        }
    }
}

impl Projectmanager {
    pub fn _new(projects: Box<dyn ProjectTrait>) -> Self {
        Projectmanager { projects }
    }

    pub async fn list_projects(&self, limit: i64, after: Option<String>, include_archived: bool)-> Result<Vec<Project>, Box<dyn std::error::Error>>{
        self.projects.list_projects(limit, after, include_archived).await
    }
    pub async fn create_project(&self, project: Project) -> Result<(), Box<dyn std::error::Error>>{
        self.projects.create_project(project).await
    }
    pub async fn retrieve_project(&self, project_id: String) -> Result<Project, Box<dyn std::error::Error>>{
        self.projects.retrieve_project(project_id).await
    }
    pub async fn modify_project(&self, project_id: String, project_name: String) -> Result<Project, Box<dyn std::error::Error>>{
        self.projects.modify_project(project_id, project_name).await
    }
    pub async fn archive_project(&self, project_id: String) -> Result<Project, Box<dyn std::error::Error>>{
        self.projects.archive_project(project_id).await
    }
}