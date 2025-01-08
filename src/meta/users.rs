use serde::{Serialize, Deserialize};

use crate::meta::init::get_pool;

// user_object table structure
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserObject {
    pub id: String,                 // The identifier, which can be referenced in API endpoints
    pub object: String,             // The object type, which is always organization.user
    pub name: String,               // The name of the user
    pub email: String,              // The email address of the user
    pub role: String,               // owner or reader
    pub added_at: i64,              // The Unix timestamp (in seconds) of when the user was added.
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserObjectDto {
    pub name: String,               // The name of the user
    pub email: String,              // The email address of the user
    pub role: String,               // owner or reader
}

// Insert a user object
pub async fn insert_user_object(
    user: UserObjectDto,
) -> Result<UserObject, Box<dyn std::error::Error>> {
    let name = user.name;
    let email = user.email;
    let role = user.role;
    let created_at = chrono::Utc::now().timestamp();
    let id = format!("{}_{}", name, created_at);

    let pool = get_pool().await?;
    let client = pool.get().await?;
    let query = String::from(
        "INSERT INTO user_object (id, object, name, email, role, added_at) VALUES ($1, $2, $3, $4, $5, $6)"
    );
    client.execute(&query, &[&id, &"organization.user", &name, &email, &role, &created_at]).await?;

    let user_object = UserObject {
        id: id,
        object: String::from("organization.user"),
        name: name,
        email: email,
        role: role,
        added_at: created_at,
    };

    Ok(user_object)
}

// List all user objects
pub async fn list_user_objects(
    limit: i64, 
    after: Option<String>, 
) -> Result<Vec<UserObject>, Box<dyn std::error::Error>> {
    let pool = get_pool().await?;
    let client = pool.get().await?;

    // dynamic build query and parameters
    let mut query = String::from(
        "SELECT * FROM user_object WHERE 1=1"
    );
    let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync>> = Vec::new();

    // if after is not None, add condition
    if let Some(after_value) = after {
        query.push_str(" AND id > $1");
        params.push(Box::new(after_value));
    }

    // add order and limit
    if params.is_empty() {
        query.push_str(" ORDER BY id ASC LIMIT $1");
    } else {
        query.push_str(" ORDER BY id ASC LIMIT $2");
    }
    params.push(Box::new(limit));

    let rows = client.query(&query, &params.iter().map(|b| &**b).collect::<Vec<_>>()).await?;

    let mut user_objects = Vec::new();
    for row in rows {
        let user_object = UserObject {
            id: row.get(0),
            object: row.get(1),
            name: row.get(2),
            email: row.get(3),
            role: row.get(4),
            added_at: row.get(5),
        };
        user_objects.push(user_object);
    }

    Ok(user_objects)
}

// Modify the user_object table
pub async fn modify_user_object(
    id: String,
    role: String,
) -> Result<UserObject, Box<dyn std::error::Error>> {
    let pool = get_pool().await?;
    let client = pool.get().await?;

    // dynamic build query and parameters
    let query = String::from(
        "UPDATE user_object SET role = $1 WHERE id = $2"
    );

    client.execute(&query, &[&role, &id]).await?;

    // return the modified user object
    let user_object = retrieve_user_object(id).await?;

    Ok(user_object)
}

// Retrieve a user object
pub async fn retrieve_user_object(
    id: String,
) -> Result<UserObject, Box<dyn std::error::Error>> {
    let pool = get_pool().await?;
    let client = pool.get().await?;

    let row = client.query_one("SELECT * FROM user_object WHERE id = $1", &[&id]).await?;
    let user_object = UserObject {
        id: row.get(0),
        object: row.get(1),
        name: row.get(2),
        email: row.get(3),
        role: row.get(4),
        added_at: row.get(5),
    };

    Ok(user_object)
}

// Delete a user object
pub async fn delete_user_object(
    id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let pool = get_pool().await?;
    let client = pool.get().await?;

    let _ = client.execute("DELETE FROM user_object WHERE id = $1", &[&id]).await?;

    Ok(())
}