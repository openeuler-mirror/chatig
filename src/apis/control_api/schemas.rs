use serde::{Deserialize, Serialize};


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