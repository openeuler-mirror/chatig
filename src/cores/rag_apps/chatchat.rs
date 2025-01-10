use actix_web::{web, HttpResponse, Error};
use actix_web::error::ErrorInternalServerError;
use async_trait::async_trait;
use bytes::Bytes;
use reqwest::{Client, Response};
use serde_json::{Value, json};
use futures::stream::StreamExt;    // For try_future and try_next

use crate::apis::models_api::schemas::{ChatCompletionRequest, Message};

use crate::cores::schemas::{KbChatResponse, KbChatStreamResponse, OpenAIStreamResponse};
use crate::cores::rag_apps::rag_controller::RAGController;

use crate::configs::settings::load_server_config;
                                      
pub struct ChatChatRAG;

#[async_trait]
impl RAGController for ChatChatRAG {
    async fn rag_chat_completions(&self, req_body: web::Json<ChatCompletionRequest>) -> Result<HttpResponse, Error> {
        // 1. Get the corresponding parameter values, if not provided in the request body, set default values
        let max_tokens = req_body.max_tokens.unwrap_or(0).clone();
        let temperature = req_body.temperature.unwrap_or(0.3).clone();
        let stream = req_body.stream.unwrap_or(false).clone();

        // Find the content of the last message with role "user"
        let mut query = String::new();
        for message in req_body.messages.iter().rev() {
            if message.role == "user" {
                query = message.content.clone();
                break;
            }
        }

        // Construct history messages
        let mut history: Vec<Message> = vec![];
        if req_body.messages[0].role != "system" {
            let message = Message {
                role: "system".to_string(),
                content: "现在你是一名专业的计算机专家，工作是一名操作系统的运维助手，负责确保系统的稳定运行和用户的满意。请提供准确的信息。".to_string(),
            };
            history.push(message);
        }

        history.extend_from_slice(&req_body.messages[..req_body.messages.len() - 1]);

        // 2. Construct the request body for the chatchat API
        let server_config = load_server_config()
            .map_err(|err| ErrorInternalServerError(format!("Failed to load server config: {}", err)))?;
        let request_body = json!({
            "query": query,
            "mode": "local_kb",
            "kb_name": "CULinux",
            "top_k": 3,
            "score_threshold": 2,
            "history": history,
            "stream": stream,
            "model": &server_config.chatchat.model_name,
            "temperature": temperature,
            "max_tokens": max_tokens,
            "prompt_name": "default",
            "return_direct": false
        });

        // Use reqwest to initiate a POST request
        let client = Client::new();
        let response = match client.post(&server_config.chatchat.kb_chat)
            .json(&request_body)
            .send()
            .await{
                Ok(resp) => resp, 
                Err(err) => return Err(ErrorInternalServerError(format!("Request failed: {}", err))),
            };

        if stream {
            // Handle streaming response requests
            kb_response_stream(response).await
        } else {
            // handle non-streaming response requests
            kb_response_non_stream(response).await
        }
    }
 
}

// Handle non-streaming response requests
async fn kb_response_non_stream(response: Response) -> Result<HttpResponse, Error> {
    // 1. Parse the JSON response body into the KbChatResponse struct
    let response_text = response.text().await
        .map_err(|err| ErrorInternalServerError(format!("Failed to read response: {}", err)))?;

    // 2. Remove escape characters from the string
    let trimmed_text = response_text.trim_matches('"');
    let unescaped_text = trimmed_text.replace("\\\"", "\"").replace("\\\\", "\\");

    // 3. Convert unescaped_text to a JSON object
    let json_value: Value = serde_json::from_str(&unescaped_text)
        .map_err(|err| ErrorInternalServerError(format!("Failed to parse unescaped JSON: {}, {}", err, unescaped_text)))?;

    // 4. Convert the JSON object to a KbChatResponse struct
    let chat_response: KbChatResponse = match serde_json::from_value(json_value) {
        Ok(chat_response) => chat_response,
        Err(err) => return Err(ErrorInternalServerError(format!("Failed to deserialize into KbChatResponse: {}", err))),
    };
    
    // 5. Return a custom response body
    let res = json!({
      "id": chat_response.id,
      "object": chat_response.object,
      "created": chat_response.created,
      "model": chat_response.model,
      "choices": [
          {
              "index": 0,
              "message": {
                  "role": "assistant",
                  "content": chat_response.choices[0].message.content
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
async fn kb_response_stream(response: Response) -> Result<HttpResponse, Error> {
    // Get the byte stream of the response body, and skip the first chunk of data
    let mut body_stream = response.bytes_stream().skip(1); 

    // reate an asynchronous stream that sends each chunk of data obtained from the response to the client
    let stream = async_stream::stream! {
        // let mut body_stream = body_stream;
        while let Some(chunk) = body_stream.next().await {
            match chunk {
                Ok(bytes) => {
                    // Convert bytes to JSON string
                    let json_str = String::from_utf8_lossy(&bytes);
                    //  Remove the prefix "data: " from the JSON string
                    let json_str = json_str.trim_start_matches("data: ");

                    let json_value: Value = match serde_json::from_str(&json_str) {
                        Ok(json) => json,
                        Err(_err) => {
                            // yield Err(format!("Failed to parse response as JSON: {}", err));
                            continue;
                        }
                    };

                    // Try to convert json_value to KbChatResponse
                    let chat_response: KbChatStreamResponse = match serde_json::from_value(json_value) {
                        Ok(chat_response) => chat_response,
                        Err(_err) => {
                            // yield Err(format!("Failed to deserialize into KbChatResponse: {}", err));
                            continue;
                        }
                    };

                    // Create a custom response body
                    let res = json!({
                        "id": chat_response.id,
                        "model": chat_response.model,
                        "choices": [
                            {
                                "index": 0,
                                "delta": {
                                    "content": chat_response.choices[0].delta.content
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
    let mut buffered_stream: Vec<Result<Bytes, String>> = Vec::new();

    // Preview the first stream data without consuming it
    if let Some(chunk) = stream_iter.as_mut().next().await {
        buffered_stream.push(chunk);
    }

    let mut first_str = String::new();
    let mut last_str = String::new();
    if let Some(Ok(bytes)) = buffered_stream.get(0) {
        // Process the first chunk (this will be a reference to the actual data)
        let json_str = String::from_utf8_lossy(&bytes);
        //  Remove the prefix "data: " from the JSON string
        let json_str = json_str.trim_start_matches("data: ");

        // Deserialize the JSON string into OpenAIDeltaMessage
        let json_value: Value = serde_json::from_str(&json_str)
            .map_err(|err| ErrorInternalServerError(format!("Failed to parse response as JSON: {}", err)))?;
        let chat_response: OpenAIStreamResponse = serde_json::from_value(json_value)
            .map_err(|err| ErrorInternalServerError(format!("Failed to deserialize into KbChatResponse: {}", err)))?;

        // Create a timestamp
        let timestamp = chrono::Utc::now().timestamp() as u64;

        // Create the first response string for streaming data
        let res = json!({
            "id": chat_response.id,
            "model": chat_response.model,
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
        first_str = format!("data: {}\n\n", serde_json::to_string(&res).unwrap());

        // Create the last response string for streaming data
        let res = json!({
            "id": chat_response.id,
            "object": "chat.completion.chunk",
            "created": timestamp,
            "model": chat_response.model,
            "choices": [
                {
                    "index": 0,
                    "delta": {},
                    "finish_reason": "stop"
                }
            ]
        });
        last_str = format!("data: {}\n\n", serde_json::to_string(&res).unwrap());
    }

    // Create a new stream that combines the original stream and the response string
    let combined_stream = async_stream::stream! {
        // Yield first response string
        yield Ok::<Bytes, String>(Bytes::from(first_str));

        // Yield buffered data first
        for chunk in buffered_stream {
            yield chunk; // Yield buffered chunk
        }

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

