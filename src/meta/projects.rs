use serde::{Serialize, Deserialize};

use crate::meta::init::get_pool;

// project_object table structure
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectObject {
    pub id: String,                    // The identifier, which can be referenced in API endpoints
    pub object: String,             // The object type, which is always organization.project
    pub name: String,               // The name of the project. This appears in reporting.
    pub created_at: i64,            // The Unix timestamp (in seconds) of when the project was created.
    pub archived_at: Option<i64>,   // The Unix timestamp (in seconds) of when the project was archived or null.
    pub status: String,             // active or archive
}

// List all project objects
pub async fn list_project_objects(
    limit: i64, 
    after: Option<String>, 
    include_archived: bool,
) -> Result<Vec<ProjectObject>, Box<dyn std::error::Error>> {
    let pool = get_pool().await?;
    let client = pool.get().await?;

    // dynamic build query and parameters
    let mut query = String::from(
        "SELECT * FROM project_object WHERE 1=1"
    );
    let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync>> = Vec::new();

    // if after is not None, add condition
    if let Some(after_value) = after {
        query.push_str(" AND id > $1");
        params.push(Box::new(after_value));
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
    params.push(Box::new(limit));

    let rows = client.query(&query, &params.iter().map(|b| &**b).collect::<Vec<_>>()).await?;

    let mut project_objects = Vec::new();
    for row in rows {
        let project_object = ProjectObject {
            id: row.get(0),
            object: row.get(1),
            name: row.get(2),
            created_at: row.get(3),
            archived_at: row.get(4),
            status: row.get(5),
        };
        project_objects.push(project_object);
    }

    Ok(project_objects)
}

// Create project object
pub async fn create_project_object(
    project_object: ProjectObject,
) -> Result<(), Box<dyn std::error::Error>> {
    let pool = get_pool().await?;
    let client = pool.get().await?;

    let query = "
        INSERT INTO project_object (id, object, name, created_at, archived_at, status)
        VALUES ($1, $2, $3, $4, $5, $6)";
    
    client.execute(query, &[
        &project_object.id,
        &project_object.object,
        &project_object.name,
        &project_object.created_at,
        &project_object.archived_at,
        &project_object.status,
    ]).await?;

    Ok(())
}

// Retrieve project object
pub async fn retrieve_project_object(
    project_id: String,
) -> Result<ProjectObject, Box<dyn std::error::Error>> {
    let pool = get_pool().await?;
    let client = pool.get().await?;

    let query = "SELECT id, object, name, created_at, archived_at, status FROM project_object WHERE id = $1";
    let row = client.query_one(query, &[&project_id]).await?;

    let project_object = ProjectObject {
        id: row.get(0),
        object: row.get(1),
        name: row.get(2),
        created_at: row.get(3),
        archived_at: row.get(4),
        status: row.get(5),
    };

    Ok(project_object)
}

// Modify project object
pub async fn modify_project_object(
    project_id: String,
    project_name: String,
) -> Result<ProjectObject, Box<dyn std::error::Error>> {
    let pool = get_pool().await?;
    let client = pool.get().await?;

    let query = "
        UPDATE project_object SET name = $1 WHERE id = $2";
    
    client.execute(query, &[
        &project_name,
        &project_id,
    ]).await?;

    let project = retrieve_project_object(project_id.clone()).await?;

    Ok(project)
}

// Archive project object
pub async fn archive_project_object(
    project_id: String,
) -> Result<ProjectObject, Box<dyn std::error::Error>> {
    let pool = get_pool().await?;
    let client = pool.get().await?;

    let query = "UPDATE project_object SET status = 'archive', archived_at = $1 WHERE id = $2";
    let archived_at = chrono::Utc::now().timestamp();
    
    client.execute(query, &[&archived_at, &project_id]).await?;

    let project = retrieve_project_object(project_id.clone()).await?;
    Ok(project)
}