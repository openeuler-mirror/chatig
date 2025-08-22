"""
ChatIG Chat Client - 推理网关聊天客户端
提供与ChatIG推理网关的完整聊天功能接口
"""

import json
import time
import asyncio
import uuid
from typing import List, Optional, Dict, Any, AsyncGenerator, Union
import httpx
from urllib.parse import urljoin

from .models import (
    ChatMessage,
    ChatCompletionRequest,
    ChatCompletionResponse,
    CompletionsResponse,
    CompletionsStreamResponse,
    StreamOptions
)

from .exceptions import (
    ChatIGError,
    AuthenticationError,
    RateLimitError,
    ServerError,
    ValidationError,
    ModelNotFoundError,
    StreamError,
    TimeoutError
)


class ChatClient:
    """
    ChatIG 推理网关聊天客户端
    
    提供与ChatIG推理网关的完整聊天功能接口，支持：
    - 标准聊天完成
    - 流式聊天完成
    - 多模型支持
    - VLLM服务支持
    - 错误处理和重试机制
    """
    
    def __init__(
        self,
        api_key: str = None,
        api_base: str = None,
        timeout: float = None,
        max_retries: int = None,
        retry_delay: float = None,
        vllm_server_url: str = None,
        vllm_model_name: str = None,
        vllm_api_key: str = None,
        **kwargs
    ):
        """
        初始化聊天客户端
        
        :param api_key: API密钥（可选，优先使用环境变量）
        :param api_base: API基础地址（可选，优先使用环境变量）
        :param timeout: 请求超时时间（秒）
        :param max_retries: 最大重试次数
        :param retry_delay: 重试延迟时间（秒）
        :param kwargs: 其他配置参数
        """
        # 获取统一配置
        try:
            from .config import get_config
            config = get_config()
        except ImportError:
            # 如果相对导入失败，尝试绝对导入
            try:
                from config import get_config
                config = get_config()
            except ImportError:
                # 如果都失败，使用默认配置
                config = None
        
        # 使用参数或配置中的值
        if config:
            self.api_key = api_key or config.api_key
            self.api_base = (api_base or config.api_base).rstrip('/')
            self.timeout = timeout or config.timeout
            self.max_retries = max_retries or config.max_retries
            self.retry_delay = retry_delay or config.retry_delay
            
            # VLLM配置
            self.vllm_server_url = vllm_server_url or config.vllm_server_url
            self.vllm_model_name = vllm_model_name or config.vllm_model_name
            self.vllm_api_key = vllm_api_key or config.vllm_api_key
        else:
            # 使用默认值
            self.api_key = api_key or ""
            self.api_base = (api_base or "http://localhost:8080").rstrip('/')
            self.timeout = timeout or 60.0
            self.max_retries = max_retries or 3
            self.retry_delay = retry_delay or 1.0
            
            # VLLM默认配置
            self.vllm_server_url = vllm_server_url or "http://localhost:8000"
            self.vllm_model_name = vllm_model_name or "Qwen/Qwen2.5-7B-Instruct"
            self.vllm_api_key = vllm_api_key or ""
        
        # 保存配置引用
        self.config = config
        
        # 构建API端点
        self.chat_completions_url = urljoin(self.api_base, "/v1/chat/completions")
        self.health_url = urljoin(self.api_base, "/v1/chat/health")
        
        # VLLM API端点 - 使用安全的URL构建方式
        self.vllm_chat_url = self._build_safe_url(self.vllm_server_url, "/v1/chat/completions")
        self.vllm_completion_url = self._build_safe_url(self.vllm_server_url, "/v1/completions")
        self.vllm_models_url = self._build_safe_url(self.vllm_server_url, "/v1/models")
        self.vllm_health_url = self._build_safe_url(self.vllm_server_url, "/health")
        
        # 创建HTTP会话
        self.session = self._create_session()
        
        # 支持的模型系列
        self.supported_models = {
            "Qwen": "Qwen系列模型",
            "GLM": "GLM系列模型", 
            "meta-llama": "Llama系列模型",
            "Bailian": "Bailian系列模型",
            "deepseek-ai": "DeepSeek系列模型"
        }
    
    def _build_safe_url(self, base_url: str, path: str) -> str:
        """
        安全地构建URL，避免编码问题
        
        :param base_url: 基础URL
        :param path: 路径
        :return: 完整的URL
        """
        try:
            # 确保base_url不以斜杠结尾
            base_url = base_url.rstrip('/')
            # 确保path以斜杠开头
            if not path.startswith('/'):
                path = '/' + path
            
            # 使用urljoin，但先确保输入是安全的
            from urllib.parse import urlparse, urlunparse
            
            # 解析base_url
            parsed = urlparse(base_url)
            # 构建新的URL
            new_path = parsed.path.rstrip('/') + path
            
            # 重新构建URL
            safe_url = urlunparse((
                parsed.scheme,
                parsed.netloc,
                new_path,
                parsed.params,
                parsed.query,
                parsed.fragment
            ))
            
            # 确保URL是UTF-8编码
            safe_url = safe_url.encode('utf-8').decode('utf-8')
            
            return safe_url
            
        except Exception as e:
            # 如果解析失败，使用简单的字符串拼接
            base_url = base_url.rstrip('/')
            if not path.startswith('/'):
                path = '/' + path
            result = base_url + path
            # 确保结果是UTF-8编码
            return result.encode('utf-8').decode('utf-8')
    
    def _create_session(self) -> httpx.Client:
        """创建HTTP会话"""
        # 使用配置中的优化设置
        if self.config and hasattr(self.config, 'get_linux_optimized_headers'):
            headers = self.config.get_linux_optimized_headers()
        elif self.config:
            headers = self.config.get_headers()
        else:
            # 默认请求头
            headers = {
                "Authorization": f"Bearer {self.api_key}",
                "Content-Type": "application/json",
                "User-Agent": "ChatIG-Python-SDK/1.0.0",
                "Accept": "application/json",
            }
        
        # 更新API密钥（只有当API密钥不为空时才设置）
        if self.api_key:
            headers["Authorization"] = f"Bearer {self.api_key}"
        
        # 获取客户端配置
        if self.config:
            client_config = self.config.get_client_config()
        else:
            client_config = {}
        
        # 从 client_config 中移除 timeout，避免重复
        if 'timeout' in client_config:
            del client_config['timeout']
        
        return httpx.Client(
            headers=headers,
            timeout=self.timeout,
            **client_config
        )
    
    def _handle_response(self, response: httpx.Response) -> Dict[str, Any]:
        """处理API响应 - 对应Rust代码中的错误处理"""
        from .exceptions import handle_api_error, map_http_status_to_exception
        
        try:
            response.raise_for_status()
            return response.json()
        except httpx.HTTPStatusError as e:
            # 尝试解析错误响应
            try:
                error_data = e.response.json()
                raise handle_api_error(error_data, e.response.status_code)
            except (json.JSONDecodeError, ValueError):
                # 如果无法解析JSON，使用状态码映射
                raise map_http_status_to_exception(e.response.status_code, e.response.text)
        except json.JSONDecodeError:
            raise ChatIGError("Invalid JSON response")
        except httpx.TimeoutException:
            raise TimeoutError("Request timeout")
    
    def _request_with_retry(self, payload: Dict[str, Any]) -> Dict[str, Any]:
        """带重试机制的请求"""
        for attempt in range(self.max_retries + 1):
            try:
                response = self.session.post(self.chat_completions_url, json=payload)
                return self._handle_response(response)
            except (httpx.RequestError, ChatIGError) as e:
                if attempt == self.max_retries:
                    raise
                
                delay = self.retry_delay * (2 ** attempt)
                time.sleep(delay)
        
        raise ChatIGError("Max retries exceeded")
    
    def _validate_model(self, model: str) -> None:
        """验证模型格式 - 对应Rust代码中的验证逻辑"""
        from .exceptions import validate_model_format, ModelSeriesNotSupportedError
        from .models import SUPPORTED_MODEL_SERIES
        
        # 验证模型格式
        validate_model_format(model)
        
        # 验证模型系列是否支持
        series = model.split('/')[0]
        if series not in SUPPORTED_MODEL_SERIES:
            raise ModelSeriesNotSupportedError(series)
    
    def health_check(self) -> bool:
        """
        健康检查
        
        :return: 如果服务正常返回True，否则返回False
        """
        try:
            response = self.session.get(self.health_url)
            return response.status_code == 200 and response.text.strip() == "OK"
        except Exception:
            return False
    
    def create_completion(
        self,
        messages: List[ChatMessage],
        model: str = "Qwen/Qwen2.5-7B-Instruct",
        temperature: Optional[float] = None,
        top_p: Optional[int] = None,
        n: Optional[int] = None,
        stream: bool = False,
        stop: Optional[List[str]] = None,
        max_tokens: Optional[int] = None,
        presence_penalty: Optional[int] = None,
        frequency_penalty: Optional[int] = None,
        logit_bias: Optional[int] = None,
        user: Optional[str] = None,
        stream_options: Optional[StreamOptions] = None,
        file_id: Optional[str] = None
    ) -> Union[ChatCompletionResponse, CompletionsResponse]:
        """
        创建聊天完成
        
        :param messages: 消息列表
        :param model: 模型名称（格式：series/model-name）
        :param temperature: 温度参数
        :param top_p: 核采样概率
        :param n: 生成回复数量
        :param stream: 是否流式输出
        :param stop: 停止词列表
        :param max_tokens: 最大token数
        :param presence_penalty: 存在惩罚
        :param frequency_penalty: 频率惩罚
        :param logit_bias: logit偏置
        :param user: 用户标识
        :param stream_options: 流式选项
        :param file_id: 文件ID
        :return: 聊天完成响应
        """
        # 验证模型
        self._validate_model(model)
        
        # 验证消息
        if not messages:
            raise ValidationError("Messages cannot be empty")
        
        # 构建请求
        request_data = {
            "model": model,
            "messages": [msg.dict() if hasattr(msg, 'dict') else msg for msg in messages],
            "stream": stream
        }
        
        # 添加可选参数
        if temperature is not None:
            request_data["temperature"] = temperature
        if top_p is not None:
            request_data["top_p"] = top_p
        if n is not None:
            request_data["n"] = n
        if stop is not None:
            request_data["stop"] = stop
        if max_tokens is not None:
            request_data["max_tokens"] = max_tokens
        if presence_penalty is not None:
            request_data["presence_penalty"] = presence_penalty
        if frequency_penalty is not None:
            request_data["frequency_penalty"] = frequency_penalty
        if logit_bias is not None:
            request_data["logit_bias"] = logit_bias
        if user is not None:
            request_data["user"] = user
        if stream_options is not None:
            request_data["stream_options"] = stream_options.dict()
        if file_id is not None:
            request_data["file_id"] = file_id
        
        # 发送请求
        response_data = self._request_with_retry(request_data)
        
        # 根据响应类型返回相应的模型
        if "reasoning_content" in response_data.get("choices", [{}])[0].get("message", {}):
            return CompletionsResponse(**response_data)
        else:
            return ChatCompletionResponse(**response_data)
    
    def chat(
        self,
        message: str,
        system_prompt: Optional[str] = None,
        model: str = "Qwen/Qwen2.5-7B-Instruct",
        temperature: float = 0.7,
        max_tokens: int = 1024
    ) -> str:
        """
        简单的聊天接口
        
        :param message: 用户消息
        :param system_prompt: 系统提示词
        :param model: 模型名称
        :param temperature: 温度参数
        :param max_tokens: 最大token数
        :return: AI回复内容
        """
        messages = []
        
        if system_prompt:
            messages.append(ChatMessage.create_system_message(system_prompt))
        
        messages.append(ChatMessage.create_user_message(message))
        
        response = self.create_completion(
            messages=messages,
            model=model,
            temperature=temperature,
            max_tokens=max_tokens
        )
        
        # 根据响应类型提取内容
        if hasattr(response, 'choices') and response.choices:
            return response.choices[0].message.content
        else:
            raise ChatIGError("Invalid response format")
    
    def stream_completion(
        self,
        messages: List[ChatMessage],
        model: str = "Qwen/Qwen2.5-7B-Instruct",
        temperature: Optional[float] = None,
        max_tokens: Optional[int] = None,
        **kwargs
    ) -> AsyncGenerator[str, None]:
        """
        流式聊天完成
        
        :param messages: 消息列表
        :param model: 模型名称
        :param temperature: 温度参数
        :param max_tokens: 最大token数
        :param kwargs: 其他参数
        :return: 异步生成器，产生对话片段
        """
        # 构建流式请求
        request_data = {
            "model": model,
            "messages": [msg.dict() if hasattr(msg, 'dict') else msg for msg in messages],
            "stream": True
        }
        
        if temperature is not None:
            request_data["temperature"] = temperature
        if max_tokens is not None:
            request_data["max_tokens"] = max_tokens
        
        # 添加其他参数
        for key, value in kwargs.items():
            if value is not None:
                request_data[key] = value
        
        async def _stream_request():
            async with httpx.AsyncClient(
                headers=self.session.headers,
                timeout=self.timeout
            ) as async_client:
                async with async_client.stream("POST", self.chat_completions_url, json=request_data) as response:
                    async for chunk in response.aiter_text():
                        if chunk.strip():
                            # 处理SSE格式的数据
                            for line in chunk.split('\n'):
                                if line.startswith('data: '):
                                    data = line[6:]  # 移除 'data: ' 前缀
                                    if data.strip() == '[DONE]':
                                        return
                                    try:
                                        json_data = json.loads(data)
                                        if 'choices' in json_data and json_data['choices']:
                                            choice = json_data['choices'][0]
                                            if 'delta' in choice:
                                                content = choice['delta'].get('content', '')
                                            else:
                                                content = choice.get('message', {}).get('content', '')
                                            if content:
                                                yield content
                                    except json.JSONDecodeError:
                                        continue
        
        return _stream_request()
    
    def chat_stream(
        self,
        message: str,
        system_prompt: Optional[str] = None,
        model: str = "Qwen/Qwen2.5-7B-Instruct",
        temperature: float = 0.7,
        max_tokens: int = 1024
    ) -> AsyncGenerator[str, None]:
        """
        流式聊天接口
        
        :param message: 用户消息
        :param system_prompt: 系统提示词
        :param model: 模型名称
        :param temperature: 温度参数
        :param max_tokens: 最大token数
        :return: 异步生成器，产生对话片段
        """
        messages = []
        
        if system_prompt:
            messages.append(ChatMessage.create_system_message(system_prompt))
        
        messages.append(ChatMessage.create_user_message(message))
        
        return self.stream_completion(
            messages=messages,
            model=model,
            temperature=temperature,
            max_tokens=max_tokens
        )
    
    def get_supported_models(self) -> Dict[str, str]:
        """
        获取支持的模型列表
        
        :return: 支持的模型字典
        """
        return self.supported_models.copy()
    
    def validate_model(self, model: str) -> bool:
        """
        验证模型是否支持
        
        :param model: 模型名称
        :return: 如果支持返回True，否则返回False
        """
        try:
            self._validate_model(model)
            return True
        except (ValidationError, ModelNotFoundError):
            return False
    
    def close(self):
        """关闭客户端"""
        self.session.close()
    
    def __enter__(self):
        """上下文管理器入口"""
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        """上下文管理器出口"""
        self.close()
    
    # VLLM相关方法
    def vllm_health_check(self) -> bool:
        """
        检查VLLM服务健康状态
        
        :return: 如果健康返回True，否则返回False
        """
        try:
            headers = self.config.get_vllm_headers() if self.config else {"Content-Type": "application/json"}
            
            # 确保URL是安全的
            safe_url = self._build_safe_url(self.vllm_server_url, "/health")
            
            # 确保所有字符串都是UTF-8编码
            safe_url = safe_url.encode('utf-8').decode('utf-8')
            
            response = self.session.get(safe_url, headers=headers, timeout=10)
            return response.status_code == 200
        except Exception:
            return False
    
    def vllm_create_completion(
        self,
        messages: List[ChatMessage],
        model: str = None,
        temperature: Optional[float] = None,
        max_tokens: Optional[int] = None,
        stream: bool = False,
        **kwargs
    ) -> Union[ChatCompletionResponse, CompletionsResponse]:
        """
        使用VLLM服务创建聊天完成
        
        :param messages: 聊天消息列表
        :param model: 模型名称（可选，使用配置中的默认模型）
        :param temperature: 温度参数
        :param max_tokens: 最大令牌数
        :param stream: 是否流式输出
        :param kwargs: 其他参数
        :return: 聊天完成响应
        """
        # 使用配置中的模型名称
        model = model or self.vllm_model_name
        
        # 构建VLLM请求
        request_data = {
            "model": model,
            "messages": [msg.dict() if hasattr(msg, 'dict') else msg for msg in messages],
            "stream": stream
        }
        
        # 添加可选参数
        if temperature is not None:
            request_data["temperature"] = temperature
        if max_tokens is not None:
            request_data["max_tokens"] = max_tokens
        
        # 添加其他参数
        for key, value in kwargs.items():
            if value is not None:
                request_data[key] = value
        
        # 发送请求到VLLM服务
        headers = self.config.get_vllm_headers() if self.config else {"Content-Type": "application/json"}
        
        try:
            # 确保URL是安全的
            safe_url = self._build_safe_url(self.vllm_server_url, "/v1/chat/completions")
            
            # 确保所有字符串都是UTF-8编码
            safe_url = safe_url.encode('utf-8').decode('utf-8')
            
            response = self.session.post(
                safe_url,
                json=request_data,
                headers=headers,
                timeout=self.timeout
            )
            
            if response.status_code == 200:
                data = response.json()
                return ChatCompletionResponse(**data)
            else:
                raise ServerError(f"VLLM service error: {response.status_code} - {response.text}")
                
        except UnicodeEncodeError as e:
            # 处理编码错误
            raise ChatIGError(f"VLLM request failed due to encoding issue: {str(e)}")
        except Exception as e:
            raise ChatIGError(f"VLLM request failed: {str(e)}")
    
    def vllm_chat(
        self,
        message: str,
        system_prompt: Optional[str] = None,
        model: str = None,
        temperature: float = 0.7,
        max_tokens: int = 1024
    ) -> str:
        """
        使用VLLM服务进行简单聊天
        
        :param message: 用户消息
        :param system_prompt: 系统提示（可选）
        :param model: 模型名称（可选）
        :param temperature: 温度参数
        :param max_tokens: 最大令牌数
        :return: 模型回复
        """
        messages = []
        
        if system_prompt:
            messages.append(ChatMessage(role="system", content=system_prompt))
        
        messages.append(ChatMessage(role="user", content=message))
        
        response = self.vllm_create_completion(
            messages=messages,
            model=model,
            temperature=temperature,
            max_tokens=max_tokens
        )
        
        return response.choices[0].message.content
    
    def get_vllm_models(self) -> Dict[str, Any]:
        """
        获取VLLM服务支持的模型列表
        
        :return: 模型信息字典
        """
        try:
            headers = self.config.get_vllm_headers() if self.config else {"Content-Type": "application/json"}
            
            # 确保URL是安全的
            safe_url = self._build_safe_url(self.vllm_server_url, "/v1/models")
            
            # 确保所有字符串都是UTF-8编码
            safe_url = safe_url.encode('utf-8').decode('utf-8')
            
            response = self.session.get(safe_url, headers=headers, timeout=10)
            
            if response.status_code == 200:
                return response.json()
            else:
                raise ServerError(f"Failed to get VLLM models list: {response.status_code}")
                
        except UnicodeEncodeError as e:
            # 处理编码错误
            raise ChatIGError(f"VLLM models list request failed due to encoding issue: {str(e)}")
        except Exception as e:
            raise ChatIGError(f"VLLM models list request failed: {str(e)}")


class AsyncChatClient(ChatClient):
    """异步聊天客户端"""
    
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.session = self._create_async_session()
    
    def _create_async_session(self) -> httpx.AsyncClient:
        """创建异步HTTP会话"""
        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json",
            "User-Agent": "ChatIG-Python-SDK/1.0.0"
        }
        
        return httpx.AsyncClient(
            headers=headers,
            timeout=self.timeout,
            http2=True
        )
    
    async def _request_with_retry(self, payload: Dict[str, Any]) -> Dict[str, Any]:
        """带重试机制的异步请求"""
        for attempt in range(self.max_retries + 1):
            try:
                response = await self.session.post(self.chat_completions_url, json=payload)
                return self._handle_response(response)
            except (httpx.RequestError, ChatIGError) as e:
                if attempt == self.max_retries:
                    raise
                
                delay = self.retry_delay * (2 ** attempt)
                await asyncio.sleep(delay)
        
        raise ChatIGError("Max retries exceeded")
    
    async def create_completion(
        self,
        messages: List[ChatMessage],
        model: str = "Qwen/Qwen2.5-7B-Instruct",
        temperature: Optional[float] = None,
        top_p: Optional[int] = None,
        n: Optional[int] = None,
        stream: bool = False,
        stop: Optional[List[str]] = None,
        max_tokens: Optional[int] = None,
        presence_penalty: Optional[int] = None,
        frequency_penalty: Optional[int] = None,
        logit_bias: Optional[int] = None,
        user: Optional[str] = None,
        stream_options: Optional[StreamOptions] = None,
        file_id: Optional[str] = None
    ) -> Union[ChatCompletionResponse, CompletionsResponse]:
        """异步创建聊天完成"""
        # 验证模型
        self._validate_model(model)
        
        # 验证消息
        if not messages:
            raise ValidationError("Messages cannot be empty")
        
        # 构建请求
        request_data = {
            "model": model,
            "messages": [msg.model_dump() for msg in messages],
            "stream": stream
        }
        
        # 添加可选参数
        if temperature is not None:
            request_data["temperature"] = temperature
        if top_p is not None:
            request_data["top_p"] = top_p
        if n is not None:
            request_data["n"] = n
        if stop is not None:
            request_data["stop"] = stop
        if max_tokens is not None:
            request_data["max_tokens"] = max_tokens
        if presence_penalty is not None:
            request_data["presence_penalty"] = presence_penalty
        if frequency_penalty is not None:
            request_data["frequency_penalty"] = frequency_penalty
        if logit_bias is not None:
            request_data["logit_bias"] = logit_bias
        if user is not None:
            request_data["user"] = user
        if stream_options is not None:
            request_data["stream_options"] = stream_options.model_dump()
        if file_id is not None:
            request_data["file_id"] = file_id
        
        # 发送请求
        response_data = await self._request_with_retry(request_data)
        
        # 根据响应类型返回相应的模型
        if "reasoning_content" in response_data.get("choices", [{}])[0].get("message", {}):
            return CompletionsResponse(**response_data)
        else:
            return ChatCompletionResponse(**response_data)
    
    async def chat(
        self,
        message: str,
        system_prompt: Optional[str] = None,
        model: str = "Qwen/Qwen2.5-7B-Instruct",
        temperature: float = 0.7,
        max_tokens: int = 1024
    ) -> str:
        """异步简单的聊天接口"""
        messages = []
        
        if system_prompt:
            messages.append(ChatMessage.create_system_message(system_prompt))
        
        messages.append(ChatMessage.create_user_message(message))
        
        response = await self.create_completion(
            messages=messages,
            model=model,
            temperature=temperature,
            max_tokens=max_tokens
        )
        
        # 根据响应类型提取内容
        if hasattr(response, 'choices') and response.choices:
            return response.choices[0].message.content
        else:
            raise ChatIGError("Invalid response format")
    
    async def close(self):
        """关闭异步客户端"""
        await self.session.aclose()
    
    async def __aenter__(self):
        """异步上下文管理器入口"""
        return self
    
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        """异步上下文管理器出口"""
        await self.close() 