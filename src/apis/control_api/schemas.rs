use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// ------------------------------------------ Models API ------------------------------------------
/*
From https://platform.openai.com/docs/api-reference/models/object
{
  "id": "davinci",
  "object": "model",  // The object type, which is always "model".
  "created": 1686935002, // The Unix timestamp (in seconds) when the model was created.
  "owned_by": "openai"
}
*/
#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct Model {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub owned_by: String,
}

// ------------------------------------------ Invitation Code API ------------------------------------------ 
// Define the invitation code API format accepted by the interface
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InvitationCodeRequest {
    pub user: String,                      // (Required) User name.
    #[allow(dead_code)]
    pub origination: Option<String>,              // The organization the user belongs to.
    #[allow(dead_code)]
    pub telephone: Option<String>,              // The telephone number of the user.
    #[allow(dead_code)]
    pub email: Option<String>,                 // The email address of the user.
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InvitationCodeResponse {
    pub id: String,                        // The generated invitation code for the user.
}