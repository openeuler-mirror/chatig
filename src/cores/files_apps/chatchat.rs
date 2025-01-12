use actix_web::{web, HttpResponse, Error};
use actix_web::error::ErrorInternalServerError;
use async_trait::async_trait;
use actix_multipart::form::MultipartForm;
use bytes::Bytes;
use reqwest::{Client, Response, multipart};
use serde_json::{Value, json};
use futures::stream::StreamExt;    // For try_future and try_next

use std::fs;
use std::path::Path;

use crate::apis::models_api::schemas::{ChatCompletionRequest, Message};
use crate::cores::schemas::{OpenAIStreamResponse, UploadTempDocsResponse, 
    FileChatResponse, FileStreamChatResponse, FileDocStreamChatResponse, OpenAIDeltaMessage, OpenAIStreamChoice};
use crate::cores::files_apps::file_controller::FileChatController;
use crate::cores::files_apps::file_controller::UploadForm;

use crate::configs::settings::GLOBAL_CONFIG;
use crate::configs::settings::load_server_config;
use crate::meta::files::{add_file_object, FileObject};


pub struct ChatChatFile;

#[async_trait]
impl FileChatController for ChatChatFile {
    // Upload temporary documents
    async fn upload_temp_docs(&self, MultipartForm(form): MultipartForm<UploadForm>) -> Result<HttpResponse, Error> {
        let config = &*GLOBAL_CONFIG;

        // save uploaded files
        let purpose = form.json.purpose.clone();
        let upload_dir = Path::new(&config.temp_docs_path);
        if !upload_dir.exists() {
            fs::create_dir_all(upload_dir)
                .map_err(|err| ErrorInternalServerError(format!("Failed to create directory: {} : {:?}", err, upload_dir)))?;
        }

        // create multipart/form-data
        let mut req_form = multipart::Form::new()
            .text("prev_id", "")
            .text("chunk_size", "750")
            .text("chunk_overlap", "150")
            .text("zh_title_enhance", "false");

        let mut total_file_size: u64 = 0;
        let mut file_name = String::new();
        for file in form.files {
            let part = multipart::Part::stream(fs::read(&file.file.path())?)
                .file_name(file.file_name.clone().unwrap_or_else(|| "unnamed".to_string()));
            req_form = req_form.part("files", part);
            
            // 
            total_file_size += file.size as u64;
    
            // move the temporary file to the target directory
            file_name = file.file_name.clone().unwrap_or_else(|| "unnamed".to_string());
            let file_path = upload_dir.join(&file_name);
            fs::rename(&file.file.path(), &file_path)?;
            
            // save file to pgsql
            let file_object = FileObject {
                object: file_path.to_str().unwrap().to_string(),
                bytes: file.size as i32,
                created_at: chrono::Utc::now().timestamp() as i64,
                filename: file_name.clone(),
                purpose: purpose.clone(),
                id: 0,
            };
            add_file_object(file_object).await
                .map_err(|err| ErrorInternalServerError(format!("Failed to add file object: {}", err)))?;
        }
        
        let client = Client::new();
        let server_config = load_server_config()
            .map_err(|err| ErrorInternalServerError(format!("Failed to load server config: {}", err)))?;
        
        let response = client.post(&server_config.chatchat.upload_temp_docs)
            .multipart(req_form)
            .send()
            .await
            .map_err(|err| ErrorInternalServerError(format!("upload_temp_docs request failed: {}", err)))?;
    
        // Convert the response to JSON UploadTempDocsResponse
        let response_text = response.text().await
            .map_err(|err| ErrorInternalServerError(format!("Failed to read response text: {}", err)))?;
        let res_json: UploadTempDocsResponse = serde_json::from_str(&response_text)
            .map_err(|err| ErrorInternalServerError(format!("Failed to parse response as JSON: {}", err)))?;
    
        // Check if there are failed files
        if !res_json.data.failed_files.is_empty() {
            let failed_files = res_json.data.failed_files;
            let mut error_message = String::new();
            for failed_file in failed_files {
                for (_key, value) in failed_file.details.iter() {
                    error_message.push_str(&format!("{}\n", value));
                }
            }
            return Err(ErrorInternalServerError(error_message));
        }
    
        // create response
        let res = json!({
            "id": res_json.data.id,
            "object": "file",
            "bytes": total_file_size,
            "created_at": chrono::Utc::now().timestamp() as u64,
            "filename": file_name,
            "purpose": purpose,
        });
    
        Ok(HttpResponse::Ok().json(res))
    }

    // Upload temporary documents
    async fn file_chat_completions(&self, req_body: web::Json<ChatCompletionRequest>) -> Result<HttpResponse, Error> {
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
        let mut history: Vec<Message> = vec![
            Message {
                role: "system".to_string(),
                content: "现在你是一名及其专业的计算机专家，工作是一名操作系统的运维助手，这份工作极其重要，不能出错！".to_string(),
            }
        ];
        history.extend_from_slice(&req_body.messages[..req_body.messages.len() - 1]);

        // 2. Construct the request body for the chatchat API
        let server_config = load_server_config()
            .map_err(|err| ErrorInternalServerError(format!("Failed to load server config: {}", err)))?;
        let request_body = json!({
            "query": query,
            "knowledge_id": req_body.file_id,
            "top_k": 3,
            "score_threshold": 2,
            "history": history,
            "stream": stream,
            "model_name": &server_config.chatchat.model_name,
            "temperature": temperature,
            "max_tokens": max_tokens,
            "prompt_name": "default"
        });

        // Use reqwest to initiate a POST request
        let client = Client::new();
        let response = match client.post(&server_config.chatchat.file_chat)
            .json(&request_body)
            .send()
            .await{
                Ok(resp) => resp, 
                Err(err) => return Err(ErrorInternalServerError(format!("Request failed: {}", err))),
        };

        // get model
        let model = &req_body.model;
        if stream {
            // Handle streaming response requests
            file_response_stream(response, model.to_string()).await
        } else {
            // handle non-streaming response requests
            file_response_non_stream(response, model.to_string()).await
        }   
    }

}

// Handle file non-streaming response requests
async fn file_response_non_stream(response: Response, model: String) -> Result<HttpResponse, Error> {
    // 1. Parse the JSON response body into the FileChatResponse struct
    let response_text = response.text().await
        .map_err(|err| ErrorInternalServerError(format!("Failed to read response: {}", err)))?;

    // 2. remove unused words and changed to string
    let pre = &response_text[5..];
    let last = &pre[..pre.len() - 4];
    let value: Value = serde_json::from_str(last).unwrap();
    let new_json_str = serde_json::to_string_pretty(&value).unwrap();

    // 3. Convert new_json_str to a JSON object
    let json_value: Value = serde_json::from_str(&new_json_str)
        .map_err(|err| ErrorInternalServerError(format!("Failed to parse unescaped JSON: {}, {}", err, new_json_str)))?;

    // 4. Convert the JSON object to a FileChatResponse struct
    let chat_response: FileChatResponse = match serde_json::from_value(json_value) {
        Ok(chat_response) => chat_response,
        Err(err) => return Err(ErrorInternalServerError(format!("Failed to deserialize into FileChatResponse: {}", err))),
    };

    // Create a timestamp
    let timestamp = chrono::Utc::now().timestamp() as u64;

    // 5. Return a custom response body
    let res = json!({
        "id": "chatchat-12345678",
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

// Handle file streaming response requests
async fn file_response_stream(response: Response, model: String) -> Result<HttpResponse, Error> {
    // Get the byte stream of the response body
    let mut body_stream = response.bytes_stream(); 

    // reate an asynchronous stream that sends each chunk of data obtained from the response to the client
    let stream = async_stream::stream! {
        while let Some(chunk) = body_stream.next().await {
            match chunk {
                Ok(bytes) => {
                    // Convert bytes to JSON string
                    let json_str = String::from_utf8_lossy(&bytes);
                    //  Remove the prefix "data: " from the JSON string
                    let json_str = json_str.trim_start_matches("data: ");
                    let json_value: Value = match serde_json::from_str(&json_str) {
                        Ok(json) => json,
                        Err(err) => {
                            yield Err(format!("Failed to parse response as JSON: {}", err));
                            continue;
                        }
                    };
                    if let Some(_answer_obj) = json_value.get("answer") {
                        // Try to convert json_value to FileStreamChatResponse
                        let chat_response: FileStreamChatResponse = match serde_json::from_value(json_value) {
                            Ok(chat_response) => chat_response,
                            Err(err) => {
                                yield Err(format!("Failed to deserialize into FileStreamChatResponse: {}", err));
                                continue;
                            }
                        };

                        // Create a custom response body
                        let res = json!({
                            "answer": chat_response.answer
                        });

                        // Convert res to String and add "data: " prefix
                        let res_str = format!("data: {}\n\n", serde_json::to_string(&res).unwrap());

                        yield Ok::<Bytes, String>(Bytes::from(res_str));
                    } else {
                        // Try to convert json_value to FileDocStreamChatResponse
                        let chat_response: FileDocStreamChatResponse = match serde_json::from_value(json_value) {
                            Ok(chat_response) => chat_response,
                            Err(err) => {
                                yield Err(format!("Failed to deserialize into FileDocStreamChatResponse: {}", err));
                                continue;
                            }
                        };

                        // Create a custom response body
                        let res = json!({
                            "docs": chat_response.docs
                        });

                        // Convert res to String and add "data: " prefix
                        let res_str = format!("data: {}\n\n", serde_json::to_string(&res).unwrap());

                        yield Ok::<Bytes, String>(Bytes::from(res_str));

                    }
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

    // Create a timestamp
    let timestamp = chrono::Utc::now().timestamp() as u64;

    // Create the first response string for streaming data
    let res = json!({
        "id": "chatchat-12345678",
        "model": model,
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
        "id": "chatchat-12345678",
        "object": "chat.completion.chunk",
        "created": timestamp,
        "model": model,
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
        // Yield buffered data first
        for chunk in buffered_stream {
            if let Ok(chunk_bytes) = chunk {
                // Create OpenAIStreamResponse here for each buffered chunk
                let json_str = String::from_utf8_lossy(&chunk_bytes);
                let json_str = json_str.trim_start_matches("data: ");
                let json_value: Value = serde_json::from_str(&json_str).unwrap();
                if let Some(answer_obj) = json_value.get("answer") {
                    if let Value::String(answer_value) = answer_obj {
                        let delta_message = OpenAIDeltaMessage {
                            content: answer_value.clone(),
                        };
                        let choice = OpenAIStreamChoice {
                            index: 0,
                            delta: delta_message,
                            finish_reason: "".to_string(),
                        };
                        let openai_response = OpenAIStreamResponse {
                            id: "chatchat-12345678".to_string(),
                            model: model.clone(),
                            choices: vec![choice],
                        };
                        let openai_response_str = serde_json::to_string(&openai_response).unwrap();
                        yield Ok::<Bytes, String>(Bytes::from(format!("data: {}\n\n", openai_response_str)));
                    }
                }
            }
        }

        // Then yield the remaining data from the original stream
        while let Some(chunk) = stream_iter.next().await {
            if let Ok(chunk_bytes) = chunk {
                // Create OpenAIStreamResponse here for each remaining chunk
                let json_str = String::from_utf8_lossy(&chunk_bytes);
                let json_str = json_str.trim_start_matches("data: ");
                let json_value: Value = serde_json::from_str(&json_str).unwrap();
                if let Some(answer_obj) = json_value.get("answer") {
                    if let Value::String(answer_value) = answer_obj {
                        let delta_message = OpenAIDeltaMessage {
                            content: answer_value.clone(),
                        };
                        let choice = OpenAIStreamChoice {
                            index: 0,
                            delta: delta_message,
                            finish_reason: "".to_string(),
                        };
                        let openai_response = OpenAIStreamResponse {
                            id: "chatchat-12345678".to_string(),
                            model: model.clone(),
                            choices: vec![choice],
                        };
                        let openai_response_str = serde_json::to_string(&openai_response).unwrap();
                        yield Ok::<Bytes, String>(Bytes::from(format!("data: {}\n\n", openai_response_str)));
                    }
                }
            }
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