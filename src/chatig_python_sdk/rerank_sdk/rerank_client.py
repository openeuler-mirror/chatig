"""
Chatig 重排序 SDK 的主客户端

本模块提供了用于与Chatig重排序API端点交互的
主要RerankClient类。
"""

import json
import time
from typing import List, Optional, Union, Dict, Any
import requests
from requests.adapters import HTTPAdapter
from urllib3.util.retry import Retry

from .models import (
    StdRerankRequest, StdRerankResponse,
    LlamaBoxRerankRequest, LlamaBoxRerankResponse,
    VLLMRerankRequest, VLLMRerankResponse,
    RerankEngineType, create_rerank_request, parse_rerank_response
)
from .exceptions import (
    RerankError, RerankValidationError, RerankConnectionError,
    RerankTimeoutError, handle_http_error
)
from .config import RerankConfig


class RerankClient:
    """
    用于与Chatig重排序API交互的客户端
    
    此客户端支持多种重排序引擎:
    - std: 标准重排序
    - vllm: VLLM重排序
    - llamabox: LlamaBox重排序
    """
    
    def __init__(self, config: Optional[RerankConfig] = None):
        """
        初始化重排序客户端
        
        参数:
            config: 客户端配置。如果为None，使用默认配置。
        """
        self.config = config or RerankConfig()
        self.session = self._create_session()
    
    def _create_session(self) -> requests.Session:
        """
        创建并配置requests会话
        
        返回:
            配置好的requests会话
        """
        session = requests.Session()
        
        # 配置重试策略
        retry_strategy = Retry(
            total=self.config.max_retries,
            backoff_factor=self.config.retry_delay,
            status_forcelist=[429, 500, 502, 503, 504],
            allowed_methods=["POST", "GET"]
        )
        
        adapter = HTTPAdapter(max_retries=retry_strategy)
        session.mount("http://", adapter)
        session.mount("https://", adapter)
        
        # 设置默认头部
        session.headers.update(self.config.headers)
        
        # 如果指定了代理，配置代理
        if self.config.proxy:
            session.proxies = {
                "http": self.config.proxy,
                "https": self.config.proxy
            }
        
        return session
    
    def health_check(self) -> bool:
        """
        检查重排序服务是否健康
        
        返回:
            如果服务健康返回True，否则返回False
        """
        try:
            url = self.config.get_health_url()
            response = self.session.get(
                url,
                timeout=self.config.timeout,
                verify=self.config.verify_ssl
            )
            return response.status_code == 200
        except Exception:
            return False
    
    def rerank(
        self,
        model: str,
        query: str,
        documents: List[str],
        engine_type: Union[str, RerankEngineType] = RerankEngineType.STD,
        top_n: Optional[int] = None,
        **kwargs
    ) -> Union[StdRerankResponse, LlamaBoxRerankResponse, VLLMRerankResponse]:
        """
        执行重排序操作
        
        参数:
            model: 用于重排序的模型名称
            query: 查询文本
            documents: 要重排序的文档列表
            engine_type: 重排序引擎类型（std, vllm, llamabox）
            top_n: 返回的顶部结果数量
            **kwargs: 特定引擎类型的附加参数
            
        返回:
            重排序响应对象
            
        抛出:
            RerankValidationError: 如果请求验证失败
            RerankConnectionError: 如果连接失败
            RerankTimeoutError: 如果请求超时
            RerankError: 其他错误
        """
        # 如果需要，将字符串转换为枚举
        if isinstance(engine_type, str):
            engine_type = RerankEngineType(engine_type)
        
        # 创建请求对象
        request_data = create_rerank_request(
            engine_type=engine_type,
            model=model,
            query=query,
            documents=documents,
            top_n=top_n,
            **kwargs
        )
        
        # 验证请求
        request_data.validate()
        
        # 发送请求
        return self._send_rerank_request(engine_type, request_data)
    
    def rerank_std(
        self,
        model: str,
        query: str,
        documents: List[str],
        top_n: Optional[int] = None,
        parameters: Optional[Dict[str, Any]] = None
    ) -> StdRerankResponse:
        """
        执行标准重排序操作
        
        参数:
            model: 用于重排序的模型名称
            query: 查询文本
            documents: 要重排序的文档列表
            top_n: 返回的顶部结果数量
            parameters: 附加参数
            
        返回:
            标准重排序响应
        """
        return self.rerank(
            model=model,
            query=query,
            documents=documents,
            engine_type=RerankEngineType.STD,
            top_n=top_n,
            parameters=parameters
        )
    
    def rerank_vllm(
        self,
        model: str,
        query: str,
        documents: List[str]
    ) -> VLLMRerankResponse:
        """
        执行VLLM重排序操作
        
        参数:
            model: 用于重排序的模型名称
            query: 查询文本
            documents: 要重排序的文档列表
            
        返回:
            VLLM重排序响应
        """
        return self.rerank(
            model=model,
            query=query,
            documents=documents,
            engine_type=RerankEngineType.VLLM
        )
    
    def rerank_llamabox(
        self,
        model: str,
        query: str,
        documents: List[str],
        top_n: Optional[int] = None
    ) -> LlamaBoxRerankResponse:
        """
        执行LlamaBox重排序操作
        
        参数:
            model: 用于重排序的模型名称
            query: 查询文本
            documents: 要重排序的文档列表
            top_n: 返回的顶部结果数量
            
        返回:
            LlamaBox重排序响应
        """
        return self.rerank(
            model=model,
            query=query,
            documents=documents,
            engine_type=RerankEngineType.LLAMABOX,
            top_n=top_n
        )
    
    def _send_rerank_request(
        self,
        engine_type: RerankEngineType,
        request_data: Union[StdRerankRequest, LlamaBoxRerankRequest, VLLMRerankRequest]
    ) -> Union[StdRerankResponse, LlamaBoxRerankResponse, VLLMRerankResponse]:
        """
        向API发送重排序请求
        
        参数:
            engine_type: 重排序引擎类型
            request_data: 请求数据对象
            
        返回:
            重排序响应对象
            
        抛出:
            RerankError: 如果请求失败
        """
        url = self.config.get_rerank_url(engine_type.value)
        
        try:
            # 将请求数据转换为JSON
            request_json = self._request_to_dict(request_data)
            
            # 发送请求
            response = self.session.post(
                url,
                json=request_json,
                timeout=self.config.timeout,
                verify=self.config.verify_ssl
            )
            
            # 处理响应
            return self._handle_response(response, engine_type)
            
        except requests.exceptions.Timeout:
            raise RerankTimeoutError(
                f"Request timed out after {self.config.timeout} seconds",
                timeout=self.config.timeout
            )
        except requests.exceptions.ConnectionError as e:
            raise RerankConnectionError(
                f"Connection failed: {str(e)}",
                url=url
            )
        except requests.exceptions.RequestException as e:
            raise RerankError(f"Request failed: {str(e)}")
    
    def _request_to_dict(self, request_data: Any) -> Dict[str, Any]:
        """
        将请求对象转换为字典
        
        参数:
            request_data: 请求数据对象
            
        返回:
            请求的字典表示
        """
        if hasattr(request_data, '__dict__'):
            return request_data.__dict__.copy()
        elif hasattr(request_data, 'to_dict'):
            return request_data.to_dict()
        else:
            return json.loads(json.dumps(request_data, default=str))
    
    def _handle_response(
        self,
        response: requests.Response,
        engine_type: RerankEngineType
    ) -> Union[StdRerankResponse, LlamaBoxRerankResponse, VLLMRerankResponse]:
        """
        处理API响应
        
        参数:
            response: HTTP响应对象
            engine_type: 重排序引擎类型
            
        返回:
            解析后的响应对象
            
        抛出:
            RerankError: 如果响应指示错误
        """
        try:
            response_data = response.json()
        except json.JSONDecodeError:
            raise RerankError(f"Invalid JSON response: {response.text}")
        
        # 检查HTTP状态码错误
        if response.status_code >= 400:
            raise handle_http_error(response.status_code, response_data)
        
        # 检查响应内容中的错误（即使HTTP状态码是200）
        if isinstance(response_data, dict):
            if response_data.get('object') == 'error':
                error_message = response_data.get('message', 'Unknown error')
                error_type = response_data.get('type', 'UnknownError')
                error_code = response_data.get('code', 400)
                # 直接抛出RerankError，避免嵌套错误
                raise RerankError(f"VLLM service error: {error_message}")
        
        # 根据引擎类型解析响应
        return parse_rerank_response(engine_type, response_data)
    
    def close(self) -> None:
        """关闭客户端会话"""
        if self.session:
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
        
        返回:
            如果VLLM服务健康返回True，否则返回False
        """
        try:
            url = self.config.get_vllm_health_url()
            headers = self.config.get_vllm_headers()
            response = self.session.get(
                url,
                headers=headers,
                timeout=self.config.timeout,
                verify=self.config.verify_ssl
            )
            return response.status_code == 200
        except Exception:
            return False
    
    def vllm_rerank(
        self,
        query: str,
        documents: List[str],
        model: str = None,
        top_n: Optional[int] = None,
        **kwargs
    ) -> VLLMRerankResponse:
        """
        使用VLLM服务进行重排序
        
        参数:
            query: 查询文本
            documents: 文档列表
            model: 模型名称（可选，使用配置中的默认模型）
            top_n: 返回前N个结果（可选）
            **kwargs: 其他参数
            
        返回:
            VLLM重排序响应
        """
        # 使用配置中的模型名称
        model = model or self.config.vllm_model_name
        
        # 创建VLLM重排序请求
        request_data = VLLMRerankRequest(
            model=model,
            query=query,
            documents=documents
        )
        
        # 验证请求
        request_data.validate()
        
        # 发送请求到VLLM服务
        try:
            url = self.config.get_vllm_rerank_url()
            headers = self.config.get_vllm_headers()
            
            response = self.session.post(
                url,
                json=self._request_to_dict(request_data),
                headers=headers,
                timeout=self.config.timeout,
                verify=self.config.verify_ssl
            )
            
            return self._handle_response(response, RerankEngineType.VLLM)
            
        except UnicodeEncodeError as e:
            # 处理编码错误
            raise RerankError(f"VLLM rerank request failed due to encoding issue: {str(e)}")
        except requests.exceptions.RequestException as e:
            raise RerankConnectionError(f"VLLM rerank request failed: {str(e)}")
        except Exception as e:
            raise RerankError(f"VLLM rerank error: {str(e)}")
    
    def vllm_rerank_direct(
        self,
        query: str,
        documents: List[str],
        model: str = None,
        **kwargs
    ) -> Dict[str, Any]:
        """
        直接调用VLLM服务进行重排序（返回原始响应）
        
        参数:
            query: 查询文本
            documents: 文档列表
            model: 模型名称（可选）
            **kwargs: 其他参数
            
        返回:
            VLLM服务的原始响应
        """
        # 使用配置中的模型名称
        model = model or self.config.vllm_model_name
        
        # 构建请求数据
        request_data = {
            "model": model,
            "query": query,
            "documents": documents
        }
        
        # 添加其他参数
        request_data.update(kwargs)
        
        try:
            url = self.config.get_vllm_rerank_url()
            headers = self.config.get_vllm_headers()
            
            response = self.session.post(
                url,
                json=request_data,
                headers=headers,
                timeout=self.config.timeout,
                verify=self.config.verify_ssl
            )
            
            if response.status_code == 200:
                return response.json()
            else:
                raise RerankError(f"VLLM service error: {response.status_code} - {response.text}")
                
        except UnicodeEncodeError as e:
            # 处理编码错误
            raise RerankError(f"VLLM rerank request failed due to encoding issue: {str(e)}")
        except requests.exceptions.RequestException as e:
            raise RerankConnectionError(f"VLLM rerank request failed: {str(e)}")
        except Exception as e:
            raise RerankError(f"VLLM rerank error: {str(e)}") 