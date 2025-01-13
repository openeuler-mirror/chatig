use utoipa::OpenApi;

use crate::apis::models_api;
use crate::apis::control_api;
//use crate::apis::funcs_api;
use crate::apis::control_api::schemas::Model;
use crate::apis::models_api::schemas::{ChatCompletionRequest,Message,EmbeddingRequest,EmbeddingResponse,EmbeddingData,Usage};
use crate::apis::control_api::models::{ModelErrorDetails,ModelErrorName};
use crate::apis::schemas::ErrorResponse;


#[derive(OpenApi)]
#[openapi(
    paths(
        models_api::chat::health,
        models_api::chat::completions,
        models_api::embeddings::v1_embeddings,
        control_api::models::models,
        control_api::models::model_info,
        control_api::models::delete_model,
    ),
    components(
        schemas(Model,ChatCompletionRequest,Message,ErrorResponse,EmbeddingRequest,EmbeddingResponse,EmbeddingData,Usage,ModelErrorDetails,ModelErrorName)
    )
)]

pub struct ApiDoc;