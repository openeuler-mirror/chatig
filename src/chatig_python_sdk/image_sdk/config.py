#!/usr/bin/env python3
"""
图片生成SDK配置文件
管理API密钥和默认设置
"""

import os
from typing import Optional
from dataclasses import dataclass

@dataclass
class APIConfig:
    """API配置类"""
    openai_api_key: Optional[str] = None
    stability_api_key: Optional[str] = None
    midjourney_api_key: Optional[str] = None
    huggingface_api_key: Optional[str] = None
    
    # 默认设置
    default_model: str = "dall-e-3"
    default_size: str = "1024x1024"
    default_quality: str = "standard"
    default_timeout: int = 60
    
    # 本地保存设置
    save_images_locally: bool = True
    default_save_dir: str = "./generated_images"
    
    def __post_init__(self):
        """初始化后加载环境变量"""
        self.openai_api_key = self.openai_api_key or os.getenv("OPENAI_API_KEY")
        self.stability_api_key = self.stability_api_key or os.getenv("STABILITY_API_KEY")
        self.midjourney_api_key = self.midjourney_api_key or os.getenv("MIDJOURNEY_API_KEY")
        self.huggingface_api_key = self.huggingface_api_key or os.getenv("HUGGINGFACE_API_KEY")

class ConfigManager:
    """配置管理器"""
    
    def __init__(self, config_file: Optional[str] = None):
        """
        初始化配置管理器
        
        :param config_file: 配置文件路径，如果为None则使用默认配置
        """
        self.config_file = config_file
        self.config = self._load_config()
    
    def _load_config(self) -> APIConfig:
        """加载配置"""
        if self.config_file and os.path.exists(self.config_file):
            return self._load_from_file()
        else:
            return APIConfig()
    
    def _load_from_file(self) -> APIConfig:
        """从文件加载配置"""
        try:
            import json
            with open(self.config_file, 'r', encoding='utf-8') as f:
                data = json.load(f)
            
            return APIConfig(
                openai_api_key=data.get("openai_api_key"),
                stability_api_key=data.get("stability_api_key"),
                midjourney_api_key=data.get("midjourney_api_key"),
                huggingface_api_key=data.get("huggingface_api_key"),
                default_model=data.get("default_model", "dall-e-3"),
                default_size=data.get("default_size", "1024x1024"),
                default_quality=data.get("default_quality", "standard"),
                default_timeout=data.get("default_timeout", 60),
                save_images_locally=data.get("save_images_locally", True),
                default_save_dir=data.get("default_save_dir", "./generated_images")
            )
        except Exception as e:
            print(f"⚠️ 配置文件加载失败: {e}，使用默认配置")
            return APIConfig()
    
    def save_config(self, config_file: Optional[str] = None):
        """保存配置到文件"""
        config_file = config_file or self.config_file
        if not config_file:
            print("⚠️ 未指定配置文件路径，无法保存")
            return
        
        try:
            import json
            os.makedirs(os.path.dirname(config_file), exist_ok=True)
            
            data = {
                "openai_api_key": self.config.openai_api_key,
                "stability_api_key": self.config.stability_api_key,
                "midjourney_api_key": self.config.midjourney_api_key,
                "huggingface_api_key": self.config.huggingface_api_key,
                "default_model": self.config.default_model,
                "default_size": self.config.default_size,
                "default_quality": self.config.default_quality,
                "default_timeout": self.config.default_timeout,
                "save_images_locally": self.config.save_images_locally,
                "default_save_dir": self.config.default_save_dir
            }
            
            with open(config_file, 'w', encoding='utf-8') as f:
                json.dump(data, f, indent=2, ensure_ascii=False)
            
            print(f"✅ 配置已保存到: {config_file}")
            
        except Exception as e:
            print(f"❌ 配置保存失败: {e}")
    
    def get_api_key(self, provider: str) -> Optional[str]:
        """获取指定提供商的API密钥"""
        provider_map = {
            "openai": self.config.openai_api_key,
            "stability": self.config.stability_api_key,
            "midjourney": self.config.midjourney_api_key,
            "huggingface": self.config.huggingface_api_key
        }
        
        return provider_map.get(provider.lower())
    
    def set_api_key(self, provider: str, api_key: str):
        """设置指定提供商的API密钥"""
        provider_map = {
            "openai": "openai_api_key",
            "stability": "stability_api_key",
            "midjourney": "midjourney_api_key",
            "huggingface": "huggingface_api_key"
        }
        
        if provider.lower() in provider_map:
            setattr(self.config, provider_map[provider.lower()], api_key)
            print(f"✅ {provider} API密钥已设置")
        else:
            print(f"❌ 不支持的提供商: {provider}")
    
    def check_api_keys(self) -> dict:
        """检查所有API密钥状态"""
        status = {}
        
        providers = [
            ("OpenAI", self.config.openai_api_key),
            ("Stability AI", self.config.stability_api_key),
            ("Midjourney", self.config.midjourney_api_key),
            ("Hugging Face", self.config.huggingface_api_key)
        ]
        
        for name, key in providers:
            if key:
                status[name] = "✅ 已设置"
            else:
                status[name] = "❌ 未设置"
        
        return status
    
    def print_status(self):
        """打印配置状态"""
        print("🔍 图片生成SDK配置状态")
        print("=" * 40)
        
        # API密钥状态
        print("\n📋 API密钥状态:")
        status = self.check_api_keys()
        for provider, state in status.items():
            print(f"  {provider}: {state}")
        
        # 默认设置
        print("\n⚙️ 默认设置:")
        print(f"  默认模型: {self.config.default_model}")
        print(f"  默认尺寸: {self.config.default_size}")
        print(f"  默认质量: {self.config.default_quality}")
        print(f"  默认超时: {self.config.default_timeout}秒")
        print(f"  本地保存: {'✅ 启用' if self.config.save_images_locally else '❌ 禁用'}")
        print(f"  保存目录: {self.config.default_save_dir}")
        
        # 环境变量提示
        print("\n💡 环境变量设置:")
        print("  export OPENAI_API_KEY='your_openai_key'")
        print("  export STABILITY_API_KEY='your_stability_key'")
        print("  export MIDJOURNEY_API_KEY='your_midjourney_key'")
        print("  export HUGGINGFACE_API_KEY='your_huggingface_key'")

# 全局配置实例
config_manager = ConfigManager()

def get_config() -> APIConfig:
    """获取全局配置"""
    return config_manager.config

def get_api_key(provider: str) -> Optional[str]:
    """获取API密钥"""
    return config_manager.get_api_key(provider)

def set_api_key(provider: str, api_key: str):
    """设置API密钥"""
    config_manager.set_api_key(provider, api_key)

def check_config() -> dict:
    """检查配置状态"""
    return config_manager.check_api_keys()

def print_config_status():
    """打印配置状态"""
    config_manager.print_status()

# 便捷函数
def create_config_file(config_file: str = "./image_sdk_config.json"):
    """创建配置文件"""
    config_manager.config_file = config_file
    config_manager.save_config(config_file)

def load_config_from_file(config_file: str):
    """从文件加载配置"""
    config_manager.config_file = config_file
    config_manager.config = config_manager._load_config()
