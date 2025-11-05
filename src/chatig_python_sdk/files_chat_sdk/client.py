# file_chat_sdk/client.py
from __future__ import annotations

import json
from typing import Any, Dict, Iterable, List, Optional, Tuple

import requests
from requests import Session

from .file_chat_config import FileChatConfig
from .file_chat_models import (
    FileChatCompletionRequest,
    ChatResponse,
    ChatChoice,
    ChatMessage,
    ChatUsage,
    UploadResponse,
    UploadFileInfo,
)

class FileChatClient:
    """
    对应 actix_web:
      - POST /v1/files                （上传文档）
      - POST /v1/file/completions     （基于文件的对话）
    提供:
      - health_check()
      - upload_files()
      - file_completions()
    """

    def __init__(self, config: Optional[FileChatConfig] = None, session: Optional[Session] = None) -> None:
        self.cfg = config or FileChatConfig()
        self._session: Session = session or requests.Session()
        self.default_model = self.cfg.default_file_chat_model

    def __enter__(self) -> "FileChatClient":
        return self

    def __exit__(self, exc_type, exc, tb) -> None:
        self.close()

    # ---------------- headers / 生命周期 ----------------
    def _headers_json(self) -> Dict[str, str]:
        headers = {"Content-Type": "application/json"}
        if self.cfg.api_key:
            headers["Authorization"] = f"Bearer {self.cfg.api_key}"
        if isinstance(self.cfg.extra_headers, dict):
            headers.update(self.cfg.extra_headers)
        return headers

    def _headers_multipart(self) -> Dict[str, str]:
        # requests 在 files= 场景会自动设置 multipart Content-Type，这里只加鉴权/额外头
        headers: Dict[str, str] = {}
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

    # ---------------- 健康检查（如无可忽略） ----------------
    def health_check(self) -> bool:
        url = self.cfg.get_health_url()
        try:
            r = self._session.get(url, headers=self._headers_json(), timeout=self.cfg.timeout)
            return 200 <= r.status_code < 300
        except Exception:
            return False

    # ---------------- 上传文件 ----------------
    def upload_files(
        self,
        file_tuples: List[Tuple[str, bytes, Optional[str]]],
        form_fields: Optional[Dict[str, str]] = None,
    ) -> UploadResponse:
        """
        上传临时文档到 /v1/files
        参数:
          - file_tuples: 列表，每项为 (field_name, file_bytes, filename)
              例: [("files", open("a.pdf","rb").read(), "a.pdf"), ("files", b"...", "b.txt")]
            若你的后端 Multipart 字段名不同，请相应调整 field_name。
          - form_fields: 额外的表单字段（如用户/会话信息等），可选。
        返回:
          - UploadResponse（宽松解析 doc_ids / files 信息）
        """
        url = self.cfg.get_upload_url()

        files = []
        for field_name, file_bytes, filename in file_tuples:
            files.append(
                (field_name, (filename or "blob", file_bytes))
            )

        data = form_fields or {}
        resp = self._session.post(
            url,
            files=files,
            data=data,
            headers=self._headers_multipart(),
            timeout=self.cfg.timeout,
        )
        try:
            resp.raise_for_status()
        except requests.HTTPError as e:
            raise requests.HTTPError(f"HTTP {resp.status_code} Error: {resp.text}") from e

        # 宽松解析：支持后端返回 {ok, doc_ids, files:[{filename,file_id,size}], message}
        try:
            j = resp.json()
        except Exception:
            # 若纯文本，兜底包装
            return UploadResponse(ok=True, message=resp.text)

        out = UploadResponse(ok=True)
        if isinstance(j, dict):
            out.ok = bool(j.get("ok", True))
            if "doc_ids" in j and isinstance(j["doc_ids"], list):
                out.doc_ids = [str(x) for x in j["doc_ids"]]
            if "files" in j and isinstance(j["files"], list):
                items: List[UploadFileInfo] = []
                for f in j["files"]:
                    if not isinstance(f, dict):
                        continue
                    items.append(
                        UploadFileInfo(
                            filename=f.get("filename") or f.get("name") or "unknown",
                            file_id=f.get("file_id") or f.get("id"),
                            size=f.get("size"),
                        )
                    )
                out.files = items
            if "message" in j and isinstance(j["message"], str):
                out.message = j["message"]
            return out

        # 未知结构，直接文本兜底
        return UploadResponse(ok=True, message=str(j))

    # ---------------- 基于文件的对话补全 ----------------
    def file_completions(
        self,
        model: Optional[str],
        messages: List[Dict[str, str]],
        *,
        doc_ids: Optional[List[str]] = None,
        session_id: Optional[str] = None,
        temperature: Optional[float] = None,
        top_p: Optional[float] = None,
        max_tokens: Optional[int] = None,
        stop: Optional[List[str]] = None,
        extra: Optional[Dict[str, Any]] = None,
    ) -> ChatResponse:
        """
        调用 /v1/file/completions
        - messages: OpenAI 形状 [{"role":"user","content":"..."}]
        - doc_ids/session_id: 将上传得到的文档/会话信息传给后端
        """
        req = FileChatCompletionRequest(
            model=model or self.default_model,
            messages=[ChatMessage(**m) for m in messages],
            doc_ids=doc_ids,
            session_id=session_id,
            temperature=temperature,
            top_p=top_p,
            max_tokens=max_tokens,
            stop=stop,
            extra=extra,
        )
        payload = req.to_payload()

        url = self.cfg.get_completions_url()
        resp = self._session.post(
            url,
            data=json.dumps(payload, ensure_ascii=False),
            headers=self._headers_json(),
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

        # 2) 简化形状：{"text":"..."} → 组装为 ChatResponse
        if isinstance(j, dict) and "text" in j:
            text = j["text"]
            return ChatResponse(
                model=j.get("model", req.model),
                choices=[ChatChoice(index=0, message=ChatMessage(role="assistant", content=text))],
                usage=None,
            )

        # 3) 兜底一些常见结构
        if "data" in j and isinstance(j["data"], list) and j["data"]:
            first = j["data"][0]
            text = first.get("text") or first.get("generated_text") or ""
            return ChatResponse(
                model=j.get("model", req.model),
                choices=[ChatChoice(index=0, message=ChatMessage(role="assistant", content=text))],
                usage=None,
            )

        raise ValueError(f"Unrecognized file chat response shape: {j}")
