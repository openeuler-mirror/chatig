use actix_web::{web, HttpResponse, Error};
use actix_web::error::ErrorInternalServerError;
use async_trait::async_trait;
use bytes::Bytes;
use reqwest::{Client, Response};
use serde_json::{Value, json};
use futures::stream::StreamExt;    // For try_future

use crate::apis::models_api::schemas::ChatCompletionRequest;
use crate::cores::schemas::{GetAnswerResponse, GetStreamAnswerResponse};
use crate::configs::settings::load_server_config;

use crate::cores::rag_apps::rag_controller::RAGController;

pub struct CopilotRAG;

#[async_trait]
impl RAGController for CopilotRAG {
    async fn rag_chat_completions(&self, req_body: web::Json<ChatCompletionRequest>) -> Result<HttpResponse, Error> {
        // 1. Find the content of the last message with role "user"
        let mut query = String::new();
        for message in req_body.messages.iter().rev() {
            if message.role == "user" {
                query = message.content.clone();
                break;
            }
        }

        // 2. Construct the request body for the chatchat API
        let request_body = json!({
            "question": query,
            "kb_sn": "default test",
            "session_id": "",
            "qa_record_id": "",
            "fetch_source": "true",
            "user_selected_plugins": [
                {
                    "plugin_name": "",
                    "plugin_auth": ""
                }
            ]
        });

        // Use reqwest to initiate a POST request
        let stream = req_body.stream.unwrap_or(false).clone();
        let server_config = load_server_config()
            .map_err(|err| ErrorInternalServerError(format!("Failed to load server config: {}", err)))?;
        let answer_url = if stream { &server_config.euler_copilot.get_stream_answer } else { &server_config.euler_copilot.get_answer };
        let model = req_body.model.clone();

        let client = Client::new();
        let response = match client.post(answer_url)
        .json(&request_body)
        .send()
        .await{
            Ok(resp) => resp, 
            Err(err) => return Err(ErrorInternalServerError(err.to_string())),
        };

        if stream {
            // Handle streaming response requests
            response_stream(response, model).await.map_err(Error::from)
        } else {
            // handle non-streaming response requests
            response_non_stream(response, model).await.map_err(Error::from)
        }
    }
}

// Handle non-streaming response requests
async fn response_non_stream(response: Response, model: String) -> Result<HttpResponse, Error> {
    // Parse the JSON response body into the GetAnswerResponse struct
    let response_json: Value = response.json().await
        .map_err(|err| ErrorInternalServerError(format!("Failed to parse response as JSON: {}", err)))?;
    let chat_response: GetAnswerResponse = serde_json::from_value(response_json)
        .map_err(|err| ErrorInternalServerError(format!("Failed to deserialize into GetAnswerResponse: {}", err)))?;

    // Create a timestamp
    let timestamp = chrono::Utc::now().timestamp() as u64;
    
    // 5. Return a custom response body
    let res = json!({
        "id": "eulercopilot-12345678",
        "object": "chat.completion",
        "created": timestamp,
        "model": model,
        "choices": [
            {
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": chat_response.answer
                },
                "finish_reason": "stop"
            }
        ],
        "usage": {
            "prompt_tokens": 0,
            "completion_tokens": 0,
            "total_tokens": 0
        }
    });

    Ok(HttpResponse::Ok().json(res))
}

// Handle streaming response requests
async fn response_stream(response: Response, model: String) -> Result<HttpResponse, Error> {
    // Get the byte stream of the response body, and skip the first chunk of data
    let mut body_stream = response.bytes_stream(); 
    // Create a custom conversation id
    let id = "eulercopilot-12345678";
    // Create a timestamp
    let timestamp = chrono::Utc::now().timestamp() as u64;

    // reate an asynchronous stream that sends each chunk of data obtained from the response to the client
    let model_clone = model.clone();
    let stream = async_stream::stream! {
        // let mut body_stream = body_stream;
        while let Some(chunk) = body_stream.next().await {
            match chunk {
                Ok(bytes) => {
                    // Convert bytes to JSON string
                    let json_str = String::from_utf8_lossy(&bytes);
                    //  Remove the prefix "data: " from the JSON string
                    let json_str = json_str.trim_start_matches("data: ");
                    
                    // Stop when the streaming data is "\n\n检索的到原始"
                    let stop_str = r#"{"content": "\n\n检索的到原始"}"#;
                    if json_str.trim() == stop_str {
                        break;
                    }

                    let json_value: Value = match serde_json::from_str(&json_str) {
                        Ok(json) => json,
                        Err(err) => {
                            yield Err(format!("Failed to parse response as JSON: {}", err));
                            continue;
                        }
                    };

                    // Try to convert json_value to GetStreamAnswerResponse
                    let chat_response: GetStreamAnswerResponse = match serde_json::from_value(json_value) {
                        Ok(chat_response) => chat_response,
                        Err(err) => {
                            yield Err(format!("Failed to deserialize into GetStreamAnswerResponse: {}", err));
                            continue;
                        }
                    };

                    // Create a custom response body
                    let res = json!({
                        "id": id,
                        "model": model_clone.clone(),
                        "choices": [
                            {
                                "index": 0,
                                "delta": {
                                    "content": chat_response.content,
                                },
                                "finish_reason": ""
                            }
                        ] 
                    });

                    // Convert res to String and add "data: " prefix
                    let res_str = format!("data: {}\n\n", serde_json::to_string(&res).unwrap());
                    yield Ok::<Bytes, String>(Bytes::from(res_str));
                },
                Err(err) => {
                    // If reading data fails, return an error
                    yield Err(format!("Stream read error: {}", err));
                }
            }
        }
    };

    // Convert the stream into a Vec for buffering
    let mut stream_iter = Box::pin(stream.fuse());
    // 去除stream_iter倒数的三个元素，不是前三个元素


    // Create the first response string for streaming data
    let res = json!({
        "id": id,
        "model": model.clone(),
        "choices": [
            {
                "index": 0,
                "delta": {
                    "role": "assistant"
                },
                "finish_reason": ""
            }
        ]
    });
    let first_str = format!("data: {}\n\n", serde_json::to_string(&res).unwrap());

    // Create the last response string for streaming data
    let res = json!({
        "id": id,
        "object": "chat.completion.chunk",
        "created": timestamp,
        "model": model.clone(),
        "choices": [
            {
                "index": 0,
                "delta": {},
                "finish_reason": "stop"
            }
        ]
    });
    let last_str = format!("data: {}\n\n", serde_json::to_string(&res).unwrap());

     // Create a new stream that combines the original stream and the response string
    let combined_stream = async_stream::stream! {
        // Yield first response string
        yield Ok::<Bytes, String>(Bytes::from(first_str));

        // Then yield the remaining data from the original stream
        while let Some(chunk) = stream_iter.next().await {
            yield chunk; // Yield remaining chunks
        }

        // Finally yield the response string
        yield Ok::<Bytes, String>(Bytes::from(last_str));

        let stop_str =  format!("data: [DONE]\n\n");
        yield Ok::<Bytes, String>(Bytes::from(stop_str));
    };

    // Return streaming response
    Ok(HttpResponse::Ok()
    .content_type("text/event-stream")
    .streaming(combined_stream))
}