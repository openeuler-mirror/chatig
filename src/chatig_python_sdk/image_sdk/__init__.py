from .image_generation import ImageGenerationAPI, ImageGenerationRequest, ImageGenerationResponse
# from .text_generation import QwenTextGenerationAPI, create_qwen_client, quick_text_generation
from .online_image_generation import (
    OnlineImageGenerationAPI, 
    ImageModelType, 
    ImageSize,
    create_openai_client,
    create_stability_client,
    quick_image_generation
)
from .config import (
    get_config,
    get_api_key,
    set_api_key,
    check_config,
    print_config_status,
    create_config_file,
    load_config_from_file
)

class ImageAPI:
    def __init__(self, config):
        pass

    def recognize(self, image_path: str) -> str:
        pass

# 导出文字生成相关功能
__all__ = [
    'ImageGenerationAPI', 
    'ImageGenerationRequest', 
    'ImageGenerationResponse',
    'QwenTextGenerationAPI',
    'create_qwen_client',
    'quick_text_generation',
    # 在线图片生成
    'OnlineImageGenerationAPI',
    'ImageModelType',
    'ImageSize',
    'create_openai_client',
    'create_stability_client',
    'quick_image_generation',
    # 配置管理
    'get_config',
    'get_api_key',
    'set_api_key',
    'check_config',
    'print_config_status',
    'create_config_file',
    'load_config_from_file'
]