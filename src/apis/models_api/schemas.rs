use serde::{Deserialize, Serialize};
use utoipa::ToSchema;


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

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: Option<u32>,
    pub total_tokens: u32,
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
}

// Image data struct, which is part of the ImageGenerationResponse.
#[derive(Serialize, Deserialize, ToSchema)]
pub struct ImageData {
    pub url: Option<String>,  // URL or file path of the generated image.
    pub b64_json: Option<String>,  // Base64-encoded image data (if applicable).
}