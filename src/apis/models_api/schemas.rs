use serde::{Deserialize, Serialize};
use utoipa::ToSchema;


// ------------------------------------------ Completion API ------------------------------------------ 
#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct Message {
    pub role: String,
    pub content: String,
}

// Define the API format accepted by the interface
#[derive(Deserialize, Serialize, ToSchema)]
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

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: Option<u32>,
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

// Define the request struct, corresponding to the request parameters of the /v1/embeddings interface.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct EmbeddingRequest {
    pub input: Vec<String>,
    pub model: String,
    #[allow(dead_code)]
    pub encoding_format: Option<String>,  // Optional, used to specify the format of the returned embedding vectors.
    #[allow(dead_code)]
    pub dimensions: Option<u32>,          // Optional, used to specify the dimension number of the generated embeddings.
    #[allow(dead_code)]
    pub user: Option<String>,            // Optional, represents the unique identifier of the end user.
}

// Define the response struct, corresponding to the response data format of the /v1/embeddings interface.
#[derive(Serialize, Deserialize, ToSchema)]
pub struct EmbeddingResponse {
    pub object: String,
    pub data: Vec<EmbeddingData>,
    pub model: String,
    pub usage: Usage,
}

// Embedding data struct, which is part of the EmbeddingResponse.
#[derive(Serialize, Deserialize, ToSchema)]
pub struct EmbeddingData {
    pub object: String,
    pub embedding: Vec<f32>,
    pub index: usize,
}

// Define the request struct, corresponding to the request parameters of the /v1/images/generations interface.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct ImageGenerationRequest {
    pub model: String,
    pub prompt: String,
    #[allow(dead_code)]
    pub size: Option<String>,  // Optional, used to specify the size of the generated image.
    #[allow(dead_code)]
    pub num_images: Option<u32>,  // Optional, used to specify the number of images to generate.
    #[allow(dead_code)]
    pub user: Option<String>,  // Optional, represents the unique identifier of the end user.
}

// Define the response struct, corresponding to the response data format of the /v1/images/generations interface.
#[derive(Serialize, Deserialize, ToSchema)]
pub struct ImageGenerationResponse {
    pub created: u64,  // Timestamp of when the image was generated.
    pub data: Vec<ImageData>,
    // #[allow(dead_code)]
    // pub model: String,
    // #[allow(dead_code)]
    // pub usage: Usage,
}

// Image data struct, which is part of the ImageGenerationResponse.
#[derive(Serialize, Deserialize, ToSchema)]
pub struct ImageData {
    pub url: Option<String>,  // URL or file path of the generated image.
    pub b64_json: Option<String>,  // Base64-encoded image data (if applicable).
}