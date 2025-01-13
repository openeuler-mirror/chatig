use serde::Serialize;
use utoipa::ToSchema;

// ------------------------------------------ General Error API ------------------------------------------
#[derive(Serialize, Debug, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}