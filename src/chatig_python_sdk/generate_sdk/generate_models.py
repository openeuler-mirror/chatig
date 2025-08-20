# generate_sdk/generate_models.py
from __future__ import annotations
from typing import List, Optional, Literal, Any, Dict
from pydantic import BaseModel, Field, ConfigDict, field_validator

# ---------- Chat 相关模型（OpenAI 形状优先） ----------

class ChatMessage(BaseModel):
    role: Literal["system", "user", "assistant", "tool"] = "user"
    content: str

class ChatDelta(BaseModel):
    role: Optional[Literal["assistant"]] = None
    content: Optional[str] = None

class ChatChoice(BaseModel):
    index: int = 0
    finish_reason: Optional[str] = None
    message: Optional[ChatMessage] = None   # 非流式
    delta: Optional[ChatDelta] = None       # 流式

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

# ---------- 标准化请求体（兼容 OpenAI & 简化形状） ----------

class StdGenerateRequest(BaseModel):
    """
    推荐传入 OpenAI 形状: {model, messages, temperature?, top_p?, max_tokens?, stream?}
    也支持简化形状: {model, prompt: "..."} —— 自动转 messages=[{role:"user", content: prompt}]
    """
    model: str
    messages: Optional[List[ChatMessage]] = None
    prompt: Optional[str] = None

    temperature: Optional[float] = None
    top_p: Optional[float] = None
    max_tokens: Optional[int] = None
    stream: Optional[bool] = None
    stop: Optional[List[str]] = None
    extra: Optional[Dict[str, Any]] = None

    model_config = ConfigDict(extra="ignore")

    @field_validator("model")
    @classmethod
    def _model_non_empty(cls, v: str) -> str:
        if not isinstance(v, str) or v.strip() == "":
            raise ValueError("model 不能为空")
        return v

    def to_payload(self) -> Dict[str, Any]:
        if not self.messages and self.prompt:
            msgs = [ChatMessage(role="user", content=self.prompt)]
        else:
            msgs = self.messages

        payload: Dict[str, Any] = {
            "model": self.model,
            "messages": [m.model_dump() for m in msgs] if msgs else None,
        }
        if self.temperature is not None:
            payload["temperature"] = self.temperature
        if self.top_p is not None:
            payload["top_p"] = self.top_p
        if self.max_tokens is not None:
            payload["max_tokens"] = self.max_tokens
        if self.stream is not None:
            payload["stream"] = self.stream
        if self.stop is not None:
            payload["stop"] = self.stop
        if self.extra:
            payload.update(self.extra)
        return {k: v for k, v in payload.items() if v is not None}

# ---------- 常见简化返回形状（如 {text:"..."}） ----------

class SimpleTextResponse(BaseModel):
    text: str
    model: Optional[str] = None
    usage: Optional[ChatUsage] = None
    model_config = ConfigDict(extra="ignore")
