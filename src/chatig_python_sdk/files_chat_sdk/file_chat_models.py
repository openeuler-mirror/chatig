# file_chat_sdk/file_chat_models.py
from __future__ import annotations
from typing import List, Optional, Literal, Any, Dict
from pydantic import BaseModel, Field, ConfigDict, field_validator

# ---------- Chat 形状（OpenAI 兼容） ----------

class ChatMessage(BaseModel):
    role: Literal["system", "user", "assistant", "tool"] = "user"
    content: str

class ChatChoice(BaseModel):
    index: int = 0
    finish_reason: Optional[str] = None
    message: Optional[ChatMessage] = None

class ChatUsage(BaseModel):
    prompt_tokens: Optional[int] = 0
    completion_tokens: Optional[int] = 0
    total_tokens: Optional[int] = 0

class ChatResponse(BaseModel):
    object: Literal["chat.completion"] = "chat.completion"
    model: str
    choices: List[ChatChoice]
    usage: Optional[ChatUsage] = None
    model_config = ConfigDict(extra="ignore")

# ---------- file_chat 请求体（与后端 ChatCompletionRequest 对齐/宽松） ----------

class FileChatCompletionRequest(BaseModel):
    """
    与后端 ChatCompletionRequest 对齐（宽松）：
      - model: "chatchat" 等
      - messages: OpenAI 形状
      - doc_ids / session_id / extra: 可选拓展，方便对接文件上下文
    """
    model: str
    messages: List[ChatMessage]

    # 可能的扩展字段（根据你们的 UploadForm/控制器需要自由扩展）
    doc_ids: Optional[List[str]] = None   # 上传接口返回的文档/会话 ID
    session_id: Optional[str] = None      # 会话标识（如有）
    extra: Optional[Dict[str, Any]] = None

    temperature: Optional[float] = None
    top_p: Optional[float] = None
    max_tokens: Optional[int] = None
    stop: Optional[List[str]] = None

    model_config = ConfigDict(extra="ignore")

    @field_validator("model")
    @classmethod
    def _model_non_empty(cls, v: str) -> str:
        if not isinstance(v, str) or v.strip() == "":
            raise ValueError("model 不能为空")
        return v

    @field_validator("messages")
    @classmethod
    def _messages_non_empty(cls, v: List[ChatMessage]) -> List[ChatMessage]:
        if not isinstance(v, list) or len(v) == 0:
            raise ValueError("messages 不能为空")
        return v

    def to_payload(self) -> Dict[str, Any]:
        payload: Dict[str, Any] = {
            "model": self.model,
            "messages": [m.model_dump() for m in self.messages],
        }
        if self.doc_ids is not None:
            payload["doc_ids"] = self.doc_ids
        if self.session_id is not None:
            payload["session_id"] = self.session_id
        if self.temperature is not None:
            payload["temperature"] = self.temperature
        if self.top_p is not None:
            payload["top_p"] = self.top_p
        if self.max_tokens is not None:
            payload["max_tokens"] = self.max_tokens
        if self.stop is not None:
            payload["stop"] = self.stop
        if self.extra:
            payload.update(self.extra)
        return payload

# ---------- 上传返回（根据后端实际返回适配） ----------

class UploadFileInfo(BaseModel):
    filename: str
    file_id: Optional[str] = None     # 如果后端生成了 ID
    size: Optional[int] = None
    model_config = ConfigDict(extra="ignore")

class UploadResponse(BaseModel):
    ok: bool = True
    files: Optional[List[UploadFileInfo]] = None
    doc_ids: Optional[List[str]] = None    # 如果后端返回一个或多个 doc/session id
    message: Optional[str] = None
    model_config = ConfigDict(extra="ignore")
