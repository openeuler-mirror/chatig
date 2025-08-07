use serde::{Serialize, Deserialize};
use std::fmt::{Debug, Display};

#[derive(Debug, Serialize, Deserialize)]
pub enum ApiError {
    NotFound(String),            // 资源未找到
    ValidationError(String),     // 验证错误
    DatabaseError(String),       // 数据库错误
    InternalServerError(String), // 内部服务器错误
    Unauthorized(String),        // 未授权
    BadRequest(String),          // 错误的请求
}

impl ApiError {
    pub fn message(&self) -> String {
        match *self {
            ApiError::NotFound(ref msg) => msg.clone(),
            ApiError::ValidationError(ref msg) => msg.clone(),
            ApiError::DatabaseError(ref msg) => msg.clone(),
            ApiError::InternalServerError(ref msg) => msg.clone(),
            ApiError::Unauthorized(ref msg) => msg.clone(),
            ApiError::BadRequest(ref msg) => msg.clone(),
        }
    }

    pub fn code(&self) -> i32 {
        match *self {
            ApiError::NotFound(_) => 404,
            ApiError::ValidationError(_) => 400,
            ApiError::DatabaseError(_) => 500,
            ApiError::InternalServerError(_) => 500,
            ApiError::Unauthorized(_) => 401,
            ApiError::BadRequest(_) => 400,
        }
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct ApiResponse<T> {
    code: i32,
    message: String,
    body: Option<T>,
}

impl<T: Debug> Display for ApiResponse<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ApiResponse {{ code: {}, message: {}, body: {:?} }}", self.code, self.message, self.body)
    }
}

impl<T> ApiResponse<T> {
    // 成功的响应
    pub fn success(message: &str, body: T) -> Self {
        ApiResponse {
            code: 200,
            message: message.to_string(),
            body: Some(body),
        }
    }

    // 错误的响应
    pub fn error(error: ApiError, body: Option<T>) -> Self {
        ApiResponse {
            code: error.code(),
            message: error.message(),
            body,
        }
    }
}

