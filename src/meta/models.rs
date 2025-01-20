use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;


// ------------------------------------------ Models API ------------------------------------------
/*
From https://platform.openai.com/docs/api-reference/models/object
{
  "id": "Qwen/Qwen2.5-7B-Instruct",
  "object": "model",  // The object type, which is always "chat, embedding, image_generation".
  "model_name": "Qwen2.5-7B-Instruct", // The name of the model. which is related to the inference engine.
  "request_url": "http://x.x.x.x:8000/v1/chat/completions", // The URL to send requests to.
  "created": 1686935002, // The Unix timestamp (in seconds) when the model was created.
  "owned_by": "openai"
}
*/
#[derive(Serialize, Deserialize, Debug, Clone, FromRow, ToSchema)]
pub struct Model {
    pub id: String,
    pub object: String,
    pub model_name: String,
    pub request_url: String,
    pub created: i64,
    pub owned_by: String,
}