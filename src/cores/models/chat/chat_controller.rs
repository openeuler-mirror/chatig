use actix_web::{web, Error, HttpResponse};
use async_trait::async_trait;
use core::str;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

// ==================================================== Completion Request Struct ====================================================
#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub struct ChatCompletionRequest {
    pub model: String,                      // (Required) Name of the model used
    pub messages: Vec<Message>,             // (Required) List of messages, each message must contain `role` and `content`.
    pub temperature: Option<f32>,           // Controls the creativity of the generated text.
    pub top_p: Option<f32>,                 // An alternative sampling method to `temperature`. `top_p` selects tokens based on cumulative probability.
    pub n: Option<f32>,                     // Number of generated responses.
    pub stop: Option<Vec<String>>,          // Strings that stop the generation, supports an array of strings.
    pub max_tokens: Option<u32>,            // Maximum number of tokens generated per request.
    pub presence_penalty: Option<f32>,      // Encourages the model to talk about new topics. Value ranges from `-2.0` to `2.0`.
    pub frequency_penalty: Option<f32>,     // Controls the likelihood of generating repetitive tokens. Value ranges from `-2.0` to `2.0`, positive values reduce repetition.
    pub logit_bias: Option<f32>,            // Adjusts the probability of specific tokens appearing. Value ranges from `-100` to `100`.
    pub stream: Option<bool>,               // Whether to enable streaming response. If `true`, the response will return parts of the content incrementally.
    pub stream_options: Option<StreamOptions>, // Stream options for the request.
    pub tool_choice: Option<ToolChoice>, // Tool choice for the request.
    pub tools: Option<Vec<Tool>>,         // List of tools to be used in the request.
    pub user: Option<String>,               // User ID to identify the source of the request.
}

#[derive(Deserialize, Serialize, Clone, ToSchema, Debug)]
pub struct Message {
    pub role: String,
    pub content: Content,                   // Updated to use Content enum
    pub name: Option<String>,               // Name of the user or assistant
}

// Enum to represent different content types
#[derive(Deserialize, Serialize, Clone, ToSchema, Debug)]
#[serde(untagged)] // Untagged to allow flexible JSON parsing
pub enum Content {
    Text(String),                           // Simple string content
    Array(Vec<ContentPart>),                // Array of content parts
    Assistant(AssistantContent),
}

// Structs for different content part types
#[derive(Deserialize, Serialize, Clone, ToSchema, Debug)]
pub struct ContentPart {
    pub r#type: String,                     // Type of the content part (e.g., "text", "image_url", "input_audio", "file")
    #[serde(flatten)]                       // Flatten to include specific fields based on type
    pub data: ContentData,
}

#[derive(Deserialize, Serialize, Clone, ToSchema, Debug)]
#[serde(untagged)] // Untagged to handle different content data structures
pub enum ContentData {
    Text { text: String },                  // Text content
    Image { image_url: ImageUrl },          // Image content
    Audio { input_audio: AudioInput },      // Audio content
    File { file: FileInput },               // File content
}

#[derive(Deserialize, Serialize, Clone, ToSchema, Debug)]
pub struct ImageUrl {
    pub url: String,                        // URL of the image
}

#[derive(Deserialize, Serialize, Clone, ToSchema, Debug)]
pub struct AudioInput {
    pub url: String,                        // URL or identifier of the audio
}

#[derive(Deserialize, Serialize, Clone, ToSchema, Debug)]
pub struct FileInput {
    pub id: String,                         // File ID
}

#[derive(Deserialize, Serialize, Clone, ToSchema, Debug)]
pub struct AssistantContent {
    pub content: Option<String>,           // 可选的文本内容
    pub audio: Option<AudioOutput>,        // 可选的音频数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<FunctionCall>, // 已废弃的 function_call
    pub tool_calls: Option<Vec<ToolCall>>, // 可选的工具调用
    pub refusal: Option<String>,           // 可选的拒绝消息
}

#[derive(Deserialize, Serialize, Clone, ToSchema, Debug)]
pub struct AudioOutput {
    pub url: String,  // 示例字段，可根据实际需求调整
}

#[derive(Deserialize, Serialize, Clone, ToSchema, Debug)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,  // 示例字段，通常是 JSON 字符串
}

#[derive(Deserialize, Serialize, Clone, ToSchema, Debug)]
pub struct ToolCall {
    pub id: String,         // 示例字段
    pub r#type: String,     // 工具类型，如 "function"
    pub function: FunctionCall, // 示例字段
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct StreamOptions {
    pub include_usage: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    /// String-based tool choice: "none", "auto", or "required".
    String(String),
    /// Object-based tool choice, specifying a function to call.
    Object(ToolChoiceObject),
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolChoiceObject {
    /// The type of the tool (currently only "function" is supported).
    pub r#type: String,
    /// The function details for the tool.
    pub function: ToolChoiceFunction,
}

/// Represents the function details within a tool choice object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolChoiceFunction {
    /// The name of the function to call.
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tool {
    /// The type of the tool (currently only "function" is supported).
    pub r#type: String,
    /// The function details for the tool.
    pub function: ToolFunction,
}

/// Represents the function details within a tool.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolFunction {
    /// The name of the function to be called.
    pub name: String,
    /// Optional description of what the function does.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional parameters the function accepts, described as a JSON Schema object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Value>,
    /// Optional flag to enable strict schema adherence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

// ==================================================== Completion Response Struct ====================================================
// Define the API format accepted by the interface
#[derive(Deserialize, Serialize, Debug, ToSchema, Clone)]
pub struct CompletionsResponse {
    pub id: String,                           // Unique identifier for each generated response.
    pub object: String,                       // Type of response object, such as `"chat.completion"`.
    pub created: u64,                         // Timestamp of when the response was generated.
    pub model: String,                        // Name of the model used.
    pub choices: Vec<CompletionsChoice>,      // List of generated text options returned.
    pub usage: CompletionsUsage,              // Usage statistics for the request.

    // Optional fields for OpenAI
    pub system_fingerprint: Option<String>,   // System fingerprint used for the request.
    pub prompt_logprobs: Option<String>,      // Log probabilities for the prompt.

    // Optional fields for mindie
    pub prefill_time: Option<u64>,            // Time taken to generate the response.
    pub decode_time_arr: Option<Vec<u64>>,    // Array of times taken for each decoding step.
}
#[derive(Deserialize, Serialize, Debug, ToSchema, Clone)]
pub struct CompletionsChoice {
    pub index: u32,                                  // Index of the completion.
    pub finish_reason: String,                       // Reason for finishing the completion.
    pub message: CompletionsAssistantMessage,        // Message object containing the completion.

    // Optional fields for OpenAI
    pub logprobs: Option<String>,                    // Log probabilities for the completion.
    pub stop_reason: Option<String>,                 // Reason for stopping the completion.
}

#[derive(Deserialize, Serialize, Debug, ToSchema, Clone)]
pub struct CompletionsAssistantMessage{
    pub role: String,                               // Role of the assistant.
    pub content: String,                            // Content of the completion.

    pub reasoning_content: Option<String>,         // Reasoning content.
    pub refusal: Option<String>,                    // Refusal message.
    pub tool_calls: Option<Vec<MessageToolCall>>,            // Tool calls.
}

#[derive(Deserialize, Serialize, Debug, ToSchema, Clone)]
pub struct MessageToolCall{
    pub id: String,                               // Unique identifier for the tool call.
    pub r#type: String,                           // Type of the tool call (e.g., "function").
    pub function: ToolCallFunction,                   // Function details for the tool call.
}

#[derive(Deserialize, Serialize, Debug, ToSchema, Clone)]
pub struct ToolCallFunction{
    pub name: String,                            // Name of the function to be called.
    pub arguments: String,                       // Arguments for the function call.
}

#[derive(Deserialize, Serialize, Debug, ToSchema, Clone)]
pub struct CompletionsUsage {
    pub completion_tokens: u32,            // Number of tokens used for the completion.
    pub prompt_tokens: u32,                // Number of tokens used for the prompt.
    pub total_tokens: u32,                 // Total number of tokens used.
    pub prompt_tokens_details: Option<PromptTokensDetails>,  // Details of the prompt tokens used.
}

#[derive(Deserialize, Serialize, Debug, ToSchema, Clone)]
pub struct PromptTokensDetails {
    pub cached_tokens: u32,                   // Number of tokens used for the completion.
}

// ==================================================== Completion Stream Response Struct ====================================================
#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub struct CompletionsStreamResponse {
    pub id: String,                                 // Unique identifier for each generated response.
    pub choices: Option<Vec<CompletionsStreamChoice>>,      // List of generated text options returned.
    pub created: u64,                               // Timestamp of when the response was generated.
    pub model: String,                              // Name of the model used.
    pub object: String,                             // Type of response object, such as `"chat.completion"`.
    pub system_fingerprint: Option<String>,         // System fingerprint used for the request.
    pub usage: Option<CompletionsUsage>,            // Usage statistics for the request.
}

#[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
pub struct CompletionsStreamChoice {
    pub finish_reason: Option<String>,             // Reason for finishing the completion.
    pub index: Option<u32>,                                // Index of the completion.
    pub logprobs: Option<String>,                  // Log probabilities for the completion.
    pub delta: Option<CompletionsDelta>,                   // delta object containing the completion.
    pub stop_reason: Option<String>,               // Reason for stopping the completion.
}

#[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
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
    async fn completions(&self, req_body: web::Json<ChatCompletionRequest>, user_name: String, user_id: String, 
        service_name: String, service_id: String, user_type: String, user_level: String, pay_status: String, aicpid: String) -> Result<HttpResponse, Error>;
}