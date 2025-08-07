use actix_web::{web, Error, HttpResponse};
use async_trait::async_trait;
use core::str;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ==================================================== Standard Rerank ====================================================
// Request struct
#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub struct StdRerankRequest {
    pub model: String,
    pub query: String,
    pub documents: Vec<String>,
    pub top_n: Option<u32>,                      // Adapted for LlamaBox
    pub parameters: Option<RerankParameters>     // Adapted for Alibaba
}

#[derive(Deserialize, Serialize, ToSchema, Clone, Debug, Copy)]
pub struct RerankParameters {
    pub top_n: Option<u32>,
    pub return_documents: Option<bool>
}

// response struct
#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub struct StdRerankResponse {
    pub results: StdRerankResult
}

#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub struct StdRerankResult {
    pub index: u32,
    pub score: f32
}

// ==================================================== LlamaBox Rerank ====================================================
// Request Struct
#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub struct LlamaBoxRerankRequest {
    pub model: String,
    pub query: String,
    pub documents: Vec<String>,
    pub top_n: Option<u32>
}

// Response Struct
#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub struct LlamaBoxRerankResponse {
    pub model: String,
    pub query: Option<String>,
    pub results: Vec<LlamaRerankResult>,
    pub usage: Usage
}

#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub struct LlamaRerankResult {
    pub index: u32,
    pub relevance_score: f32,
    pub document: Document
}

#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub struct Document {
    pub text: String
}

#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub total_tokens: u32
}

// ==================================================== VLLM Rerank ====================================================
// Request Struct
#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub struct VLLMRerankRequest {
    pub model: String,
    pub query: String,
    pub documents: Vec<String>
}

// Response Struct
#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub struct VLLMRerankRespnse {
    pub id: String,
    pub model: String,
    pub usage: VLLMUsage,
    pub results: Vec<VLLMRerankResult>
}

#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub struct VLLMUsage {
    pub total_tokens: u32
}

#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub struct VLLMRerankResult {
    pub index: u32,
    pub document: VLLMDocument,
    pub relevance_score: f32
}

#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub struct VLLMDocument {
    pub text: String
}

// ==================================================== Mindie Rerank ====================================================
#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub struct MindieRerankRequest {
    pub query: String,
    pub texts: Vec<String>
}

#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub struct MindieRerankResponse {
    pub index: u32,
    pub score: f32
}


// ==================================================== Rerank Trait ====================================================
#[async_trait]
pub trait StdRerankTrait: Send + Sync {
    async fn rerank(&self, req_body: web::Json<StdRerankRequest>, aicpid: String) -> Result<HttpResponse, Error>;
}

#[async_trait]
pub trait LlamaBoxRerankTrait: Send + Sync {
    async fn rerank(&self, req_body: web::Json<LlamaBoxRerankRequest>, aicpid: String) -> Result<HttpResponse, Error>;
}

#[async_trait]
pub trait VLLMRerankTrait: Send + Sync {
    async fn rerank(&self, req_body: web::Json<VLLMRerankRequest>, aicpid: String) -> Result<HttpResponse, Error>;
}
