"""
Chatig 重排序 SDK 的数据模型

本模块包含用于重排序请求和响应的数据结构，
映射到Rust后端结构。
"""

from typing import List, Optional, Union, Dict, Any
from dataclasses import dataclass, field
from enum import Enum

from .exceptions import RerankValidationError


class RerankEngineType(str, Enum):
    """支持的重排序引擎类型"""
    STD = "std"
    VLLM = "vllm"
    LLAMABOX = "llamabox"


@dataclass
class RerankParameters:
    """重排序请求的参数"""
    top_n: Optional[int] = None
    return_documents: Optional[bool] = None


@dataclass
class Document:
    """重排序响应的文档结构"""
    text: str


@dataclass
class Usage:
    """重排序请求的使用统计"""
    prompt_tokens: Optional[int] = None
    total_tokens: Optional[int] = None


@dataclass
class RerankResult:
    """基础重排序结果结构"""
    index: int
    score: float
    document: Optional[Document] = None


# ====================================================
# 标准重排序模型
# ====================================================

@dataclass
class StdRerankRequest:
    """标准重排序请求结构"""
    model: str
    query: str
    documents: List[str]
    top_n: Optional[int] = None
    parameters: Optional[RerankParameters] = None

    def validate(self) -> None:
        """验证请求参数"""
        if not self.model:
            raise RerankValidationError("Model name cannot be empty", field="model")
        if not self.query:
            raise RerankValidationError("Query text cannot be empty", field="query")
        if not self.documents:
            raise RerankValidationError("Documents list cannot be empty", field="documents")


@dataclass
class StdRerankResult:
    """标准重排序结果结构"""
    index: int
    score: float


@dataclass
class StdRerankResponse:
    """标准重排序响应结构"""
    results: StdRerankResult


# ====================================================
# LlamaBox 重排序模型
# ====================================================

@dataclass
class LlamaBoxRerankRequest:
    """LlamaBox重排序请求结构"""
    model: str
    query: str
    documents: List[str]
    top_n: Optional[int] = None

    def validate(self) -> None:
        """验证请求参数"""
        if not self.model:
            raise RerankValidationError("Model name cannot be empty", field="model")
        if not self.query:
            raise RerankValidationError("Query text cannot be empty", field="query")
        if not self.documents:
            raise RerankValidationError("Documents list cannot be empty", field="documents")


@dataclass
class LlamaRerankResult:
    """LlamaBox重排序结果结构"""
    index: int
    relevance_score: float
    document: Document


@dataclass
class LlamaBoxRerankResponse:
    """LlamaBox重排序响应结构"""
    model: str
    query: Optional[str] = None
    results: List[LlamaRerankResult] = field(default_factory=list)
    usage: Optional[Usage] = None


# ====================================================
# VLLM 重排序模型
# ====================================================

@dataclass
class VLLMRerankRequest:
    """VLLM重排序请求结构"""
    model: str
    query: str
    documents: List[str]

    def validate(self) -> None:
        """验证请求参数"""
        if not self.model:
            raise RerankValidationError("Model name cannot be empty", field="model")
        if not self.query:
            raise RerankValidationError("Query text cannot be empty", field="query")
        if not self.documents:
            raise RerankValidationError("Documents list cannot be empty", field="documents")


@dataclass
class VLLMRerankResult:
    """VLLM重排序结果结构"""
    index: int
    document: Document
    relevance_score: float


@dataclass
class VLLMUsage:
    """VLLM使用统计"""
    total_tokens: int


@dataclass
class VLLMRerankResponse:
    """VLLM重排序响应结构"""
    id: str
    model: str
    usage: VLLMUsage
    results: List[VLLMRerankResult] = field(default_factory=list)


# ====================================================
# Mindie 重排序模型（完整性考虑）
# ====================================================

@dataclass
class MindieRerankRequest:
    """Mindie重排序请求结构"""
    query: str
    texts: List[str]

    def validate(self) -> None:
        """验证请求参数"""
        if not self.query:
            raise RerankValidationError("Query text cannot be empty", field="query")
        if not self.texts:
            raise RerankValidationError("Texts list cannot be empty", field="texts")


@dataclass
class MindieRerankResponse:
    """Mindie重排序响应结构"""
    index: int
    score: float


# ====================================================
# 工具函数
# ====================================================

def create_rerank_request(
    engine_type: RerankEngineType,
    model: str,
    query: str,
    documents: List[str],
    **kwargs
) -> Union[StdRerankRequest, LlamaBoxRerankRequest, VLLMRerankRequest]:
    """
    根据引擎类型创建重排序请求的工厂函数
    
    参数:
        engine_type: 重排序引擎类型
        model: 模型名称
        query: 查询文本
        documents: 要重排序的文档列表
        **kwargs: 附加参数
        
    返回:
        相应的重排序请求对象
    """
    if engine_type == RerankEngineType.STD:
        return StdRerankRequest(
            model=model,
            query=query,
            documents=documents,
            top_n=kwargs.get('top_n'),
            parameters=kwargs.get('parameters')
        )
    elif engine_type == RerankEngineType.LLAMABOX:
        return LlamaBoxRerankRequest(
            model=model,
            query=query,
            documents=documents,
            top_n=kwargs.get('top_n')
        )
    elif engine_type == RerankEngineType.VLLM:
        return VLLMRerankRequest(
            model=model,
            query=query,
            documents=documents
        )
    else:
        raise ValueError(f"Unsupported engine type: {engine_type}")


def parse_rerank_response(
    engine_type: RerankEngineType,
    response_data: Dict[str, Any]
) -> Union[StdRerankResponse, LlamaBoxRerankResponse, VLLMRerankResponse]:
    """
    根据引擎类型解析重排序响应
    
    参数:
        engine_type: 重排序引擎类型
        response_data: API响应数据
        
    返回:
        相应的重排序响应对象
    """
    if engine_type == RerankEngineType.STD:
        results_data = response_data.get('results', [])
        if isinstance(results_data, list) and len(results_data) > 0:
            # 取第一个结果（top_n=1的情况）
            result_data = results_data[0]
        else:
            # 默认结果
            result_data = {}
        
        result = StdRerankResult(
            index=result_data.get('index', 0),
            score=result_data.get('score', 0.0)
        )
        return StdRerankResponse(results=result)
        
    elif engine_type == RerankEngineType.LLAMABOX:
        results = [
            LlamaRerankResult(
                index=result.get('index', 0),
                relevance_score=result.get('relevance_score', 0.0),
                document=Document(text=result.get('document', {}).get('text', ''))
            )
            for result in response_data.get('results', [])
        ]
        usage_data = response_data.get('usage', {})
        usage = Usage(
            prompt_tokens=usage_data.get('prompt_tokens'),
            total_tokens=usage_data.get('total_tokens')
        )
        return LlamaBoxRerankResponse(
            model=response_data.get('model', ''),
            query=response_data.get('query'),
            results=results,
            usage=usage
        )
        
    elif engine_type == RerankEngineType.VLLM:
        results = [
            VLLMRerankResult(
                index=result.get('index', 0),
                document=Document(text=result.get('document', {}).get('text', '')),
                relevance_score=result.get('relevance_score', 0.0)
            )
            for result in response_data.get('results', [])
        ]
        usage_data = response_data.get('usage', {})
        usage = VLLMUsage(total_tokens=usage_data.get('total_tokens', 0))
        return VLLMRerankResponse(
            id=response_data.get('id', ''),
            model=response_data.get('model', ''),
            usage=usage,
            results=results
        )
    else:
        raise ValueError(f"Unsupported engine type: {engine_type}")