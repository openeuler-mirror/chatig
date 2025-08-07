use actix_web::{web, HttpResponse, Error};
use actix_web::error::{ErrorInternalServerError, ErrorBadRequest};
use async_trait::async_trait;
use crate::cores::control::services::ServiceManager;
use crate::cores::models::chat::chat_controller::{Completions, ChatCompletionRequest, Message, Content, ContentData};
use crate::cores::models::chat::chat_utils::{
    completions_response_stream,
    completions_response_non_stream,
    get_request_body,
    RequestInfo,
};
use crate::cores::utils::get_response;

pub struct StdChatModel {
    pub active_model: String,
}

#[async_trait]
impl Completions for StdChatModel {
    async fn completions(
        &self,
        req_body: web::Json<ChatCompletionRequest>,
        user_name: String,
        user_id: String,
        service_name: String,
        service_id: String,
        user_type: String,
        user_level: String,
        pay_status: String,
        aicpid: String,
    ) -> Result<HttpResponse, Error> {
        // 1. Read the model's parameter configuration
        let req_model_name = req_body.model.clone();

        let input_tokens = calculate_token_count(req_body.messages.clone());
        let service_manager = ServiceManager::default();
        let service_option = service_manager.get_service_by_model(&self.active_model, Some(input_tokens), aicpid).await?;
        let mut service = match service_option {
            Some(svc) => svc,
            None => return Err(ErrorBadRequest(format!("{} model is not supported", req_model_name))),
        };

        // 2. Build the request body
        let (request_body, is_stream, return_usage) = get_request_body(req_body, service.model_name.clone())
            .await
            .map_err(|err| ErrorInternalServerError(err.to_string()))?;

        // 3. Use reqwest to initiate a POST request
        let (response, start_time) = match get_response(request_body, &mut service, req_model_name.clone()).await {
            Ok((resp, start_time)) => (resp, start_time),
            Err(err) => return Err(ErrorInternalServerError(err.to_string())),
        };

        // 4. Return the response based on the request's streaming status
        let req_info = RequestInfo {
            req_model_name,
            user_name,
            user_id,
            service_name: service_name.clone(),
            service_id: service_id.clone(),
            start_time,
            return_usage,
            user_type: user_type.clone(),
            user_level: user_level.clone(),
            pay_status: pay_status.clone(),
            instance_id: service.id.clone(),
        };

        if is_stream {
            completions_response_stream(response, req_info).await
        } else {
            completions_response_non_stream(response, req_info).await
        }
    }
}

// This function calculates the total token count for a given set of messages
fn calculate_token_count(messages: Vec<Message>) -> f32 {
    let mut total_tokens: f32 = 0.0;

    for message in messages {
        match message.content {
            Content::Text(text) => {
                total_tokens += calculate_text_tokens(&text);
            }
            Content::Array(content_parts) => {
                for part in content_parts {
                    if part.r#type == "text" {
                        if let ContentData::Text { text } = part.data {
                            total_tokens += calculate_text_tokens(&text);
                        }
                    }
                }
            }
            Content::Assistant(assistant) => {
                if let Some(content) = assistant.content {
                    total_tokens += calculate_text_tokens(&content);
                }
            }
        }
    }

    total_tokens
}

// Helper function to calculate tokens for a single text string
fn calculate_text_tokens(text: &str) -> f32 {
    let mut tokens: f32 = 0.0;
    
    for c in text.chars() {
        if c.is_ascii() {
            // English character: 0.3 tokens
            tokens += 0.3;
        } else {
            // Chinese character (or other non-ASCII): 0.6 tokens
            tokens += 0.6;
        }
    }
    
    tokens
}