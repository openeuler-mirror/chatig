use utoipa::OpenApi;

use crate::apis::models_api;
use crate::apis::control_api;
//use crate::apis::funcs_api;
use crate::apis::control_api::schemas::Model;
use crate::apis::models_api::schemas::{ChatCompletionRequest,Message,EmbeddingRequest,EmbeddingResponse,EmbeddingData,Usage};
use crate::apis::control_api::models::{ModelErrorDetails,ModelErrorName};
use crate::cores::schemas::{CompletionsResponse,CompletionsChoice,CompletionsAssistantMessage,CompletionsUsage,CompletionsStreamResponse,CompletionsStreamChoice,CompletionsDelta};
use crate::apis::control_api::files::DeleteFileResponse;
use crate::apis::schemas::ErrorResponse;
use crate::meta::files::FileObject;


#[derive(OpenApi)]
#[openapi(
    paths(
        models_api::chat::health,
        //models_api::chat::completions,
        models_api::embeddings::v1_embeddings,
        control_api::models::models,
        control_api::models::model_info,
        control_api::models::delete_model,
        control_api::files::delete_file,
        control_api::files::list_file,
        control_api::files::get_file,
        control_api::files::get_file_content,
        //funcs_api::file_chat::file_chat,
        //funcs_api::rag::rag_chat_completions,
    ),
    components(
        schemas(Model,ChatCompletionRequest,Message,ErrorResponse,EmbeddingRequest,EmbeddingResponse,EmbeddingData,Usage,ModelErrorDetails,ModelErrorName,DeleteFileResponse,FileObject,CompletionsResponse,CompletionsChoice,CompletionsAssistantMessage,CompletionsUsage,CompletionsStreamResponse,CompletionsStreamChoice,CompletionsDelta)
    )
)]

pub struct ApiDoc;