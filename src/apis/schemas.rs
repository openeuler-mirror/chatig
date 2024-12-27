use serde::Serialize;

// ------------------------------------------ General Error API ------------------------------------------
#[derive(Serialize, Debug)]
pub struct ErrorResponse {
    pub error: String,
}