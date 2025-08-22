# embedding_sdk/client.py
from __future__ import annotations

import json
from typing import Any, Dict, List, Optional

import requests
from requests import Session

from .embedding_config import EmbeddingConfig
from .embedding_models import (
    StdEmbeddingRequest,
    EmbeddingResponse,
    EmbeddingData,
    EmbeddingValidationError,
)


class EmbeddingsClient:
    """
    ChatIG Embeddings 轻量客户端
    - health_check()
    - embed(model, input)  —— 兼容 OpenAI 返回结构
    """

    def __init__(
        self,
        config: Optional[EmbeddingConfig] = None,
        session: Optional[Session] = None,
    ) -> None:
        self.cfg = config or EmbeddingConfig()
        self._session: Session = session or requests.Session()
        self.default_model: str = getattr(
            self.cfg, "default_embedding_model", "Qwen/Qwen2.5-7B-Embeddings"
        )

    def __enter__(self) -> "EmbeddingsClient":
        return self

    def __exit__(self, exc_type, exc, tb) -> None:
        self.close()

    # --------- 内部工具 ----------
    def _headers(self) -> Dict[str, str]:
        headers = {"Content-Type": "application/json"}
        api_key = getattr(self.cfg, "api_key", None)
        if api_key:
            headers["Authorization"] = f"Bearer {api_key}"
        extra = getattr(self.cfg, "extra_headers", None)
        if isinstance(extra, dict):
            headers.update(extra)
        return headers

    # --------- 生命周期 ----------
    def close(self) -> None:
        try:
            self._session.close()
        except Exception:
            pass

    # --------- 健康检查 ----------
    def health_check(self) -> bool:
        url = self.cfg.get_health_url()
        try:
            r = self._session.get(url, headers=self._headers(), timeout=self.cfg.timeout)
            return 200 <= r.status_code < 300
        except Exception:
            return False

    # --------- 向量化 ----------
    def embed(self, model: str, input: List[str] | str) -> EmbeddingResponse:
        # 1) 本地校验
        req = StdEmbeddingRequest(model=model, input=input)
        _ = req.model, req.input  # 触发 pydantic 校验

        # 2) str -> [str]
        payload_input: List[str] = [req.input] if isinstance(req.input, str) else req.input

        url = self.cfg.get_embeddings_url()
        payload: Dict[str, Any] = {"model": req.model, "input": payload_input}

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

        try:
            j = resp.json()
        except Exception as e:
            raise ValueError(f"Response is not valid JSON: {e}") from e

        # OpenAI 形状优先
        if isinstance(j, dict) and "data" in j:
            try:
                items = []
                for i, d in enumerate(j.get("data", [])):
                    emb = d.get("embedding", d.get("vector"))
                    if not isinstance(emb, list):
                        raise ValueError("data[i].embedding is not a list")
                    items.append(EmbeddingData(embedding=emb, index=d.get("index", i)))
                usage = EmbeddingUsage(
                    prompt_tokens=(j.get("usage") or {}).get("prompt_tokens", 0),
                    total_tokens=(j.get("usage") or {}).get("total_tokens", 0),
                )
                return EmbeddingResponse(
                    data=items,
                    model=j.get("model", req.model),
                    usage=usage,
                )
            except Exception as e:
                raise ValueError(f"Failed to parse OpenAI-like response: {e}") from e

        # 其它备选形状
        if "embeddings" in j and isinstance(j["embeddings"], list):
            items = [EmbeddingData(embedding=v, index=i) for i, v in enumerate(j["embeddings"])]
            return EmbeddingResponse(
                data=items,
                model=j.get("model", req.model),
                usage=EmbeddingUsage(total_tokens=j.get("total_tokens", 0)),
            )

        if "data" in j and isinstance(j["data"], list):
            items = []
            for i, d in enumerate(j["data"]):
                v = d.get("vector")
                if isinstance(v, list):
                    items.append(EmbeddingData(embedding=v, index=i))
            if items:
                return EmbeddingResponse(
                    data=items,
                    model=j.get("model", req.model),
                    usage=EmbeddingUsage(
                        prompt_tokens=(j.get("usage") or {}).get("prompt_tokens", 0),
                        total_tokens=(j.get("usage") or {}).get("total_tokens", 0),
                    ),
                )

        raise ValueError(f"Unrecognized embeddings response shape: {j}")
