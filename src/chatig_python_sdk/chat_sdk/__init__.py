"""
ChatIG Chat Module - 推理网关SDK封装
提供与ChatIG推理网关的聊天功能接口
"""

from .chat_client import ChatClient
from .config import ChatConfig
from .models import (
    ChatMessage,
    ChatCompletionRequest,
    ChatCompletionResponse,
    Choice as ChatCompletionChoice,
    Usage as ChatCompletionUsage,
    StreamOptions,
    CompletionsResponse,
    CompletionsChoice,
    CompletionsAssistantMessage,
    CompletionsUsage,
    CompletionsStreamResponse,
    CompletionsStreamChoice,
    CompletionsDelta
)
from .exceptions import (
    ChatIGError, 
    AuthenticationError, 
    RateLimitError, 
    ServerError,
    ValidationError,
    ModelNotFoundError,
    ModelSeriesNotSupportedError,
    InvalidModelFormatError,
    StreamError,
    TimeoutError,
    NetworkError,
    APIError,
    ConfigurationError
)

__version__ = "1.0.0"

__all__ = [
    # 主要客户端
    "ChatClient",
    "ChatConfig",
    
    # 数据模型
    "ChatMessage",
    "ChatCompletionRequest", 
    "ChatCompletionResponse",
    "ChatCompletionChoice",
    "ChatCompletionUsage",
    "StreamOptions",
    "CompletionsResponse",
    "CompletionsChoice",
    "CompletionsAssistantMessage",
    "CompletionsUsage",
    "CompletionsStreamResponse",
    "CompletionsStreamChoice",
    "CompletionsDelta",
    
    # 异常类
    "ChatIGError",
    "AuthenticationError", 
    "RateLimitError",
    "ServerError",
    "ValidationError",
    "ModelNotFoundError",
    "ModelSeriesNotSupportedError",
    "InvalidModelFormatError",
    "StreamError",
    "TimeoutError",
    "NetworkError",
    "APIError",
    "ConfigurationError",
    
    # 版本信息
    "__version__"
] 