# generate_sdk/client.py
from __future__ import annotations

import json
from typing import Any, Dict, Iterable, Optional

import requests
from requests import Session

from .generate_config import GenerateConfig
from .generate_models import (
    StdGenerateRequest,
    ChatResponse,
    ChatChoice,
    ChatDelta,
    ChatMessage,
    ChatUsage,
    SimpleTextResponse,
)

class GenerateClient:
    """
    对应 actix_web:
      - POST /v1/tgi/generate
      - POST /v1/tgi/generate_stream
    提供:
      - health_check()
      - generate()          —— 非流式
      - generate_stream()   —— 流式（SSE/行流）
    兼容返回：优先 OpenAI Chat Completions 形状，其次常见简化形状。
    """

    def __init__(self, config: Optional[GenerateConfig] = None, session: Optional[Session] = None) -> None:
        self.cfg = config or GenerateConfig()
        self._session: Session = session or requests.Session()
        self.default_model = self.cfg.default_generate_model

    def __enter__(self) -> "GenerateClient":
        return self

    def __exit__(self, exc_type, exc, tb) -> None:
        self.close()

    # ---------------- headers / 生命周期 ----------------
    def _headers(self) -> Dict[str, str]:
        headers = {"Content-Type": "application/json"}
        if self.cfg.api_key:
            headers["Authorization"] = f"Bearer {self.cfg.api_key}"
        if isinstance(self.cfg.extra_headers, dict):
            headers.update(self.cfg.extra_headers)
        return headers

    def close(self) -> None:
        try:
            self._session.close()
        except Exception:
            pass

    # ---------------- 健康检查 ----------------
    def health_check(self) -> bool:
        url = self.cfg.get_health_url()
        try:
            r = self._session.get(url, headers=self._headers(), timeout=self.cfg.timeout)
            return 200 <= r.status_code < 300
        except Exception:
            return False

    # ---------------- 非流式 ----------------
    def generate(
        self,
        model: Optional[str] = None,
        messages: Optional[list[dict]] = None,
        prompt: Optional[str] = None,
        **kwargs,
    ) -> ChatResponse:
        """
        调用 /v1/tgi/generate
        入参兼容：
          - OpenAI: messages=[{role, content}, ...]
          - 简化: prompt="..."
        """
        req = StdGenerateRequest(
            model=model or self.default_model,
            messages=[ChatMessage(**m) for m in messages] if messages else None,
            prompt=prompt,
            **kwargs,
        )
        payload = req.to_payload()

        url = self.cfg.get_generate_url()
        resp = self._session.post(
            url,
            data=json.dumps(payload, ensure_ascii=False),
            headers=self._headers(),
            timeout=self.cfg.timeout,
        )
        try:
            resp.raise_for_status()
        except requests.HTTPError as e:
            raise requests.HTTPError(f"HTTP {resp.status_code} Error: {resp.text}") from e

        j = resp.json()

        # 1) OpenAI 形状
        if isinstance(j, dict) and "choices" in j:
            usage = j.get("usage")
            return ChatResponse(
                model=j.get("model", req.model),
                choices=[ChatChoice(**c) for c in j.get("choices", [])],
                usage=ChatUsage(**usage) if isinstance(usage, dict) else None,
            )

        # 2) 简化：{"text":"..."}
        if isinstance(j, dict) and "text" in j:
            s = SimpleTextResponse(**j)
            return ChatResponse(
                model=s.model or req.model,
                choices=[ChatChoice(index=0, message=ChatMessage(role="assistant", content=s.text))],
                usage=s.usage,
            )

        # 3) 兜底：{data:[{text|generated_text:"..."}]}
        if "data" in j and isinstance(j["data"], list) and j["data"]:
            first = j["data"][0]
            text = first.get("text") or first.get("generated_text") or ""
            return ChatResponse(
                model=j.get("model", req.model),
                choices=[ChatChoice(index=0, message=ChatMessage(role="assistant", content=text))],
                usage=None,
            )

        raise ValueError(f"Unrecognized generate response shape: {j}")

    # ---------------- 流式 ----------------
    def generate_stream(
        self,
        model: Optional[str] = None,
        messages: Optional[list[dict]] = None,
        prompt: Optional[str] = None,
        **kwargs,
    ) -> Iterable[str]:
        """
        调用 /v1/tgi/generate_stream
        逐块产出增量文本（content 片段）。
        兼容：
          - SSE: 行以 "data: {json}"，json 里遵循 OpenAI delta 形状
          - 行文本/简单 JSON 增量：{"text"|"delta"|"content"|"chunk":"..."}
        """
        req = StdGenerateRequest(
            model=model or self.default_model,
            messages=[ChatMessage(**m) for m in messages] if messages else None,
            prompt=prompt,
            stream=True,
            **kwargs,
        )
        payload = req.to_payload()

        url = self.cfg.get_generate_stream_url()
        with self._session.post(
            url,
            data=json.dumps(payload, ensure_ascii=False),
            headers=self._headers(),
            timeout=self.cfg.timeout,
            stream=True,
        ) as r:
            r.raise_for_status()
            for raw in r.iter_lines(decode_unicode=True):
                if not raw:
                    continue
                line = raw.strip()
                # SSE
                if line.startswith("data:"):
                    line = line[len("data:"):].strip()
                # 纯文本
                if line and not (line.startswith("{") or line.startswith("[")):
                    yield line
                    continue
                # JSON
                try:
                    j = json.loads(line)
                except Exception:
                    continue
                if isinstance(j, dict) and "choices" in j:
                    try:
                        choices = j.get("choices") or []
                        if choices:
                            delta = choices[0].get("delta") or {}
                            content = delta.get("content")
                            if content:
                                yield content
                                continue
                    except Exception:
                        pass
                for key in ("text", "delta", "content", "chunk"):
                    if key in j and isinstance(j[key], str):
                        yield j[key]
                        break
