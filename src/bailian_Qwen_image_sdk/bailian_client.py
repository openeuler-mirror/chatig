from __future__ import annotations

import base64
import os
from dataclasses import dataclass
from typing import Optional, Union

import requests


class BailianError(Exception):
    pass


@dataclass
class BailianClient:
    api_key: Optional[str] = None
    base_url: str = (
        "https://dashscope.aliyuncs.com/api/v1/services/aigc/multimodal-generation/generation"
    )
    timeout: float = 60.0
    max_retries: int = 2

    def __post_init__(self) -> None:
        if not self.api_key:
            self.api_key = os.getenv("BAILIAN_API_KEY")
        if not self.api_key:
            raise BailianError("BAILIAN_API_KEY 未配置，且未在构造函数中提供 api_key")
        self._headers = {
            "Content-Type": "application/json",
            "Authorization": f"Bearer {self.api_key}",
        }

    # Public API
    def generate(self, prompt: str, size: str = "1328*1328", watermark: bool = False) -> bytes:
        payload = {
            "model": "qwen-image",
            "input": {
                "messages": [
                    {
                        "role": "user",
                        "content": [
                            {
                                "text": prompt,
                            }
                        ],
                    }
                ]
            },
            "parameters": {"size": size, "watermark": watermark},
        }
        resp = self._request_json(payload)
        image_url = self._extract_image_url(resp)
        return self._download_image(image_url)

    def edit(
        self,
        image: Union[bytes, str],
        prompt: str,
        negative_prompt: str = "",
        watermark: bool = False,
    ) -> bytes:
        if isinstance(image, bytes):
            image_b64 = base64.b64encode(image).decode("utf-8")
            image_val = f"data:image/jpeg;base64,{image_b64}"
        else:
            image_val = image

        payload = {
            "model": "qwen-image-edit",
            "input": {
                "messages": [
                    {
                        "role": "user",
                        "content": [
                            {"image": image_val},
                            {"text": prompt},
                        ],
                    }
                ]
            },
            "parameters": {
                "negative_prompt": negative_prompt,
                "watermark": watermark,
            },
        }
        resp = self._request_json(payload)
        image_url = self._extract_image_url(resp)
        return self._download_image(image_url)

    # Internal helpers
    def _request_json(self, payload: dict) -> dict:
        last_err: Optional[Exception] = None
        for _ in range(self.max_retries + 1):
            try:
                r = requests.post(
                    self.base_url, headers=self._headers, json=payload, timeout=self.timeout
                )
                if r.status_code >= 400:
                    raise BailianError(f"HTTP {r.status_code}: {r.text}")
                return r.json()
            except Exception as e:  # network or json
                last_err = e
        raise BailianError(f"调用百炼失败: {last_err}")

    @staticmethod
    def _extract_image_url(resp: dict) -> str:
        try:
            choices = resp["output"]["choices"]
            content = choices[0]["message"]["content"]
            image_url = content[0]["image"]
            if not isinstance(image_url, str) or not image_url:
                raise ValueError("invalid image url")
            return image_url
        except Exception as e:
            raise BailianError(f"解析响应失败，未找到图片URL: {e}; 响应片段: {str(resp)[:300]}")

    def _download_image(self, url: str) -> bytes:
        last_err: Optional[Exception] = None
        for _ in range(self.max_retries + 1):
            try:
                r = requests.get(url, timeout=self.timeout)
                if r.status_code >= 400:
                    raise BailianError(f"下载图片失败 HTTP {r.status_code}")
                return r.content
            except Exception as e:
                last_err = e
        raise BailianError(f"下载图片失败: {last_err}")


