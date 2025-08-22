"""
ChatIG Chat Module 配置管理
支持远程VLLM服务配置
"""

import os
from typing import Optional
from pathlib import Path
from dotenv import load_dotenv

# 加载环境变量
load_dotenv()


class ChatConfig:
    """ChatIG 聊天模块配置类"""
    
    def __init__(self):
        # API 配置
        self.api_key: str = os.getenv("CHATIG_API_KEY", "")
        self.api_base: str = os.getenv("CHATIG_API_BASE", "http://localhost:8080")
        
        # 远程VLLM服务配置
        self.vllm_server_url: str = os.getenv("VLLM_SERVER_URL", "http://localhost:8000")
        self.vllm_model_name: str = os.getenv("VLLM_MODEL_NAME", "Qwen/Qwen2.5-7B-Instruct")
        self.vllm_api_key: str = os.getenv("VLLM_API_KEY", "")
        
        # 连接配置
        self.timeout: float = float(os.getenv("CHATIG_TIMEOUT", "60.0"))
        self.max_retries: int = int(os.getenv("CHATIG_MAX_RETRIES", "3"))
        self.retry_delay: float = float(os.getenv("CHATIG_RETRY_DELAY", "1.0"))
        
        # 日志配置
        self.log_level: str = os.getenv("CHATIG_LOG_LEVEL", "INFO")
        self.log_file: str = os.getenv("CHATIG_LOG_FILE", "chatig.log")
        
        # 开发配置
        self.debug: bool = os.getenv("CHATIG_DEBUG", "false").lower() == "true"
        
        # Linux 环境优化配置
        self.http2_enabled: bool = os.getenv("CHATIG_HTTP2", "false").lower() == "true"
        self.connection_pool_size: int = int(os.getenv("CHATIG_POOL_SIZE", "10"))
        self.keep_alive_timeout: float = float(os.getenv("CHATIG_KEEP_ALIVE", "30.0"))
        
        # 缓存配置
        self.cache_enabled: bool = os.getenv("CHATIG_CACHE", "true").lower() == "true"
        self.cache_dir: str = os.getenv("CHATIG_CACHE_DIR", str(Path.home() / ".cache" / "chatig"))
        
        # 会话配置
        self.session_persistence: bool = os.getenv("CHATIG_SESSION_PERSISTENCE", "true").lower() == "true"
        self.session_dir: str = os.getenv("CHATIG_SESSION_DIR", str(Path.home() / ".chatig" / "sessions"))
    
    def validate(self) -> bool:
        """验证配置"""
        if not self.api_key:
            print("⚠️  警告: CHATIG_API_KEY 未设置")
            return False
        
        if not self.api_base:
            print("⚠️  警告: CHATIG_API_BASE 未设置")
            return False
        
        # 验证VLLM配置
        if not self.vllm_server_url:
            print("⚠️  警告: VLLM_SERVER_URL 未设置")
            return False
        
        if not self.vllm_model_name:
            print("⚠️  警告: VLLM_MODEL_NAME 未设置")
            return False
        
        return True
    
    def get_headers(self) -> dict:
        """获取请求头"""
        headers = {
            "Content-Type": "application/json",
            "User-Agent": "ChatIG-Python-SDK/1.0.0 (Linux)",
            "Accept": "application/json",
            "Accept-Encoding": "gzip, deflate",
        }
        
        # 添加API密钥
        if self.api_key:
            headers["Authorization"] = f"Bearer {self.api_key}"
        
        return headers
    
    def get_vllm_headers(self) -> dict:
        """获取VLLM服务请求头"""
        headers = {
            "Content-Type": "application/json",
            "User-Agent": "ChatIG-VLLM-SDK/1.0.0",
            "Accept": "application/json",
        }
        
        # 添加VLLM API密钥
        if self.vllm_api_key:
            headers["Authorization"] = f"Bearer {self.vllm_api_key}"
        
        return headers
    
    def get_client_config(self) -> dict:
        """获取客户端配置"""
        config = {
            "timeout": self.timeout,
        }
        
        if self.http2_enabled:
            config["http2"] = True
        
        return config
    
    def setup_directories(self):
        """设置必要的目录"""
        # 创建缓存目录
        if self.cache_enabled:
            Path(self.cache_dir).mkdir(parents=True, exist_ok=True)
        
        # 创建会话目录
        if self.session_persistence:
            Path(self.session_dir).mkdir(parents=True, exist_ok=True)
    
    def to_dict(self) -> dict:
        """转换为字典"""
        return {
            "api_key": self.api_key[:8] + "..." if self.api_key else "",
            "api_base": self.api_base,
            "vllm_server_url": self.vllm_server_url,
            "vllm_model_name": self.vllm_model_name,
            "vllm_api_key": self.vllm_api_key[:8] + "..." if self.vllm_api_key else "",
            "timeout": self.timeout,
            "max_retries": self.max_retries,
            "retry_delay": self.retry_delay,
            "log_level": self.log_level,
            "debug": self.debug,
            "http2_enabled": self.http2_enabled,
            "cache_enabled": self.cache_enabled,
            "session_persistence": self.session_persistence,
        }
    
    def get_vllm_completion_url(self) -> str:
        """获取VLLM补全API URL"""
        return f"{self.vllm_server_url}/v1/completions"
    
    def get_vllm_chat_url(self) -> str:
        """获取VLLM聊天API URL"""
        return f"{self.vllm_server_url}/v1/chat/completions"
    
    def get_vllm_models_url(self) -> str:
        """获取VLLM模型列表API URL"""
        return f"{self.vllm_server_url}/v1/models"
    
    def get_vllm_health_url(self) -> str:
        """获取VLLM健康检查API URL"""
        return f"{self.vllm_server_url}/health"


class LinuxOptimizedConfig(ChatConfig):
    """Linux 环境优化配置"""
    
    def __init__(self):
        super().__init__()
        
        # Linux 特定优化
        self.use_system_ca_certs: bool = True
        self.enable_compression: bool = True
        self.tcp_keepalive: bool = True
        self.socket_timeout: float = 30.0
        
        # 系统资源限制
        self.max_connections: int = min(int(os.getenv("CHATIG_MAX_CONNECTIONS", "20")), 100)
        self.connection_timeout: float = float(os.getenv("CHATIG_CONNECTION_TIMEOUT", "10.0"))
        
        # 异步优化
        self.async_pool_size: int = int(os.getenv("CHATIG_ASYNC_POOL_SIZE", "20"))
        self.async_timeout: float = float(os.getenv("CHATIG_ASYNC_TIMEOUT", "30.0"))
    
    def get_linux_optimized_headers(self) -> dict:
        """获取Linux优化的请求头"""
        headers = self.get_headers()
        headers.update({
            "Accept-Encoding": "gzip, deflate, br",
            "Connection": "keep-alive",
        })
        return headers
    
    def get_async_config(self) -> dict:
        """获取异步配置"""
        return {
            "timeout": self.async_timeout,
            "max_connections": self.async_pool_size,
            "http2": self.http2_enabled,
            "limits": {
                "max_connections": self.max_connections,
                "max_keepalive_connections": self.max_connections,
                "keepalive_expiry": self.keep_alive_timeout,
            }
        }


# 全局配置实例
config = LinuxOptimizedConfig()


def get_config() -> LinuxOptimizedConfig:
    """获取配置实例"""
    return config


def load_config_from_file(config_file: str) -> LinuxOptimizedConfig:
    """从文件加载配置"""
    if os.path.exists(config_file):
        load_dotenv(config_file)
        return LinuxOptimizedConfig()
    return config


def setup_environment():
    """设置环境"""
    config.setup_directories()
    
    # 设置环境变量
    os.environ.setdefault("PYTHONPATH", os.getcwd())
    
    # Linux 环境优化
    if os.name == "posix":  # Linux/Unix
        # 设置文件描述符限制
        try:
            import resource
            soft, hard = resource.getrlimit(resource.RLIMIT_NOFILE)
            if soft < 1024:
                resource.setrlimit(resource.RLIMIT_NOFILE, (min(1024, hard), hard))
        except ImportError:
            pass
        
        # 设置线程池大小
        os.environ.setdefault("PYTHONTHREADDEBUG", "0")
    
    return config 