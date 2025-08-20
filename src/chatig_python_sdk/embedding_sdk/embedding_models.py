# -*- coding: utf-8 -*-
"""
embedding_models.py
-------------------
Embedding SDK 的数据与校验模型。

包含：
- EmbeddingEngineType: 引擎类型枚举
- EmbeddingValidationError: 客户端校验异常
- StdEmbeddingRequest: 标准化的嵌入请求体（含 validate/normalize 方法）
- EmbeddingData/Usage/EmbeddingResponse: 与服务端响应对齐的数据模型
"""

from __future__ import annotations
from enum import Enum
from typing import List, Optional, Union
from pydantic import BaseModel, Field, validator


class EmbeddingEngineType(str, Enum):
    QWEN = "Qwen"
    GLM = "GLM"
    LLAMA = "meta-llama"
    BAILIAN = "Bailian"
    DEEPSEEK = "deepseek-ai"
    OTHER = "other"


class EmbeddingValidationError(ValueError):
    """本地参数校验错误"""


class StdEmbeddingRequest(BaseModel):
    model: str = Field(..., description="模型名称或路径")
    input: Union[str, List[str]] = Field(..., description="单条文本或文本数组")
    encoding_format: str = Field("float", description="float | base64")
    dimensions: Optional[int] = Field(None, description="自定义维度（可选）")
    user: Optional[str] = Field(None, description="可选的用户标识")

    class Config:
        validate_assignment = True

    @validator("encoding_format")
    def _check_encoding_format(cls, v: str) -> str:
        allowed = {"float", "base64"}
        if v not in allowed:
            raise EmbeddingValidationError(f"encoding_format 必须是 {allowed}, 实际: {v}")
        return v

    @validator("dimensions")
    def _check_dimensions(cls, v: Optional[int]) -> Optional[int]:
        if v is not None and v <= 0:
            raise EmbeddingValidationError("dimensions 必须为正整数")
        return v

    def normalize(self) -> "StdEmbeddingRequest":
        """将 input 统一为 List[str] 并去除空白"""
        if isinstance(self.input, str):
            normalized = [self.input]
        else:
            normalized = list(self.input)
        normalized = [s for s in (x.strip() for x in normalized) if s]
        object.__setattr__(self, "input", normalized)  # pydantic BaseModel 的字段写保护需要用 object.__setattr__
        return self

    def validate(self) -> None:  # 与测试脚本保持一致的接口
        if not self.model or not str(self.model).strip():
            raise EmbeddingValidationError("model 不能为空")
        # 归一化输入
        self.normalize()
        if not isinstance(self.input, list) or not self.input:
            raise EmbeddingValidationError("input 必须是非空的字符串数组")
        if not all(isinstance(x, str) and x.strip() for x in self.input):
            raise EmbeddingValidationError("input 中存在空字符串或非字符串元素")


class EmbeddingData(BaseModel):
    object: str = Field("embedding", Literal=True)
    index: int
    embedding: List[float]


class Usage(BaseModel):
    prompt_tokens: int = 0
    total_tokens: int = 0


class EmbeddingResponse(BaseModel):
    object: str = Field("list", Literal=True)
    data: List[EmbeddingData]
    model: str
    usage: Usage
