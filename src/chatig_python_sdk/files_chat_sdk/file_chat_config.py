# file_chat_sdk/file_chat_config.py
from __future__ import annotations
from dataclasses import dataclass
from typing import Optional, Dict

@dataclass
class FileChatConfig:
    base_url: str = "http://127.0.0.1:8001"

    # 与后端 actix 路由保持一致
    upload_path: str = "/v1/files"
    completions_path: str = "/v1/file/completions"
    health_path: str = "/health"  # 如无可忽略

    # 鉴权/超时/额外头
    api_key: Optional[str] = None
    timeout: float = 60.0
    extra_headers: Optional[Dict[str, str]] = None

    # 默认模型（按你的后端适配）
    default_file_chat_model: str = "chatchat"

    def get_health_url(self) -> str:
        return f"{self.base_url}{self.health_path}"

    def get_upload_url(self) -> str:
        return f"{self.base_url}{self.upload_path}"

    def get_completions_url(self) -> str:
        return f"{self.base_url}{self.completions_path}"
