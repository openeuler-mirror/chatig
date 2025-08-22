"""
ChatIG Chat Module 异常类定义
对应Rust代码中的错误处理
"""

from typing import Optional, Dict, Any


class ChatIGError(Exception):
    """ChatIG基础异常类"""
    
    def __init__(self, message: str, status_code: int = None, error_type: str = None):
        self.message = message
        self.status_code = status_code
        self.error_type = error_type or "ChatIGError"
        super().__init__(self.message)
    
    def __str__(self):
        if self.status_code:
            return f"{self.error_type}({self.status_code}): {self.message}"
        return f"{self.error_type}: {self.message}"
    
    def to_dict(self) -> Dict[str, Any]:
        """转换为字典格式，对应Rust中的ErrorResponse"""
        return {
            "error": self.message,
            "status_code": self.status_code,
            "error_type": self.error_type
        }


class AuthenticationError(ChatIGError):
    """认证错误 - 对应401错误"""
    
    def __init__(self, message: str = "Authentication failed", status_code: int = 401):
        super().__init__(message, status_code, "AuthenticationError")


class RateLimitError(ChatIGError):
    """速率限制错误 - 对应429错误"""
    
    def __init__(self, message: str = "Rate limit exceeded", status_code: int = 429):
        super().__init__(message, status_code, "RateLimitError")


class ServerError(ChatIGError):
    """服务器错误 - 对应500错误"""
    
    def __init__(self, message: str = "Server error", status_code: int = 500):
        super().__init__(message, status_code, "ServerError")


class ValidationError(ChatIGError):
    """数据验证错误 - 对应400错误"""
    
    def __init__(self, message: str = "Validation error", status_code: int = 400):
        super().__init__(message, status_code, "ValidationError")


class ModelNotFoundError(ChatIGError):
    """模型未找到错误 - 对应404错误"""
    
    def __init__(self, model_name: str, status_code: int = 404):
        message = f"Model '{model_name}' not found or not supported"
        super().__init__(message, status_code, "ModelNotFoundError")


class ModelSeriesNotSupportedError(ChatIGError):
    """模型系列不支持错误"""
    
    def __init__(self, series: str, status_code: int = 400):
        message = f"Unsupported {series} model series!"
        super().__init__(message, status_code, "ModelSeriesNotSupportedError")


class InvalidModelFormatError(ChatIGError):
    """模型格式错误"""
    
    def __init__(self, model_name: str, status_code: int = 400):
        message = f"Invalid model format: {model_name}. Expected format: 'series/model-name'"
        super().__init__(message, status_code, "InvalidModelFormatError")


class StreamError(ChatIGError):
    """流式处理错误"""
    
    def __init__(self, message: str = "Stream processing error"):
        super().__init__(message, error_type="StreamError")


class TimeoutError(ChatIGError):
    """超时错误 - 对应408错误"""
    
    def __init__(self, message: str = "Request timeout", status_code: int = 408):
        super().__init__(message, status_code, "TimeoutError")


class NetworkError(ChatIGError):
    """网络连接错误"""
    
    def __init__(self, message: str = "Network connection error"):
        super().__init__(message, error_type="NetworkError")


class APIError(ChatIGError):
    """API调用错误"""
    
    def __init__(self, message: str, status_code: int = None):
        super().__init__(message, status_code, "APIError")


class ConfigurationError(ChatIGError):
    """配置错误"""
    
    def __init__(self, message: str):
        super().__init__(message, error_type="ConfigurationError")


# 错误映射函数
def map_http_status_to_exception(status_code: int, message: str = None) -> ChatIGError:
    """根据HTTP状态码映射到对应的异常类"""
    if status_code == 400:
        return ValidationError(message or "Bad request")
    elif status_code == 401:
        return AuthenticationError(message or "Authentication failed")
    elif status_code == 404:
        return ModelNotFoundError(message or "Resource not found")
    elif status_code == 408:
        return TimeoutError(message or "Request timeout")
    elif status_code == 429:
        return RateLimitError(message or "Rate limit exceeded")
    elif status_code >= 500:
        return ServerError(message or "Server error")
    else:
        return APIError(message or f"API error with status {status_code}", status_code)


# 错误处理工具函数
def handle_api_error(response_data: Dict[str, Any], status_code: int = None) -> ChatIGError:
    """处理API错误响应"""
    error_message = response_data.get("error", "Unknown error")
    
    # 如果响应中包含特定的错误类型，优先使用
    if "error_type" in response_data:
        error_type = response_data["error_type"]
        if error_type == "ModelSeriesNotSupportedError":
            return ModelSeriesNotSupportedError(error_message)
        elif error_type == "InvalidModelFormatError":
            return InvalidModelFormatError(error_message)
        elif error_type == "ModelNotFoundError":
            return ModelNotFoundError(error_message)
    
    # 否则根据状态码映射
    return map_http_status_to_exception(status_code or 500, error_message)


def validate_model_format(model_name: str) -> None:
    """验证模型格式"""
    if not model_name:
        raise ValidationError("Model name cannot be empty")
    
    if '/' not in model_name:
        raise InvalidModelFormatError(model_name)
    
    parts = model_name.split('/')
    if len(parts) != 2:
        raise InvalidModelFormatError(model_name)
    
    series, model = parts
    if not series or not model:
        raise InvalidModelFormatError(model_name)


def validate_messages(messages: list) -> None:
    """验证消息列表"""
    if not messages:
        raise ValidationError("Messages cannot be empty")
    
    for i, msg in enumerate(messages):
        if not isinstance(msg, dict):
            raise ValidationError(f"Message {i} must be a dictionary")
        
        if 'role' not in msg:
            raise ValidationError(f"Message {i} must have 'role' field")
        
        if 'content' not in msg:
            raise ValidationError(f"Message {i} must have 'content' field")
        
        role = msg['role']
        if role not in ['system', 'user', 'assistant']:
            raise ValidationError(f"Message {i} has invalid role: {role}") 