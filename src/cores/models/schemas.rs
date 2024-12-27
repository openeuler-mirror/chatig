use core::str;
use serde::{Deserialize, Serialize};


// Define the API format accepted by the interface
#[derive(Deserialize, Serialize)]
pub struct CompletionsResponse {
    pub id: String,              // Unique identifier for each generated response.
    pub choices: Vec<CompletionsChoice>,    // List of generated text options returned.
    pub created: u64,            // Timestamp of when the response was generated.
    pub model: String,           // Name of the model used.
    pub object: String,          // Type of response object, such as `"chat.completion"`.
    pub service_tier: Option<String>,   // Service tier used for the request.
    pub system_fingerprint: Option<String>, // System fingerprint used for the request.
    pub usage: CompletionsUsage,            // Usage statistics for the request.
    pub message_id: Option<String>, // Unique identifier for the message.
    pub status: Option<String>,  // Status of the request.
}
#[derive(Deserialize, Serialize)]
pub struct CompletionsChoice {
    pub finish_reason: String,   // Reason for finishing the completion.
    pub index: u32,              // Index of the completion.
    pub logprobs: Option<String>, // Log probabilities for the completion.
    pub message: CompletionsAssistantMessage, // Message object containing the completion.
}

#[derive(Deserialize, Serialize)]
pub struct CompletionsAssistantMessage{
    pub content: String,        // Content of the completion.
    pub refusal: Option<String>, // Refusal message.
    pub role: String,           // Role of the assistant.
    pub function_call: Option<String>, // Function call.
    pub tool_calls: Option<Vec<String>>, // Tool calls.
}

#[derive(Deserialize, Serialize)]
pub struct CompletionsUsage {
    pub completion_tokens: u32,  // Number of tokens used for the completion.
    pub prompt_tokens: u32,      // Number of tokens used for the prompt.
    pub total_tokens: u32,       // Total number of tokens used.
}


#[derive(Deserialize, Serialize)]
pub struct CompletionsStreamResponse {
    pub id: String,              // Unique identifier for each generated response.
    pub choices: Vec<CompletionsStreamChoice>,    // List of generated text options returned.
    pub created: u64,            // Timestamp of when the response was generated.
    pub model: String,           // Name of the model used.
    pub object: String,          // Type of response object, such as `"chat.completion"`.
    pub service_tier: Option<String>,   // Service tier used for the request.
    pub system_fingerprint: Option<String>, // System fingerprint used for the request.
    pub usage: Option<CompletionsUsage>,            // Usage statistics for the request.
    pub message_id: Option<String>, // Unique identifier for the message.
    pub status: Option<String>,  // Status of the request.
}

#[derive(Deserialize, Serialize)]
pub struct CompletionsStreamChoice {
    pub finish_reason: Option<String>,   // Reason for finishing the completion.
    pub index: u32,              // Index of the completion.
    pub logprobs: Option<String>, // Log probabilities for the completion.
    pub delta: CompletionsDelta,   // delta object containing the completion.
}

#[derive(Deserialize, Serialize)]
pub struct CompletionsDelta {
    pub content: Option<String>,        // Content of the completion.
    pub function_call: Option<String>, // Function call.
    pub refusal: Option<String>, // Refusal message.
    pub role: Option<String>,           // Role of the assistant.
    pub tool_calls: Option<Vec<String>>, // Tool calls.
}
