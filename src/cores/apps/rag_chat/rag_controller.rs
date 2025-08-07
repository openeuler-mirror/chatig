use actix_web::{web, Error, HttpResponse};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// ==================================================== Completion Request Struct ====================================================
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ChatCompletionRequest {
    pub model: String,                      // (Required) Name of the model used
    pub messages: Vec<Message>,             // (Required) List of messages, each message must contain `role` and `content`.
    #[allow(dead_code)]
    pub temperature: Option<f32>,           // Controls the creativity of the generated text.
    #[allow(dead_code)]
    pub top_p: Option<f32>,                 // An alternative sampling method to `temperature`. `top_p` selects tokens based on cumulative probability.
    #[allow(dead_code)]
    pub n: Option<f32>,                     // Number of generated responses.
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
    pub stream_options: Option<StreamOptions>, // Stream options for the request.
    #[allow(dead_code)]
    pub file_id: Option<String>,            // File ID to identify the file.
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Message {
    pub role: String,
    pub content: String,                   // Updated to use Content enum
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct StreamOptions {
    pub include_usage: Option<bool>,
}

// ------------------------------------------ ChatChat ------------------------------------------ 
// Define the API format accepted by the interface
#[derive(Deserialize, Serialize)]
pub struct KbChatResponse {
    pub id: String,              // Unique identifier for each generated response.
    pub object: String,          // Type of response object, such as `"chat.completion"`.
    pub model: String,           // Name of the model used.
    pub created: u64,            // Timestamp of when the response was generated.
    #[allow(dead_code)]
    pub status: Option<String>,
    #[allow(dead_code)]
    pub message_type: u32,
    #[allow(dead_code)]
    pub message_id: Option<String>,
    #[allow(dead_code)]
    pub is_ref: bool,
    pub choices: Vec<KbChoice>,    // List of generated text options returned.
}

#[derive(Deserialize, Serialize)]
pub struct KbChoice {
    pub message: KbAssistantMessage,
}

#[derive(Deserialize, Serialize)]
pub struct KbAssistantMessage {
    #[allow(dead_code)]
    pub role: String,
    pub content: String,
    #[allow(dead_code)]
    pub finish_reason: Option<String>,
    #[allow(dead_code)]
    pub tool_calls: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct KbChatStreamResponse {
    pub id: String,              // Unique identifier for each generated response.
    pub object: String,          // Type of response object, such as `"chat.completion"`.
    pub model: String,           // Name of the model used.
    pub created: u64,            // Timestamp of when the response was generated.
    #[allow(dead_code)]
    pub status: Option<String>,
    #[allow(dead_code)]
    pub message_type: u32,
    #[allow(dead_code)]
    pub message_id: Option<String>,
    #[allow(dead_code)]
    pub is_ref: bool,
    pub choices: Vec<KbStreamChoice>,    // List of generated text options returned.
}

#[derive(Deserialize, Serialize, Debug)]
pub struct KbStreamChoice {
    pub delta: KbDelta,
    pub role: String
}

#[derive(Deserialize, Serialize, Debug)]
pub struct KbDelta {
    pub content: String,
    pub tool_calls: Option<Vec<String>>
}


// ------------------------------------------ EulerCopilot ------------------------------------------ 
#[derive(Deserialize, Serialize)]
pub struct GetAnswerResponse{
    pub answer: String,
    pub sources: Vec<String>,
    pub source_contents: Vec<String>,
    pub scores: Option<Vec<f32>>,
}

#[derive(Deserialize, Serialize)]
pub struct GetStreamAnswerResponse{
    pub content: String,
}

// ------------------------------------------ OpenAI ------------------------------------------ 
#[derive(Deserialize, Serialize, Debug)]
pub struct OpenAIStreamResponse {
    pub id: String,              // Unique identifier for each generated response.
    pub model: String,           // Name of the model used.
    pub choices: Vec<OpenAIStreamChoice>,    // List of generated text options returned.
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OpenAIStreamChoice{
    pub index: u32,
    pub delta: OpenAIDeltaMessage,
    pub finish_reason: String,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct OpenAIDeltaMessage {
    pub content: String,
}

#[async_trait]
pub trait RAGController: Send + Sync {
    async fn rag_chat_completions(&self, req_body: web::Json<ChatCompletionRequest>) -> Result<HttpResponse, Error>;
}