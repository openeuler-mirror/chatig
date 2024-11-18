use serde::{Deserialize, Serialize};
use tokio_postgres::NoTls;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;

use crate::configs::settings::Config;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db_pool: Pool<PostgresConnectionManager<NoTls>>, // 使用连接池
}

// ------------------------------------------ Completion API ------------------------------------------ 
#[derive(Deserialize, Serialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

// Define the API format accepted by the interface
#[derive(Deserialize, Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,                      // (Required) Name of the model used
    pub messages: Vec<Message>,             // (Required) List of messages, each message must contain `role` and `content`.
    #[allow(dead_code)]
    pub temperature: Option<f32>,           // Controls the creativity of the generated text.
    #[allow(dead_code)]
    pub top_p: Option<u32>,                 // An alternative sampling method to `temperature`. `top_p` selects tokens based on cumulative probability.
    #[allow(dead_code)]
    pub n: Option<u32>,                     // Number of generated responses.
    #[allow(dead_code)]
    pub stream: Option<bool>,               // Whether to enable streaming response. If `true`, the response will return parts of the content incrementally.
    #[allow(dead_code)]
    pub stop: Option<Vec<String>>,          // Strings that stop the generation, supports an array of strings.
    #[allow(dead_code)]
    pub max_tokens: Option<u32>,            // Maximum number of tokens generated per request.
    #[allow(dead_code)]
    pub presence_penalty: Option<i32>,      // Encourages the model to talk about new topics. Value ranges from `-2.0` to `2.0`.
    #[allow(dead_code)]
    pub frequency_penalty: Option<i32>,     // Controls the likelihood of generating repetitive tokens. Value ranges from `-2.0` to `2.0`, positive values reduce repetition.
    #[allow(dead_code)]
    pub logit_bias: Option<i32>,            // Adjusts the probability of specific tokens appearing. Value ranges from `-100` to `100`.
    #[allow(dead_code)]
    pub user: Option<String>,               // User ID to identify the source of the request.
    #[allow(dead_code)]
    pub file_id: Option<String>,            // File ID to identify the file.
}

#[derive(Serialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Serialize)]
pub struct AssistantMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct Choice {
    pub message: AssistantMessage,
    pub finish_reason: String,
    pub index: u32,
}

// Define the API format accepted by the interface
#[derive(Serialize)]
pub struct ChatCompletionResponse {
    pub id: String,              // Unique identifier for each generated response.
    pub object: String,          // Type of response object, such as `"chat.completion"`.
    pub created: u64,            // Timestamp of when the response was generated.
    pub model: String,           // Name of the model used.
    pub usage: Usage,            // Token usage, including `prompt_tokens` (number of prompt tokens), `completion_tokens` (number of generated content tokens), `total_tokens` (total number of tokens).
    pub choices: Vec<Choice>,    // List of generated text options returned.
}

// ------------------------------------------ General Error API ------------------------------------------
#[derive(Serialize, Debug)]
pub struct ErrorResponse {
    pub error: String,
}

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
#[derive(Serialize)]
pub struct Model {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub owned_by: String,
}