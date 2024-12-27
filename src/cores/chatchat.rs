use actix_web::{web, HttpResponse};
use actix_multipart::Multipart;
use bytes::Bytes;
use reqwest::{Client, Response, multipart};
use serde_json::{Value, json};
use futures::stream::{StreamExt, TryStreamExt};    // For try_future and try_next

use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::utils::AppState;
use crate::apis::models_api::schemas::{ChatCompletionRequest, Message};

use crate::cores::schemas::{KbChatResponse, KbChatStreamResponse, OpenAIStreamResponse, UploadTempDocsResponse, 
    FileChatResponse, FileStreamChatResponse, FileDocStreamChatResponse, OpenAIDeltaMessage, OpenAIStreamChoice};

use crate::configs::settings::load_server_config;
use crate::meta::files::{add_file_object, FileObject};
                                      

// knowledge base chat completions
pub async fn kb_chat(req_body: web::Json<ChatCompletionRequest>) -> Result<HttpResponse, String> {
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
    let server_config = load_server_config().map_err(|err| format!("Failed to load server config: {}", err))?;
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
            Err(err) => return Err(format!("Request failed: {}", err)),
        };

    if stream {
        // Handle streaming response requests
        kb_response_stream(response).await
    } else {
        // handle non-streaming response requests
        kb_response_non_stream(response).await
    }
}

// Handle non-streaming response requests
async fn kb_response_non_stream(response: Response) -> Result<HttpResponse, String> {
    // 1. Parse the JSON response body into the KbChatResponse struct
    let response_text = response.text().await.map_err(|err| format!("Failed to read response: {}", err))?;

    // 2. Remove escape characters from the string
    let trimmed_text = response_text.trim_matches('"');
    let unescaped_text = trimmed_text.replace("\\\"", "\"").replace("\\\\", "\\");

    // 3. Convert unescaped_text to a JSON object
    let json_value: Value = serde_json::from_str(&unescaped_text)
    .map_err(|err| format!("Failed to parse unescaped JSON: {}, {}", err, unescaped_text))?;

    // 4. Convert the JSON object to a KbChatResponse struct
    let chat_response: KbChatResponse = match serde_json::from_value(json_value) {
        Ok(chat_response) => chat_response,
        Err(err) => return Err(format!("Failed to deserialize into KbChatResponse: {}", err)),
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
async fn kb_response_stream(response: Response) -> Result<HttpResponse, String> {
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
            .map_err(|err| format!("Failed to parse response as JSON: {}", err))?;
        let chat_response: OpenAIStreamResponse = serde_json::from_value(json_value)
            .map_err(|err| format!("Failed to deserialize into KbChatResponse: {}", err))?;

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

// Upload temporary documents
pub async fn upload_temp_docs(mut payload: Multipart, data: web::Data<AppState>) -> Result<HttpResponse, String> {
    let config = &data.config;
    let mut purpose: Option<String> = None;
    let mut filename: Option<String> = None;
    let mut total_file_content: Vec<u8> = Vec::new();
    let mut total_file_size: usize = 0;

    // Create multipart/form-data
    while let Ok(Some(mut field)) = payload.try_next().await {
        match field.name() {
            "purpose" => {
                // Read the purpose string
                let purpose_str = field.next().await.unwrap_or_else(|| Ok(Bytes::new())).unwrap_or_else(|_| Bytes::new());
                let purpose_str = String::from_utf8_lossy(&purpose_str).to_string();
                purpose = Some(purpose_str);
            }
            "file" => {
                // Read file content
                let mut file_size: usize = 0;
                let mut file_content: Vec<u8> = Vec::new();
                while let Some(chunk) = field.next().await {
                    let data = chunk.map_err(|err| err.to_string())?;
                    file_size += data.len();
                    file_content.extend_from_slice(&data);
                }
                total_file_size += file_size;
                total_file_content.extend_from_slice(&file_content);

                // Generate filename
                filename = field.content_disposition().get_filename().map(|name| name.to_string()).or(Some("uploaded_file".to_string()));

                // Check if the save directory exists, if not, create it
                let dir_path = Path::new(&config.temp_docs_path);
                if !dir_path.exists() {
                    std::fs::create_dir_all(dir_path).map_err(|err| format!("Failed to create directory: {} : {:?}", err, dir_path))?;
                }
                // Generate the file path
                let file_path = dir_path.join(filename.clone().unwrap_or_else(|| "uploaded_file".to_string()));
                let mut file = File::create(file_path.clone()).map_err(|err| format!("Failed to create file: {} : {:?}", err, file_path))?;
                file.write_all(&file_content).map_err(|err| format!("Failed to write to file: {}", err))?;

                // save file to pgsql
                let pool = &data.db_pool;
                let file_object = FileObject {
                    object: file_path.to_str().unwrap().to_string(),
                    bytes: file_size as i32,
                    created_at: chrono::Utc::now().timestamp() as i64,
                    filename: filename.clone().unwrap_or_else(|| "uploaded_file".to_string()),
                    purpose: purpose.clone().unwrap_or_else(|| "file_chat".to_string()),
                    id: 0,
                };
                add_file_object(pool, file_object).await.map_err(|err| format!("Failed to add file object: {}", err))?;

                // Print the successful storage file and file directory in the terminal
                println!("File saved successfully at: {:?}", file_path);
            }
            _ => (),
        }
    }

    // create multipart/form-data
    let form = multipart::Form::new()
        .part("files", 
                multipart::Part::bytes(total_file_content.clone())
                .file_name(filename.clone()
                .unwrap_or_else(|| "uploaded_file".to_string())))    // Use the filename from the upload request
        .text("prev_id", "")
        .text("chunk_size", "750")
        .text("chunk_overlap", "150")
        .text("zh_title_enhance", "false");

    let client = Client::new();
    let server_config = load_server_config().map_err(|err| format!("Failed to load server config: {}", err))?;
    let response = client.post(&server_config.chatchat.upload_temp_docs)
        .multipart(form)
        .send()
        .await
        .map_err(|err| format!("upload_temp_docs request failed: {}", err))?;

    // Convert the response to JSON UploadTempDocsResponse
    let response_text = response.text().await.map_err(|err| format!("Failed to read response text: {}", err))?;
    let res_json: UploadTempDocsResponse = serde_json::from_str(&response_text)
        .map_err(|err| format!("Failed to parse response as JSON: {}", err))?;

    // Check if there are failed files
    if !res_json.data.failed_files.is_empty() {
        let failed_files = res_json.data.failed_files;
        let mut error_message = String::new();
        for failed_file in failed_files {
            for (_key, value) in failed_file.details.iter() {
                error_message.push_str(&format!("{}\n", value));
            }
        }
        return Err(error_message);
    }

    // create response
    let res = json!({
        "id": res_json.data.id,
        "object": "file",
        "bytes": total_file_size,
        "created_at": chrono::Utc::now().timestamp() as u64,
        "filename": filename,
        "purpose": purpose,
    });

    Ok(HttpResponse::Ok().json(res))
}

pub async fn file_chat(req_body: web::Json<ChatCompletionRequest>) -> Result<HttpResponse, String> {
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
    let server_config = load_server_config().map_err(|err| format!("Failed to load server config: {}", err))?;
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
            Err(err) => return Err(format!("Request failed: {}", err)),
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

// Handle file non-streaming response requests
async fn file_response_non_stream(response: Response, model: String) -> Result<HttpResponse, String> {
    // 1. Parse the JSON response body into the FileChatResponse struct
    let response_text = response.text().await.map_err(|err| format!("Failed to read response: {}", err))?;

    // 2. remove unused words and changed to string
    let pre = &response_text[5..];
    let last = &pre[..pre.len() - 4];
    let value: Value = serde_json::from_str(last).unwrap();
    let new_json_str = serde_json::to_string_pretty(&value).unwrap();

    // 3. Convert new_json_str to a JSON object
    let json_value: Value = serde_json::from_str(&new_json_str)
    .map_err(|err| format!("Failed to parse unescaped JSON: {}, {}", err, new_json_str))?;

    // 4. Convert the JSON object to a FileChatResponse struct
    let chat_response: FileChatResponse = match serde_json::from_value(json_value) {
        Ok(chat_response) => chat_response,
        Err(err) => return Err(format!("Failed to deserialize into FileChatResponse: {}", err)),
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
async fn file_response_stream(response: Response, model: String) -> Result<HttpResponse, String> {
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