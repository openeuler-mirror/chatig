use utoipa::OpenApi;

use crate::apis::models_api;
use crate::apis::control_api;
use crate::meta::models::Model;
use crate::apis::models_api::schemas::{EmbeddingRequest, EmbeddingResponse, EmbeddingData};
use crate::cores::chat_models::chat_controller::{ChatCompletionRequest, Message,Usage};
use crate::apis::control_api::models::{ModelErrorDetails, ModelErrorName};
use crate::cores::chat_models::chat_controller::{CompletionsResponse, CompletionsChoice, CompletionsAssistantMessage, 
    CompletionsUsage, CompletionsStreamResponse, CompletionsStreamChoice, CompletionsDelta};
use crate::apis::schemas::ErrorResponse;
use crate::meta::files::traits::File;


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
        control_api::files::get_all_files,
        control_api::files::get_file,
        //funcs_api::file_chat::file_chat,
        //funcs_api::rag::rag_chat_completions,
    ),
    components(
        schemas(Model, ChatCompletionRequest, Message, ErrorResponse, EmbeddingRequest, EmbeddingResponse, 
            EmbeddingData, Usage, ModelErrorDetails, ModelErrorName, File, CompletionsResponse, CompletionsChoice,
            CompletionsAssistantMessage, CompletionsUsage, CompletionsStreamResponse, CompletionsStreamChoice, CompletionsDelta)
    )
)]

pub struct ApiDoc;