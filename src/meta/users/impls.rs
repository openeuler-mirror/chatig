use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;

use crate::meta::users::traits::{UserObjectTrait, UserObject, UserObjectDto};
use crate::meta::connection::DBCrud;

pub struct UserObjectImpl;

#[async_trait]
impl UserObjectTrait for UserObjectImpl {
    // Insert a user object
    async fn insert_user_object(&self, user: UserObjectDto) -> Result<UserObject, Box<dyn std::error::Error>> {
        let name = user.name;
        let created_at = chrono::Utc::now().timestamp();
        let id = format!("{}_{}", name, created_at);
        let user_object = json!({
            "id": id,
            "object": &"organization.user",
            "name": name,
            "email": user.email,
            "role": user.role,
            "added_at": &created_at,
        });
        DBCrud::create("user_object", &user_object).await?;

        // convert user_object to UserObject
        let user_object = serde_json::from_value::<UserObject>(user_object).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        Ok(user_object)
    }

    // List all user objects
    async fn list_user_objects(&self, limit: i64, after: Option<String>) -> Result<Vec<UserObject>, Box<dyn std::error::Error>> {
        let mut conditions = HashMap::new();

        match after {
            Some(after_value) => {
                let json_value = json!(after_value.clone());
                conditions.insert("id", &json_value);  // Insert reference
                let user_objects = DBCrud::list_with_pagination::<UserObject>(
                    "user_object",
                    Some(conditions),
                    Some(("id", "ASC")),
                    limit,
                )
                .await?;
            
                Ok(user_objects)
            },
            None => {
                let user_objects = DBCrud::list_with_pagination::<UserObject>(
                    "user_object",
                    None,
                    Some(("id", "ASC")),
                    limit,
                )
                .await?;
            
                Ok(user_objects)
            }
        }
    

    }

    // Modify the user_object table
    async fn modify_user_object(&self, id: String, role: String) -> Result<UserObject, Box<dyn std::error::Error>> {
        // Prepare updates and conditions
        let updates = &[("role", json!(role))];
        let conditions = &[("id", json!(id))];

        // Perform the update
        DBCrud::update("user_object", updates, Some(conditions)).await?;

        // Clone id before moving it
        let id_clone = id.clone();
        // Return the modified user object
        match self.retrieve_user_object(id).await? {
            Some(user_object) => Ok(user_object),
            None => Err(format!("User object not found with id: {}", &id_clone).into()),
        }
    }

    // Retrieve a user object
    async fn retrieve_user_object(&self, id: String) -> Result<Option<UserObject>, Box<dyn std::error::Error>> {
        // Fetch the user object from the database
        let user_object = DBCrud::get::<UserObject>("user_object", "id", &json!(id)).await?;

        Ok(user_object)
    }

    // Delete a user object
    async fn delete_user_object(&self, id: String) -> Result<u64, Box<dyn std::error::Error>> {
        // Delete the user object from the database
        let conditions = &[("id", json!(id))];
        let delete_num = DBCrud::delete("user_object", Some(conditions)).await?;

        Ok(delete_num)
    }
}


