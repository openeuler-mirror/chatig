"""
Chatig 重排序 SDK

一个用于与Chatig重排序API端点交互的Python SDK。
支持多种重排序引擎：std、vllm和llamabox。
"""

from .rerank_client import RerankClient
from .models import (
    StdRerankRequest,
    StdRerankResponse,
    LlamaBoxRerankRequest,
    LlamaBoxRerankResponse,
    VLLMRerankRequest,
    VLLMRerankResponse,
    RerankParameters,
    RerankResult,
    Document,
    Usage
)
from .exceptions import (
    RerankError,
    RerankValidationError,
    RerankConnectionError,
    RerankTimeoutError
)
from .config import RerankConfig

__version__ = "1.0.0"
__author__ = "Chatig Team"

__all__ = [
    "RerankClient",
    "StdRerankRequest",
    "StdRerankResponse", 
    "LlamaBoxRerankRequest",
    "LlamaBoxRerankResponse",
    "VLLMRerankRequest",
    "VLLMRerankResponse",
    "RerankParameters",
    "RerankResult",
    "Document",
    "Usage",
    "RerankError",
    "RerankValidationError",
    "RerankConnectionError",
    "RerankTimeoutError",
    "RerankConfig"
] 