# generate_sdk/generate_config.py
from __future__ import annotations
from dataclasses import dataclass
from typing import Optional, Dict

@dataclass
class GenerateConfig:
    base_url: str = "http://127.0.0.1:8000"

    # 与 actix 路由保持一致
    generate_path: str = "/v1/tgi/generate"
    generate_stream_path: str = "/v1/tgi/generate_stream"
    health_path: str = "/health"

    # 鉴权/超时/额外头
    api_key: Optional[str] = None
    timeout: float = 60.0
    extra_headers: Optional[Dict[str, str]] = None

    # 默认模型
    default_generate_model: str = "Qwen/Qwen2.5-7B-Instruct"

    def get_health_url(self) -> str:
        return f"{self.base_url}{self.health_path}"

    def get_generate_url(self) -> str:
        return f"{self.base_url}{self.generate_path}"

    def get_generate_stream_url(self) -> str:
        return f"{self.base_url}{self.generate_stream_path}"
