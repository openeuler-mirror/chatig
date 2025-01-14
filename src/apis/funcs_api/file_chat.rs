use actix_web::{post, web, Error, HttpResponse, Responder};
use actix_web::error::ErrorBadRequest;
use actix_multipart::form::MultipartForm;

use crate::apis::models_api::schemas::ChatCompletionRequest;
use crate::apis::schemas::ErrorResponse;
use crate::cores::files_apps::file_controller::FileChatController;
use crate::cores::files_apps::chatchat::ChatChatFile;
use crate::cores::files_apps::file_controller::UploadForm;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(file_chat);
}

// define an interface layer that calls the completions method of the large model
struct FileChatModel {
    model: Box<dyn FileChatController>,
}

impl FileChatModel {
    fn new(model: Box<dyn FileChatController>) -> Self {
        FileChatModel { model }
    }
    async fn upload_temp_docs(&self, MultipartForm(form): MultipartForm<UploadForm>) -> Result<HttpResponse, Error>{
        self.model.upload_temp_docs(MultipartForm(form)).await
    }
    async fn file_chat_completions(&self, req_body: web::Json<ChatCompletionRequest>) -> Result<HttpResponse, Error> {
        self.model.file_chat_completions(req_body).await
    }
}

// post https://***/v1/files
#[post("v1/files")]
pub async fn upload_file(MultipartForm(form): MultipartForm<UploadForm>) -> Result<impl Responder, Error> {
    let file_chat_model = FileChatModel::new(Box::new(ChatChatFile {}));

    // 2. Call the underlying API and return a unified data format
    let response = file_chat_model.upload_temp_docs(MultipartForm(form)).await;

    // 3. Construct the response body based on the API's return result
    match response {
        Ok(resp) => Ok(resp),
        Err(err) => {
            let error_response = ErrorResponse {
                error: format!("Failed to get response from upload_temp_docs: {}", err),
            };
            Ok(HttpResponse::InternalServerError().json(error_response))
        }
    }
}

#[utoipa::path(
    post,  // 请求方法
    path = "/v1/file/completions",  // 路径
    request_body = ChatCompletionRequest, //有问题
    responses(
        (status = 200, body = String), //还没写完
        (status = 400, body = ErrorResponse),
        (status = 500, body = ErrorResponse),
    )  // 响应内容
)]

#[post("/v1/file/completions")]
pub async fn file_chat(req_body: web::Json<ChatCompletionRequest>) -> Result<impl Responder, Error> {
    // 1. Validate that required fields exist in the request data
    if req_body.model.is_empty() || req_body.messages.is_empty() {
        let error_response = ErrorResponse {
            error: "Invalid request: model or messages cannot be empty.".into(),
        };
        return Ok(HttpResponse::BadRequest().json(error_response));
    }

    // 2. Call the underlying API and return a unified data format
    let model_name = req_body.model.clone();
    let file_chat_model: FileChatModel = match model_name.as_str() {
        "chatchat" => FileChatModel::new(Box::new(ChatChatFile {})),
        _ => return Err(ErrorBadRequest(format!("Unsupported {} model series!", model_name))),
    };

    // 3. Send the request to the model service
    let response = file_chat_model.file_chat_completions(req_body).await;
    match response {
        Ok(resp) => Ok(resp),
        Err(err) => {
            let error_response = ErrorResponse {
                error: format!("Failed to get response from {} chat completions: {}", model_name, err),
            };
            Ok(HttpResponse::InternalServerError().json(error_response))
        }
    }  
}