"""
ChatIG Chat Module 数据模型
对应Rust代码中的请求和响应结构体
"""

from typing import List, Optional, Dict, Any, Union
from pydantic import BaseModel, Field, validator
from datetime import datetime


class Message(BaseModel):
    """聊天消息模型 - 对应Rust中的Message"""
    role: str = Field(..., description="消息角色: system|user|assistant")
    content: str = Field(..., description="消息内容")
    
    @validator('role')
    def validate_role(cls, v):
        valid_roles = ['system', 'user', 'assistant']
        if v not in valid_roles:
            raise ValueError(f'role must be one of {valid_roles}')
        return v


class StreamOptions(BaseModel):
    """流式选项 - 对应Rust中的StreamOptions"""
    include_usage: bool = Field(False, description="是否包含使用统计")


class ChatCompletionRequest(BaseModel):
    """聊天完成请求模型 - 对应Rust中的ChatCompletionRequest"""
    model: str = Field(..., description="模型名称 (Required)")
    messages: List[Message] = Field(..., description="消息列表 (Required)")
    temperature: Optional[float] = Field(None, ge=0.0, le=2.0, description="控制生成文本的创造性")
    top_p: Optional[float] = Field(None, ge=0.0, le=1.0, description="核采样概率，temperature的替代方法")
    n: Optional[int] = Field(None, ge=1, le=10, description="生成的回复数量")
    stream: Optional[bool] = Field(None, description="是否启用流式响应")
    stop: Optional[List[str]] = Field(None, description="停止生成的字符串列表")
    max_tokens: Optional[int] = Field(None, ge=1, description="每次请求生成的最大token数")
    presence_penalty: Optional[float] = Field(None, ge=-2.0, le=2.0, description="鼓励模型谈论新话题，范围-2.0到2.0")
    frequency_penalty: Optional[float] = Field(None, ge=-2.0, le=2.0, description="控制重复token的可能性，范围-2.0到2.0")
    logit_bias: Optional[int] = Field(None, ge=-100, le=100, description="调整特定token出现的概率")
    user: Optional[str] = Field(None, description="用户ID，用于识别请求来源")
    stream_options: Optional[StreamOptions] = Field(None, description="流式选项")
    file_id: Optional[str] = Field(None, description="文件ID")
    
    @validator('messages')
    def validate_messages(cls, v):
        if not v:
            raise ValueError('messages cannot be empty')
        return v
    
    @validator('model')
    def validate_model(cls, v):
        if not v:
            raise ValueError('model cannot be empty')
        # 验证模型格式: series/model-name
        if '/' not in v:
            raise ValueError('model must be in format "series/model-name"')
        return v


# ==================== Chat Completion Response Models ====================

class AssistantMessage(BaseModel):
    """助手消息模型 - 对应Rust中的AssistantMessage"""
    role: str = Field("assistant", description="消息角色")
    content: str = Field(..., description="消息内容")


class Choice(BaseModel):
    """选择模型 - 对应Rust中的Choice"""
    message: AssistantMessage = Field(..., description="助手消息")
    finish_reason: Optional[str] = Field(None, description="生成停止的原因")
    index: int = Field(..., description="选择索引")


class Usage(BaseModel):
    """使用统计模型 - 对应Rust中的Usage"""
    prompt_tokens: int = Field(..., description="提示token数")
    completion_tokens: Optional[int] = Field(None, description="完成token数")
    total_tokens: int = Field(..., description="总token数")


class ChatCompletionResponse(BaseModel):
    """聊天完成响应模型 - 对应Rust中的ChatCompletionResponse"""
    id: str = Field(..., description="每个生成响应的唯一标识符")
    object: str = Field("chat.completion", description="响应对象类型")
    created: int = Field(..., description="响应生成的时间戳")
    model: str = Field(..., description="使用的模型名称")
    usage: Usage = Field(..., description="token使用情况")
    choices: List[Choice] = Field(..., description="返回的生成文本选项列表")


# ==================== Completions API 相关模型 ====================

class PromptTokensDetails(BaseModel):
    """提示token详情 - 对应Rust中的PromptTokensDetails"""
    cached_tokens: int = Field(..., description="缓存的token数")


class CompletionsAssistantMessage(BaseModel):
    """Completions API 助手消息模型 - 对应Rust中的CompletionsAssistantMessage"""
    role: str = Field("assistant", description="助手角色")
    reasoning_content: Optional[str] = Field(None, description="推理内容")
    content: str = Field(..., description="消息内容")
    refusal: Optional[str] = Field(None, description="拒绝信息")
    tool_calls: Optional[List[str]] = Field(None, description="工具调用")


class CompletionsChoice(BaseModel):
    """Completions API 选择模型 - 对应Rust中的CompletionsChoice"""
    index: int = Field(..., description="完成索引")
    message: CompletionsAssistantMessage = Field(..., description="助手消息")
    logprobs: Optional[str] = Field(None, description="log概率")
    finish_reason: str = Field(..., description="完成原因")
    stop_reason: Optional[str] = Field(None, description="停止原因")


class CompletionsUsage(BaseModel):
    """Completions API 使用统计模型 - 对应Rust中的CompletionsUsage"""
    completion_tokens: int = Field(..., description="完成token数")
    prompt_tokens: int = Field(..., description="提示token数")
    total_tokens: int = Field(..., description="总token数")
    prompt_tokens_details: Optional[PromptTokensDetails] = Field(None, description="提示token详情")


class CompletionsResponse(BaseModel):
    """Completions API 响应模型 - 对应Rust中的CompletionsResponse"""
    id: str = Field(..., description="每个生成响应的唯一标识符")
    object: str = Field("chat.completion", description="响应对象类型")
    created: int = Field(..., description="响应生成的时间戳")
    model: str = Field(..., description="使用的模型名称")
    choices: List[CompletionsChoice] = Field(..., description="返回的生成文本选项列表")
    usage: CompletionsUsage = Field(..., description="请求的使用统计")
    system_fingerprint: Optional[str] = Field(None, description="用于请求的系统指纹")
    prompt_logprobs: Optional[str] = Field(None, description="提示的log概率")


# ==================== Stream Response Models ====================

class CompletionsDelta(BaseModel):
    """流式响应增量模型 - 对应Rust中的CompletionsDelta"""
    role: Optional[str] = Field(None, description="角色")
    content: Optional[str] = Field(None, description="内容")
    refusal: Optional[str] = Field(None, description="拒绝信息")
    function_call: Optional[str] = Field(None, description="函数调用")
    tool_calls: Optional[List[str]] = Field(None, description="工具调用")


class CompletionsStreamChoice(BaseModel):
    """流式响应选择模型 - 对应Rust中的CompletionsStreamChoice"""
    finish_reason: Optional[str] = Field(None, description="完成原因")
    index: int = Field(..., description="选择索引")
    logprobs: Optional[str] = Field(None, description="log概率")
    delta: CompletionsDelta = Field(..., description="增量内容")
    stop_reason: Optional[str] = Field(None, description="停止原因")


class CompletionsStreamResponse(BaseModel):
    """流式响应模型 - 对应Rust中的CompletionsStreamResponse"""
    id: str = Field(..., description="每个生成响应的唯一标识符")
    choices: List[CompletionsStreamChoice] = Field(..., description="返回的生成文本选项列表")
    created: int = Field(..., description="响应生成的时间戳")
    model: str = Field(..., description="使用的模型名称")
    object: str = Field("chat.completion.chunk", description="响应对象类型")
    system_fingerprint: Optional[str] = Field(None, description="用于请求的系统指纹")
    usage: Optional[CompletionsUsage] = Field(None, description="请求的使用统计")


# ==================== 扩展功能 ====================

class ChatMessage(Message):
    """扩展的聊天消息模型，提供便捷方法"""
    
    @classmethod
    def create_system_message(cls, content: str) -> 'ChatMessage':
        """创建系统消息"""
        return cls(role="system", content=content)
    
    @classmethod
    def create_user_message(cls, content: str) -> 'ChatMessage':
        """创建用户消息"""
        return cls(role="user", content=content)
    
    @classmethod
    def create_assistant_message(cls, content: str) -> 'ChatMessage':
        """创建助手消息"""
        return cls(role="assistant", content=content)
    
    @classmethod
    def create_conversation(cls, messages: List[Dict[str, str]]) -> List['ChatMessage']:
        """从字典列表创建对话"""
        return [cls(role=msg['role'], content=msg['content']) for msg in messages]


# ==================== 错误响应模型 ====================

class ErrorResponse(BaseModel):
    """错误响应模型 - 对应Rust中的ErrorResponse"""
    error: str = Field(..., description="错误信息")


# ==================== 支持的模型系列 ====================

SUPPORTED_MODEL_SERIES = {
    "Qwen": "Qwen系列模型",
    "GLM": "GLM系列模型", 
    "meta-llama": "Meta Llama系列模型",
    "Bailian": "Bailian系列模型",
    "deepseek-ai": "DeepSeek系列模型"
}

def validate_model_series(model_name: str) -> bool:
    """验证模型系列是否支持"""
    if '/' not in model_name:
        return False
    series = model_name.split('/')[0]
    return series in SUPPORTED_MODEL_SERIES

def get_model_series(model_name: str) -> Optional[str]:
    """获取模型系列"""
    if '/' not in model_name:
        return None
    return model_name.split('/')[0]

def get_model_name(model_name: str) -> Optional[str]:
    """获取具体模型名称"""
    if '/' not in model_name:
        return None
    return model_name.split('/')[1] 