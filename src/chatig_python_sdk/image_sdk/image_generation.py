import httpx
import time
from typing import Optional, List
from pydantic import BaseModel, Field

class ImageGenerationRequest(BaseModel):
    """图片生成请求模型 - 与chatig后端保持一致"""
    prompt: str = Field(..., description="生成图片的提示词")
    model: str = Field("sdxl-turbo", description="模型名称")
    n: Optional[int] = Field(1, description="生成图片数量")
    size: Optional[str] = Field("1024x1024", description="图片尺寸")
    user: Optional[str] = Field(None, description="用户标识")

class ImageData(BaseModel):
    """图片数据模型"""
    url: str = Field(..., description="图片URL")
    revised_prompt: Optional[str] = Field(None, description="修订后的提示词")

class ImageGenerationResponse(BaseModel):
    """图片生成响应模型 - 与chatig后端保持一致"""
    created: int = Field(..., description="创建时间戳")
    data: List[ImageData] = Field(..., description="图片数据列表")

class ImageGenerationAPI:
    """图片生成API客户端"""
    
    def __init__(self, api_base: str, api_key: str, timeout: int = 30):
        self.api_base = api_base.rstrip("/")
        self.api_key = api_key
        self.timeout = timeout
        self.session = httpx.Client(
            base_url=self.api_base,
            headers={
                "Authorization": f"Bearer {self.api_key}",
                "Content-Type": "application/json",
                "User-Agent": "ChatIG-Python-SDK/1.0.0"
            },
            timeout=self.timeout,
            http2=False  # 避免h2依赖问题
        )

    def generate(self, request: ImageGenerationRequest) -> ImageGenerationResponse:
        """
        生成图片
        
        :param request: 图片生成请求
        :return: 图片生成响应
        """
        url = "/images/generations"  # 去掉重复的/v1，因为base_url已经包含了
        
        try:
            # 使用model_dump替代dict（Pydantic v2兼容）
            payload = request.model_dump(exclude_none=True)
            resp = self.session.post(url, json=payload)
            resp.raise_for_status()
            return ImageGenerationResponse(**resp.json())
        except httpx.HTTPStatusError as e:
            if e.response.status_code == 404:
                raise Exception(f"图片生成接口不存在: {e.response.url}")
            elif e.response.status_code == 401:
                raise Exception("API密钥无效")
            elif e.response.status_code == 400:
                error_data = e.response.json()
                raise Exception(f"请求参数错误: {error_data.get('error', '未知错误')}")
            else:
                raise Exception(f"图片生成失败: {e.response.status_code} {e.response.text}")
        except Exception as e:
            raise Exception(f"图片生成请求失败: {str(e)}")

    def close(self):
        """关闭客户端"""
        self.session.close()