from __future__ import annotations

import base64
from dataclasses import dataclass
from typing import Optional

import requests


class PlatformError(Exception):
    pass


@dataclass
class PlatformClient:
    base_url: str
    api_key: Optional[str] = None
    timeout: float = 30.0
    max_retries: int = 2

    def __post_init__(self) -> None:
        headers = {"Content-Type": "application/json"}
        if self.api_key:
            headers["Authorization"] = f"Bearer {self.api_key}"
        self._headers = headers

    # ===== Generate =====
    def generate(self, prompt: str, size: str = "1328*1328", watermark: bool = False) -> bytes:
        payload = {"prompt": prompt, "size": size, "watermark": watermark}
        data = self._post_json("/generate", payload)
        b64 = data.get("image_base64")
        if not b64:
            raise PlatformError("响应缺少 image_base64")
        return base64.b64decode(b64)

    def generate_raw(self, prompt: str, size: str = "1328*1328", watermark: bool = False) -> bytes:
        form = {"prompt": prompt, "size": size, "watermark": str(watermark).lower()}
        return self._post_bytes("/generate/raw", form)

    # ===== Edit (Bailian) =====
    def edit_bailian(
        self, image_bytes: bytes, prompt: str, negative_prompt: str = "", watermark: bool = False
    ) -> bytes:
        files = {"file": ("image.png", image_bytes, "image/png")}
        form = {
            "prompt": prompt,
            "negative_prompt": negative_prompt,
            "watermark": str(watermark).lower(),
        }
        data = self._post_multipart_json("/edit/bailian", files, form)
        b64 = data.get("image_base64")
        if not b64:
            raise PlatformError("响应缺少 image_base64")
        return base64.b64decode(b64)

    def edit_bailian_raw(
        self, image_bytes: bytes, prompt: str, negative_prompt: str = "", watermark: bool = False
    ) -> bytes:
        files = {"file": ("image.png", image_bytes, "image/png")}
        form = {
            "prompt": prompt,
            "negative_prompt": negative_prompt,
            "watermark": str(watermark).lower(),
        }
        return self._post_multipart_bytes("/edit/bailian/raw", files, form)

    # ===== Sessions =====
    def create_session(self) -> str:
        data = self._post_json("/sessions", {})
        sid = data.get("session_id")
        if not sid:
            raise PlatformError("响应缺少 session_id")
        return sid

    # ===== Low-level =====
    def _post_json(self, path: str, payload: dict) -> dict:
        url = self.base_url.rstrip("/") + path
        last_err: Optional[Exception] = None
        for _ in range(self.max_retries + 1):
            try:
                r = requests.post(url, json=payload, headers=self._headers, timeout=self.timeout)
                if r.status_code >= 400:
                    raise PlatformError(f"HTTP {r.status_code}: {r.text}")
                return r.json()
            except Exception as e:
                last_err = e
        raise PlatformError(f"请求失败: {last_err}")

    def _post_bytes(self, path: str, form: dict) -> bytes:
        url = self.base_url.rstrip("/") + path
        last_err: Optional[Exception] = None
        for _ in range(self.max_retries + 1):
            try:
                r = requests.post(url, data=form, headers={k: v for k, v in self._headers.items() if k.lower() != "content-type"}, timeout=self.timeout)
                if r.status_code >= 400:
                    raise PlatformError(f"HTTP {r.status_code}: {r.text}")
                return r.content
            except Exception as e:
                last_err = e
        raise PlatformError(f"请求失败: {last_err}")

    def _post_multipart_json(self, path: str, files: dict, form: dict) -> dict:
        url = self.base_url.rstrip("/") + path
        last_err: Optional[Exception] = None
        for _ in range(self.max_retries + 1):
            try:
                r = requests.post(url, files=files, data=form, headers={k: v for k, v in self._headers.items() if k.lower() != "content-type"}, timeout=self.timeout)
                if r.status_code >= 400:
                    raise PlatformError(f"HTTP {r.status_code}: {r.text}")
                return r.json()
            except Exception as e:
                last_err = e
        raise PlatformError(f"请求失败: {last_err}")

    def _post_multipart_bytes(self, path: str, files: dict, form: dict) -> bytes:
        url = self.base_url.rstrip("/") + path
        last_err: Optional[Exception] = None
        for _ in range(self.max_retries + 1):
            try:
                r = requests.post(url, files=files, data=form, headers={k: v for k, v in self._headers.items() if k.lower() != "content-type"}, timeout=self.timeout)
                if r.status_code >= 400:
                    raise PlatformError(f"HTTP {r.status_code}: {r.text}")
                return r.content
            except Exception as e:
                last_err = e
        raise PlatformError(f"请求失败: {last_err}")


