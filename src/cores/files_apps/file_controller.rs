use actix_web::{web, Error, HttpResponse};
use actix_multipart::form::{json::Json as MpJson, tempfile::TempFile, MultipartForm};
use async_trait::async_trait;

use serde::Deserialize;

use crate::apis::models_api::schemas::ChatCompletionRequest;

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


// , payload: Multipart
#[async_trait]
pub trait FileChatController: Send + Sync {
    async fn upload_temp_docs(&self, MultipartForm(form): MultipartForm<UploadForm>) -> Result<HttpResponse, Error>;
    async fn file_chat_completions(&self, req_body: web::Json<ChatCompletionRequest>) -> Result<HttpResponse, Error>;
}