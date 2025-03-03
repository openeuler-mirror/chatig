use actix_web::{web, Error, HttpResponse};
use async_trait::async_trait;
use core::str;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ==================================================== Completion Request Struct ====================================================
#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
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
    pub stream_options: Option<StreamOptions>, // Stream options for the request.
    #[allow(dead_code)]
    pub file_id: Option<String>,            // File ID to identify the file.
}

#[derive(Deserialize, Serialize, Clone, ToSchema, Debug)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct StreamOptions {
    pub include_usage: bool,
}

// ==================================================== Completion Response Struct ====================================================
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

#[derive(Serialize)]
pub struct Choice {
    pub message: AssistantMessage,
    pub finish_reason: String,
    pub index: u32,
}

#[derive(Serialize)]
pub struct AssistantMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: Option<u32>,
    pub total_tokens: u32,
}

// Define the API format accepted by the interface
#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct CompletionsResponse {
    pub id: String,                           // Unique identifier for each generated response.
    pub object: String,                       // Type of response object, such as `"chat.completion"`.
    pub created: u64,                         // Timestamp of when the response was generated.
    pub model: String,                        // Name of the model used.
    pub choices: Vec<CompletionsChoice>,      // List of generated text options returned.
    pub usage: CompletionsUsage,              // Usage statistics for the request.
    pub system_fingerprint: Option<String>,   // System fingerprint used for the request.
    pub prompt_logprobs: Option<String>,      // Log probabilities for the prompt.
}
#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct CompletionsChoice {
    pub index: u32,                                  // Index of the completion.
    pub message: CompletionsAssistantMessage,        // Message object containing the completion.
    pub logprobs: Option<String>,                    // Log probabilities for the completion.
    pub finish_reason: String,                       // Reason for finishing the completion.
    pub stop_reason: Option<String>,                 // Reason for stopping the completion.
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct CompletionsAssistantMessage{
    pub role: String,                               // Role of the assistant.
    pub reasoning_content: Option<String>,         // Reasoning content.
    pub content: String,                            // Content of the completion.
    pub refusal: Option<String>,                    // Refusal message.
    pub tool_calls: Option<Vec<String>>,            // Tool calls.
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct CompletionsUsage {
    pub completion_tokens: u32,            // Number of tokens used for the completion.
    pub prompt_tokens: u32,                // Number of tokens used for the prompt.
    pub total_tokens: u32,                 // Total number of tokens used.
    pub prompt_tokens_details: Option<PromptTokensDetails>,  // Details of the prompt tokens used.
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct PromptTokensDetails {
    pub cached_tokens: u32,                   // Number of tokens used for the completion.
}


#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub struct CompletionsStreamResponse {
    pub id: String,                                 // Unique identifier for each generated response.
    pub choices: Vec<CompletionsStreamChoice>,      // List of generated text options returned.
    pub created: u64,                               // Timestamp of when the response was generated.
    pub model: String,                              // Name of the model used.
    pub object: String,                             // Type of response object, such as `"chat.completion"`.
    pub system_fingerprint: Option<String>,         // System fingerprint used for the request.
    pub usage: Option<CompletionsUsage>,            // Usage statistics for the request.
}

#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub struct CompletionsStreamChoice {
    pub finish_reason: Option<String>,             // Reason for finishing the completion.
    pub index: u32,                                // Index of the completion.
    pub logprobs: Option<String>,                  // Log probabilities for the completion.
    pub delta: CompletionsDelta,                   // delta object containing the completion.
    pub stop_reason: Option<String>,               // Reason for stopping the completion.
}

#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub struct CompletionsDelta {
    pub role: Option<String>,               // Role of the assistant.
    pub content: Option<String>,            // Content of the completion.
    pub refusal: Option<String>,            // Refusal message.
    pub function_call: Option<String>,      // Function call.
    pub tool_calls: Option<Vec<String>>,    // Tool calls.
}


// ==================================================== Completion Trait ====================================================
#[async_trait]
pub trait Completions: Send + Sync {
    async fn completions(&self, req_body: web::Json<ChatCompletionRequest>, userid: String, appkey: String) -> Result<HttpResponse, Error>;
}