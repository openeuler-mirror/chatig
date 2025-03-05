use core::str;
use serde::{Deserialize, Serialize};

// ------------------------------------------ OpenAI ------------------------------------------ 
#[derive(Deserialize, Serialize, Debug)]
pub struct OpenAIDeltaMessage {
    pub content: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OpenAIStreamChoice{
    pub index: u32,
    pub delta: OpenAIDeltaMessage,
    pub finish_reason: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OpenAIStreamResponse {
    pub id: String,              // Unique identifier for each generated response.
    pub model: String,           // Name of the model used.
    pub choices: Vec<OpenAIStreamChoice>,    // List of generated text options returned.
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

// Define the File API format accepted by the interface
#[derive(Deserialize, Serialize)]
pub struct FileChatResponse {
    pub answer: String,        // The answer.
    pub docs: Vec<String>,          // The content for the docs.
}

#[derive(Deserialize, Serialize)]
pub struct FileStreamChatResponse {
    pub answer: String,        // The answer.
}

#[derive(Deserialize, Serialize)]
pub struct FileDocStreamChatResponse {
    pub docs: Vec<String>,        // The answer.
}

#[derive(Deserialize, Debug)]
pub struct UploadTempDocsResponse {
    #[allow(dead_code)]
    pub code: u32,
    #[allow(dead_code)]
    pub msg: String,
    pub data: UploadTempDocsResponseData,
}

#[derive(Deserialize, Debug)]
pub struct FailedFile {
    #[serde(flatten)] // to handle the dynamic key inside `failed_files`
    #[allow(dead_code)]
    pub details: std::collections::HashMap<String, String>
}

#[derive(Deserialize, Debug)]
pub struct UploadTempDocsResponseData {
    pub id: String,
    #[allow(dead_code)]
    pub failed_files: Vec<FailedFile>,
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


