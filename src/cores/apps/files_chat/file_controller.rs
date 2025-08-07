use actix_web::{web, Error, HttpResponse};
use actix_multipart::form::{json::Json as MpJson, tempfile::TempFile, MultipartForm};
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


// ------------------------------------------ Files ------------------------------------------

#[derive(Debug, Deserialize)]
pub struct Metadata {
    pub purpose: String,
}

#[derive(Debug, MultipartForm)]
pub struct UploadForm {
    #[multipart(limit = "100MB")]
    pub files: Vec<TempFile>,
    pub json: MpJson<Metadata>,
}

// ------------------------------------------ Files ------------------------------------------
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
pub struct UploadTempDocsResponseData {
    pub id: String,
    #[allow(dead_code)]
    pub failed_files: Vec<FailedFile>,
}

#[derive(Deserialize, Debug)]
pub struct FailedFile {
    #[serde(flatten)] // to handle the dynamic key inside `failed_files`
    #[allow(dead_code)]
    pub details: std::collections::HashMap<String, String>
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

// , payload: Multipart
#[async_trait]
pub trait FileChatController: Send + Sync {
    async fn upload_temp_docs(&self, MultipartForm(form): MultipartForm<UploadForm>) -> Result<HttpResponse, Error>;
    async fn file_chat_completions(&self, req_body: web::Json<ChatCompletionRequest>) -> Result<HttpResponse, Error>;
} 