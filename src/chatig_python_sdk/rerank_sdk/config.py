"""
Chatig 重排序 SDK 的配置

本模块处理重排序SDK客户端的配置。
"""

import os
from typing import Optional, Dict, Any
from dataclasses import dataclass, field
from urllib.parse import urljoin


@dataclass
class RerankConfig:
    """
    重排序客户端的配置
    
    属性:
        base_url: Chatig API的基础URL
        api_key: 用于认证的API密钥
        timeout: 请求超时时间（秒）
        max_retries: 失败请求的最大重试次数
        retry_delay: 重试之间的延迟时间（秒）
        headers: 请求中包含的附加头部
        verify_ssl: 是否验证SSL证书
        proxy: 代理配置
    """
    
    base_url: str = "http://localhost:8000"
    api_key: Optional[str] = None
    timeout: float = 30.0
    max_retries: int = 3
    retry_delay: float = 1.0
    headers: Dict[str, str] = field(default_factory=dict)
    verify_ssl: bool = True
    proxy: Optional[str] = None
    
    # 远程VLLM服务配置
    vllm_server_url: str = "http://localhost:8000"
    vllm_model_name: str = "Qwen/Qwen2.5-7B-Instruct"
    vllm_api_key: Optional[str] = None
    
    def __post_init__(self):
        """初始化后设置"""
        # 设置默认头部
        if not self.headers:
            self.headers = {
                "Content-Type": "application/json",
                "User-Agent": "Chatig-Rerank-SDK/1.0.0"
            }
        
        # 如果提供了API密钥，添加到头部
        if self.api_key:
            self.headers["Authorization"] = f"Bearer {self.api_key}"
    
    @classmethod
    def from_env(cls) -> "RerankConfig":
        """
        从环境变量创建配置
        
        环境变量:
        - CHATIG_BASE_URL: API的基础URL
        - CHATIG_API_KEY: 用于认证的API密钥
        - CHATIG_TIMEOUT: 请求超时时间（秒）
        - CHATIG_MAX_RETRIES: 最大重试次数
        - CHATIG_RETRY_DELAY: 重试延迟时间
        - CHATIG_VERIFY_SSL: 是否验证SSL（true/false）
        - CHATIG_PROXY: 代理URL
        - VLLM_SERVER_URL: VLLM服务器URL
        - VLLM_MODEL_NAME: VLLM模型名称
        - VLLM_API_KEY: VLLM API密钥
        
        返回:
            RerankConfig实例
        """
        return cls(
            base_url=os.getenv("CHATIG_BASE_URL", "http://localhost:8000"),
            api_key=os.getenv("CHATIG_API_KEY"),
            timeout=float(os.getenv("CHATIG_TIMEOUT", "30.0")),
            max_retries=int(os.getenv("CHATIG_MAX_RETRIES", "3")),
            retry_delay=float(os.getenv("CHATIG_RETRY_DELAY", "1.0")),
            verify_ssl=os.getenv("CHATIG_VERIFY_SSL", "true").lower() == "true",
            proxy=os.getenv("CHATIG_PROXY"),
            vllm_server_url=os.getenv("VLLM_SERVER_URL", "http://localhost:8000"),
            vllm_model_name=os.getenv("VLLM_MODEL_NAME", "Qwen/Qwen2.5-7B-Instruct"),
            vllm_api_key=os.getenv("VLLM_API_KEY")
        )
    
    def get_rerank_url(self, engine_type: str = "std") -> str:
        """
        获取重排序请求的完整URL
        
        参数:
            engine_type: 重排序引擎类型（std, vllm, llamabox）
            
        返回:
            重排序端点的完整URL
        """
        if engine_type == "std":
            return urljoin(self.base_url, "/v1/rerank")
        elif engine_type == "vllm":
            return urljoin(self.base_url, "/v1/rerank/vllm")
        elif engine_type == "llamabox":
            return urljoin(self.base_url, "/v1/rerank/llamabox")
        else:
            raise ValueError(f"Unsupported engine type: {engine_type}")
    
    def get_health_url(self) -> str:
        """
        获取健康检查URL
        
        返回:
            健康检查端点的完整URL
        """
        return urljoin(self.base_url, "/v1/chat/health")
    
    def get_vllm_rerank_url(self) -> str:
        """
        获取VLLM重排序URL
        
        返回:
            VLLM重排序端点的完整URL
        """
        return self._build_safe_url(self.vllm_server_url, "/v1/rerank")
    
    def get_vllm_health_url(self) -> str:
        """
        获取VLLM健康检查URL
        
        返回:
            VLLM健康检查端点的完整URL
        """
        return self._build_safe_url(self.vllm_server_url, "/health")
    
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
    
    def get_vllm_headers(self) -> Dict[str, str]:
        """
        获取VLLM服务请求头
        
        返回:
            VLLM请求头字典
        """
        headers = {
            "Content-Type": "application/json",
            "User-Agent": "ChatIG-Rerank-VLLM-SDK/1.0.0"
        }
        
        if self.vllm_api_key:
            headers["Authorization"] = f"Bearer {self.vllm_api_key}"
        
        return headers
    
    def to_dict(self) -> Dict[str, Any]:
        """
        将配置转换为字典
        
        返回:
            配置的字典表示
        """
        return {
            "base_url": self.base_url,
            "api_key": self.api_key,
            "timeout": self.timeout,
            "max_retries": self.max_retries,
            "retry_delay": self.retry_delay,
            "headers": self.headers.copy(),
            "verify_ssl": self.verify_ssl,
            "proxy": self.proxy,
            "vllm_server_url": self.vllm_server_url,
            "vllm_model_name": self.vllm_model_name,
            "vllm_api_key": self.vllm_api_key
        }
    
    def copy(self) -> "RerankConfig":
        """
        创建配置的副本
        
        返回:
            配置的副本
        """
        return RerankConfig(**self.to_dict())
    
    def update(self, **kwargs) -> None:
        """
        使用新值更新配置
        
        参数:
            **kwargs: 要更新的配置值
        """
        for key, value in kwargs.items():
            if hasattr(self, key):
                setattr(self, key, value)
        
        # 重新运行后初始化以更新头部
        self.__post_init__()


# 默认配置
DEFAULT_CONFIG = RerankConfig()


def get_default_config() -> RerankConfig:
    """
    获取默认配置
    
    返回:
        默认配置实例
    """
    return DEFAULT_CONFIG.copy()


def set_default_config(config: RerankConfig) -> None:
    """
    设置默认配置
    
    参数:
        config: 要设置为默认的配置
    """
    global DEFAULT_CONFIG
    DEFAULT_CONFIG = config.copy() 