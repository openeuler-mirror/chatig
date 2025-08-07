use async_trait::async_trait;

use crate::meta::projects::traits::{ProjectTrait, Project};
use crate::meta::init::get_pool;

pub struct ProjectImpl;

#[async_trait]
impl ProjectTrait for ProjectImpl {
    // List all project objects
    async fn list_projects(&self, limit: i64, after: Option<String>, include_archived: bool) -> Result<Vec<Project>, Box<dyn std::error::Error>> {
        let pool = get_pool().await?;
        let client = pool.get().await?;

        // dynamic build query and parameters
        let mut query = String::from(
            "SELECT * FROM project_object WHERE 1=1"
        );
        let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();

        // if after is not None, add condition
        if let Some(after_value) = after {
            query.push_str(" AND id > $1");
            let after_value_clone = after_value.clone();
            params.push(Box::leak(Box::new(after_value_clone)));
        }

        // if !include_archived, filter out archived projects
        if !include_archived {
            query.push_str(" AND status = 'active'");
        }

        // add order and limit
        if params.is_empty() {
            query.push_str(" ORDER BY id ASC LIMIT $1");
        } else {
            query.push_str(" ORDER BY id ASC LIMIT $2");
        }
        params.push(&limit);

        let rows = client.query(&query, &params.iter().map(|b| &**b).collect::<Vec<_>>()).await?;

        let mut projects = Vec::new();
        for row in rows {
            let project = Project {
                id: row.get(0),
                object: row.get(1),
                name: row.get(2),
                created_at: row.get(3),
                archived_at: row.get(4),
                status: row.get(5),
            };
            projects.push(project);
        }

        Ok(projects)
    }

    // Create project object
    async fn create_project(&self, project: Project) -> Result<(), Box<dyn std::error::Error>> {
        let pool = get_pool().await?;
        let client = pool.get().await?;

        let query = "
            INSERT INTO project_object (id, object, name, created_at, archived_at, status)
            VALUES ($1, $2, $3, $4, $5, $6)";
        
        client.execute(query, &[
            &project.id,
            &project.object,
            &project.name,
            &project.created_at,
            &project.archived_at,
            &project.status,
        ]).await?;

        Ok(())
    }

    // Retrieve project object
    async fn retrieve_project(
        &self,
        project_id: String,
    ) -> Result<Project, Box<dyn std::error::Error>> {
        let pool = get_pool().await?;
        let client = pool.get().await?;

        let query = "SELECT id, object, name, created_at, archived_at, status FROM project_object WHERE id = $1";
        let row = client.query_one(query, &[&project_id]).await?;

        let project = Project {
            id: row.get(0),
            object: row.get(1),
            name: row.get(2),
            created_at: row.get(3),
            archived_at: row.get(4),
            status: row.get(5),
        };

        Ok(project)
    }

    // Modify project object
    async fn modify_project(
        &self,
        project_id: String,
        project_name: String,
    ) -> Result<Project, Box<dyn std::error::Error>> {
        let pool = get_pool().await?;
        let client = pool.get().await?;

        let query = "
            UPDATE project SET name = $1 WHERE id = $2";
        
        client.execute(query, &[
            &project_name,
            &project_id,
        ]).await?;

        let project = self.retrieve_project(project_id.clone()).await?;

        Ok(project)
    }

    // Archive project object
    async fn archive_project(&self, project_id: String) -> Result<Project, Box<dyn std::error::Error>> {
        let pool = get_pool().await?;
        let client = pool.get().await?;

        let query = "UPDATE project SET status = 'archive', archived_at = $1 WHERE id = $2";
        let archived_at = chrono::Utc::now().timestamp();
        
        client.execute(query, &[&archived_at, &project_id]).await?;

        let project = self.retrieve_project(project_id.clone()).await?;
        Ok(project)
    }
}
