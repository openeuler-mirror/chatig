"""
Chatig 重排序 SDK 的异常类

本模块定义了重排序SDK中使用的自定义异常。
"""

from typing import Optional, Dict, Any


class RerankError(Exception):
    """所有重排序相关错误的基础异常"""
    
    def __init__(self, message: str, status_code: Optional[int] = None, 
                 response_data: Optional[Dict[str, Any]] = None):
        super().__init__(message)
        self.message = message
        self.status_code = status_code
        self.response_data = response_data or {}
    
    def __str__(self) -> str:
        if self.status_code:
            return f"RerankError (HTTP {self.status_code}): {self.message}"
        return f"RerankError: {self.message}"


class RerankValidationError(RerankError):
    """当请求验证失败时抛出"""
    
    def __init__(self, message: str, field: Optional[str] = None):
        super().__init__(f"Validation error{f' in field {field}' if field else ''}: {message}")
        self.field = field


class RerankConnectionError(RerankError):
    """当连接到重排序服务失败时抛出"""
    
    def __init__(self, message: str, url: Optional[str] = None):
        super().__init__(f"Connection error{f' to {url}' if url else ''}: {message}")
        self.url = url


class RerankTimeoutError(RerankError):
    """当重排序请求超时时抛出"""
    
    def __init__(self, message: str, timeout: Optional[float] = None):
        super().__init__(f"Timeout error{f' after {timeout} seconds' if timeout else ''}: {message}")
        self.timeout = timeout


class RerankAuthenticationError(RerankError):
    """当认证失败时抛出"""
    
    def __init__(self, message: str = "Authentication failed"):
        super().__init__(message)


class RerankRateLimitError(RerankError):
    """当超过速率限制时抛出"""
    
    def __init__(self, message: str = "Rate limit exceeded", 
                 retry_after: Optional[int] = None):
        super().__init__(message)
        self.retry_after = retry_after


class RerankModelNotFoundError(RerankError):
    """当指定的模型未找到时抛出"""
    
    def __init__(self, model: str):
        super().__init__(f"Model not found '{model}'")
        self.model = model


class RerankEngineNotSupportedError(RerankError):
    """当指定的引擎不支持时抛出"""
    
    def __init__(self, engine: str):
        super().__init__(f"Unsupported engine '{engine}'")
        self.engine = engine


class RerankResponseParseError(RerankError):
    """当响应解析失败时抛出"""
    
    def __init__(self, message: str, response_text: Optional[str] = None):
        super().__init__(f"Response parsing error: {message}")
        self.response_text = response_text


class RerankServiceUnavailableError(RerankError):
    """当重排序服务不可用时抛出"""
    
    def __init__(self, message: str = "Service unavailable"):
        super().__init__(message)


class RerankQuotaExceededError(RerankError):
    """当超过配额时抛出"""
    
    def __init__(self, message: str = "Quota exceeded"):
        super().__init__(message)


def handle_http_error(status_code: int, response_data: Dict[str, Any]) -> RerankError:
    """
    将HTTP错误状态码转换为相应的异常
    
    参数:
        status_code: HTTP状态码
        response_data: API响应数据
        
    返回:
        相应的异常实例
    """
    error_message = response_data.get('error', 'Unknown error')
    
    if status_code == 400:
        return RerankValidationError(error_message)
    elif status_code == 401:
        return RerankAuthenticationError(error_message)
    elif status_code == 404:
        return RerankModelNotFoundError(response_data.get('model', 'unknown'))
    elif status_code == 429:
        return RerankRateLimitError(error_message)
    elif status_code == 503:
        return RerankServiceUnavailableError(error_message)
    elif status_code >= 500:
        return RerankError(f"Server error: {error_message}", status_code, response_data)
    else:
        return RerankError(f"HTTP {status_code}: {error_message}", status_code, response_data) 