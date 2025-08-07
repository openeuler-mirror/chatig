use actix_web::{web, Error, HttpResponse};
use async_trait::async_trait;
use core::str;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ==================================================== Generate Request Struct ====================================================
#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub struct GenerateRequest {
    pub inputs: Input,
    pub model: String,
    pub parameters: Option<GenerateParameters>,
}

#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
#[serde(untagged)]
pub enum Input {
    Single(String),
    Multi(Vec<InputPart>),
}

#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub struct InputPart {
    #[serde(rename = "type")]
    pub input_type: String,  // 类型，如 "text"、"image_url"、"video_url"、
    #[serde(flatten)]
    pub data: InputData,
}

#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
#[serde(untagged)]
pub enum InputData {
    Text { text: String }, // 文本内容
    Image { image_url: String }, // 图片 URL
    Video { video_url: String }, // 视频 URL
    Audio { audio_url: String }, // 音频 URL
}

#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub struct GenerateParameters {
    pub decoder_input_details: Option<bool>,
    pub details: Option<bool>,
    pub do_sample: Option<bool>,
    pub max_new_tokens: Option<u32>,
    pub repetition_penalty: Option<f32>,
    pub return_full_text: Option<bool>,
    pub seed: Option<u32>,
    pub temperature: Option<f32>,
    pub top_k: Option<u32>,
    pub top_p: Option<f32>,
    pub truncate: Option<u32>,
    pub typical_p: Option<f32>,
    pub watermark: Option<bool>,
    pub stop: Option<String>,
    pub adapter_id: Option<String>,
}
// ==================================================== Generate Response Struct ====================================================

#[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
pub struct GenerateResponse {
    pub details: Option<GenerateDetails>,
    pub generated_text: String, // 生成的文本
}

#[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
pub struct GenerateDetails {
    pub finish_reason: String, // 结束原因
    pub generated_tokens: u32, // 生成的 token 数量
    pub prefill: Option<Vec<Token>>, // 填充的 token
    pub prompt_tokens: u32, // 提示的 token 数量
    pub seed: u32, // 随机种子
    pub tokens: Vec<Token>, // 生成的 token
}

#[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
pub struct Token {
    pub id: u32, // token ID
    pub logprob: Option<f32>, // 概率对数
    pub special: Option<bool>, // 是否特殊 token
    pub text: Option<String>, // token 文本
}

// ==================================================== Generate Stream Response Struct ====================================================

#[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
pub struct GenerateStreamResponse {
    pub token: StreamToken,
    pub generated_text: Option<String>,
    pub details: Option<GenerateStreamDetails>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StreamToken {
    pub id: u32,
    pub text: String,
    pub logprob: Option<f32>,
    pub special: Option<bool>,
}

#[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
pub struct GenerateStreamDetails {
    pub prompt_tokens: u32,
    pub finish_reason: String,
    pub generated_tokens: u32,
    pub seed: u64,
}


// ==================================================== Generate Trait ====================================================
#[async_trait]
pub trait Generate: Send + Sync {
    async fn generate(&self, req_body: web::Json<GenerateRequest>, user_name: String, user_id: String, 
        service_name: String, service_id: String, user_type: String, user_level: String, pay_status: String, aicpid: String, is_stream:bool) -> Result<HttpResponse, Error>;
}