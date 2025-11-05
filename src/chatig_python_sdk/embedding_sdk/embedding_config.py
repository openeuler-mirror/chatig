# -*- coding: utf-8 -*-
"""
embedding_config.py
-------------------
Embedding SDK 的配置模块。

- 从环境变量或默认值加载配置：
  CHATIG_API_KEY, CHATIG_API_BASE, CHATIG_TIMEOUT, CHATIG_MAX_RETRIES, CHATIG_RETRY_DELAY
- 提供便捷的 URL 生成方法：get_health_url(), get_embeddings_url()
- 提供 headers() 统一注入鉴权头
"""

from __future__ import annotations
import os
from typing import Optional
from pydantic import BaseModel, Field, validator


class EmbeddingConfig(BaseModel):
    
    # 基础配置
    api_key: Optional[str] = Field(default_factory=lambda: os.getenv("CHATIG_API_KEY", "sk-gw2m0VxGMSjXqG37TWQXlCGs0Wz6wDPe") or None)
    base_url: str = Field(default_factory=lambda: os.getenv("CHATIG_API_BASE", "http://127.0.0.1:8001"))
    timeout: float = Field(default_factory=lambda: float(os.getenv("CHATIG_TIMEOUT", "60.0")))
    max_retries: int = Field(default_factory=lambda: int(os.getenv("CHATIG_MAX_RETRIES", "3")))
    retry_delay: float = Field(default_factory=lambda: float(os.getenv("CHATIG_RETRY_DELAY", "1.0")))

    class Config:
        validate_assignment = True

    @validator("base_url")
    def _strip_trailing_slash(cls, v: str) -> str:
        return v[:-1] if v.endswith("/") else v

    def get_health_url(self) -> str:
        return f"{self.base_url}/health"

    def get_embeddings_url(self) -> str:
        return f"{self.base_url}/v1/embeddings"

    def headers(self) -> dict:
        headers = {"Content-Type": "application/json"}
        if self.api_key:
            headers["Authorization"] = f"Bearer {self.api_key}"
        return headers
