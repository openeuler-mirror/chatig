use crate::meta::users::traits::{UserObjectTrait, UserObject, UserObjectDto};
use crate::meta::users::impls::UserObjectImpl;

pub struct Usermanager {
    users: Box<dyn UserObjectTrait>,
}

impl Default for Usermanager {
    fn default() -> Self {
        Usermanager {
            users: Box::new(UserObjectImpl),
        }
    }
}

impl Usermanager {
    pub fn _new(users: Box<dyn UserObjectTrait>) -> Self {
        Usermanager { users }
    }

    pub async fn insert_user_object(&self, user: UserObjectDto) -> Result<UserObject, Box<dyn std::error::Error>> {
        self.users.insert_user_object(user).await
    }

    pub async fn list_user_objects(&self, limit: i64, after: Option<String>) -> Result<Vec<UserObject>, Box<dyn std::error::Error>> {
        self.users.list_user_objects(limit, after).await
    }

    pub async fn modify_user_object(&self, id: String, role: String) -> Result<UserObject, Box<dyn std::error::Error>> {
        self.users.modify_user_object(id, role).await
    }

    pub async fn retrieve_user_object(&self, id: String) -> Result<Option<UserObject>, Box<dyn std::error::Error>> {
        self.users.retrieve_user_object(id).await
    }

    pub async fn delete_user_object(&self, id: String) -> Result<u64, Box<dyn std::error::Error>> {
        self.users.delete_user_object(id).await
    }
}