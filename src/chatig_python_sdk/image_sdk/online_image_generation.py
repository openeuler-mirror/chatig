import httpx
import time
import base64
import os
from typing import Optional, List, Dict, Any, Union
from pydantic import BaseModel, Field
from enum import Enum

class ImageModelType(str, Enum):
    """支持的图片生成模型类型"""
    OPENAI_DALLE3 = "dall-e-3"
    OPENAI_DALLE2 = "dall-e-2"
    STABILITY_SDXL = "stability-sdxl"
    STABILITY_SD3 = "stability-sd3"
    MIDJOURNEY = "midjourney"
    HUGGINGFACE = "huggingface"

class ImageSize(str, Enum):
    """支持的图片尺寸"""
    SMALL = "256x256"
    MEDIUM = "512x512"
    LARGE = "1024x1024"
    WIDE = "1792x1024"
    TALL = "1024x1792"

class OnlineImageGenerationRequest(BaseModel):
    """在线图片生成请求模型"""
    prompt: str = Field(..., description="生成图片的提示词")
    model: ImageModelType = Field(ImageModelType.OPENAI_DALLE3, description="模型类型")
    size: ImageSize = Field(ImageSize.LARGE, description="图片尺寸")
    n: Optional[int] = Field(1, description="生成图片数量")
    quality: Optional[str] = Field("standard", description="图片质量")
    style: Optional[str] = Field(None, description="图片风格")
    api_key: Optional[str] = Field(None, description="API密钥")
    api_base: Optional[str] = Field(None, description="API基础地址")

class OnlineImageData(BaseModel):
    """在线图片数据模型"""
    url: Optional[str] = Field(None, description="图片URL")
    b64_json: Optional[str] = Field(None, description="Base64编码的图片数据")
    revised_prompt: Optional[str] = Field(None, description="修订后的提示词")
    local_path: Optional[str] = Field(None, description="本地保存路径")

class OnlineImageGenerationResponse(BaseModel):
    """在线图片生成响应模型"""
    created: int = Field(..., description="创建时间戳")
    data: List[OnlineImageData] = Field(..., description="图片数据列表")
    model: str = Field(..., description="使用的模型")
    provider: str = Field(..., description="服务提供商")

class OnlineImageGenerationAPI:
    """在线图片生成API客户端"""
    
    def __init__(self, 
                 api_key: str = "", 
                 api_base: str = "https://api.openai.com/v1",
                 default_model: ImageModelType = ImageModelType.OPENAI_DALLE3,
                 timeout: int = 60):
        """
        初始化在线图片生成API客户端
        
        :param api_key: API密钥
        :param api_base: API基础地址
        :param default_model: 默认模型类型
        :param timeout: 超时时间（秒）
        """
        self.api_key = api_key
        self.api_base = api_base.rstrip("/")
        self.default_model = default_model
        self.timeout = timeout
        
        # 设置请求头
        headers = {
            "Content-Type": "application/json",
            "User-Agent": "ChatIG-Python-SDK/1.0.0"
        }
        
        if api_key:
            headers["Authorization"] = f"Bearer {api_key}"
        
        self.session = httpx.Client(
            headers=headers,
            timeout=self.timeout,
            http2=False
        )

    def generate_image(self, 
                      prompt: str,
                      model: Optional[ImageModelType] = None,
                      size: Optional[ImageSize] = None,
                      quality: str = "standard",
                      style: Optional[str] = None,
                      save_local: bool = False,
                      save_dir: str = "./generated_images") -> OnlineImageGenerationResponse:
        """
        生成图片
        
        :param prompt: 图片描述提示词
        :param model: 模型类型，默认使用初始化时的模型
        :param size: 图片尺寸
        :param quality: 图片质量
        :param style: 图片风格
        :param save_local: 是否保存到本地
        :param save_dir: 本地保存目录
        :return: 图片生成响应
        """
        model = model or self.default_model
        size = size or ImageSize.LARGE
        
        request = OnlineImageGenerationRequest(
            prompt=prompt,
            model=model,
            size=size,
            quality=quality,
            style=style,
            api_key=self.api_key,
            api_base=self.api_base
        )
        
        try:
            if model == ImageModelType.OPENAI_DALLE3:
                return self._generate_openai_dalle3(request, save_local, save_dir)
            elif model == ImageModelType.OPENAI_DALLE2:
                return self._generate_openai_dalle2(request, save_local, save_dir)
            elif model == ImageModelType.STABILITY_SDXL:
                return self._generate_stability_sdxl(request, save_local, save_dir)
            elif model == ImageModelType.STABILITY_SD3:
                return self._generate_stability_sd3(request, save_local, save_dir)
            else:
                raise Exception(f"不支持的模型类型: {model}")
        except Exception as e:
            raise Exception(f"图片生成失败: {str(e)}")

    def _generate_openai_dalle3(self, 
                               request: OnlineImageGenerationRequest, 
                               save_local: bool, 
                               save_dir: str) -> OnlineImageGenerationResponse:
        """使用OpenAI DALL-E 3生成图片"""
        url = f"{self.api_base}/images/generations"
        
        payload = {
            "model": "dall-e-3",
            "prompt": request.prompt,
            "size": request.size,
            "quality": request.quality,
            "n": request.n
        }
        
        if request.style:
            payload["style"] = request.style
        
        try:
            resp = self.session.post(url, json=payload)
            resp.raise_for_status()
            result = resp.json()
            
            # 转换响应格式
            image_data_list = []
            for item in result.get("data", []):
                image_data = OnlineImageData(
                    url=item.get("url"),
                    revised_prompt=item.get("revised_prompt")
                )
                
                # 如果需要保存到本地
                if save_local and item.get("url"):
                    local_path = self._download_and_save_image(
                        item["url"], save_dir, request.prompt
                    )
                    image_data.local_path = local_path
                
                image_data_list.append(image_data)
            
            return OnlineImageGenerationResponse(
                created=int(time.time()),
                data=image_data_list,
                model="dall-e-3",
                provider="OpenAI"
            )
            
        except httpx.HTTPStatusError as e:
            if e.response.status_code == 401:
                raise Exception("OpenAI API密钥无效，请检查密钥是否正确")
            elif e.response.status_code == 400:
                error_data = e.response.json()
                raise Exception(f"请求参数错误: {error_data.get('error', {}).get('message', '未知错误')}")
            elif e.response.status_code == 429:
                raise Exception("API请求频率超限，请稍后重试")
            else:
                raise Exception(f"OpenAI API请求失败: {e.response.status_code} {e.response.text}")

    def _generate_openai_dalle2(self, 
                               request: OnlineImageGenerationRequest, 
                               save_local: bool, 
                               save_dir: str) -> OnlineImageGenerationResponse:
        """使用OpenAI DALL-E 2生成图片"""
        url = f"{self.api_base}/images/generations"
        
        payload = {
            "model": "dall-e-2",
            "prompt": request.prompt,
            "size": request.size,
            "n": request.n
        }
        
        try:
            resp = self.session.post(url, json=payload)
            resp.raise_for_status()
            result = resp.json()
            
            image_data_list = []
            for item in result.get("data", []):
                image_data = OnlineImageData(
                    url=item.get("url"),
                    revised_prompt=item.get("revised_prompt")
                )
                
                if save_local and item.get("url"):
                    local_path = self._download_and_save_image(
                        item["url"], save_dir, request.prompt
                    )
                    image_data.local_path = local_path
                
                image_data_list.append(image_data)
            
            return OnlineImageGenerationResponse(
                created=int(time.time()),
                data=image_data_list,
                model="dall-e-2",
                provider="OpenAI"
            )
            
        except httpx.HTTPStatusError as e:
            if e.response.status_code == 401:
                raise Exception("OpenAI API密钥无效")
            else:
                raise Exception(f"OpenAI DALL-E 2请求失败: {e.response.status_code}")

    def _generate_stability_sdxl(self, 
                                request: OnlineImageGenerationRequest, 
                                save_local: bool, 
                                save_dir: str) -> OnlineImageGenerationResponse:
        """使用Stability AI SDXL生成图片"""
        url = "https://api.stability.ai/v1/generation/stable-diffusion-xl-1024-v1-0/text-to-image"
        
        headers = {"Authorization": f"Bearer {self.api_key}"}
        
        payload = {
            "text_prompts": [{"text": request.prompt}],
            "cfg_scale": 7,
            "height": int(request.size.split("x")[1]),
            "width": int(request.size.split("x")[0]),
            "samples": request.n,
            "steps": 30
        }
        
        try:
            resp = self.session.post(url, json=payload, headers=headers)
            resp.raise_for_status()
            result = resp.json()
            
            image_data_list = []
            for item in result.get("artifacts", []):
                # Stability AI返回base64数据
                image_data = OnlineImageData(
                    b64_json=item.get("base64")
                )
                
                if save_local and item.get("base64"):
                    local_path = self._save_base64_image(
                        item["base64"], save_dir, request.prompt
                    )
                    image_data.local_path = local_path
                
                image_data_list.append(image_data)
            
            return OnlineImageGenerationResponse(
                created=int(time.time()),
                data=image_data_list,
                model="stable-diffusion-xl",
                provider="Stability AI"
            )
            
        except httpx.HTTPStatusError as e:
            if e.response.status_code == 401:
                raise Exception("Stability AI API密钥无效")
            else:
                raise Exception(f"Stability AI请求失败: {e.response.status_code}")

    def _generate_stability_sd3(self, 
                               request: OnlineImageGenerationRequest, 
                               save_local: bool, 
                               save_dir: str) -> OnlineImageGenerationResponse:
        """使用Stability AI SD3生成图片"""
        url = "https://api.stability.ai/v1/generation/stable-diffusion-v1-6/text-to-image"
        
        headers = {"Authorization": f"Bearer {self.api_key}"}
        
        payload = {
            "text_prompts": [{"text": request.prompt}],
            "cfg_scale": 7,
            "height": int(request.size.split("x")[1]),
            "width": int(request.size.split("x")[0]),
            "samples": request.n,
            "steps": 30
        }
        
        try:
            resp = self.session.post(url, json=payload, headers=headers)
            resp.raise_for_status()
            result = resp.json()
            
            image_data_list = []
            for item in result.get("artifacts", []):
                image_data = OnlineImageData(
                    b64_json=item.get("base64")
                )
                
                if save_local and item.get("base64"):
                    local_path = self._save_base64_image(
                        item["base64"], save_dir, request.prompt
                    )
                    image_data.local_path = local_path
                
                image_data_list.append(image_data)
            
            return OnlineImageGenerationResponse(
                created=int(time.time()),
                data=image_data_list,
                model="stable-diffusion-v1-6",
                provider="Stability AI"
            )
            
        except httpx.HTTPStatusError as e:
            if e.response.status_code == 401:
                raise Exception("Stability AI API密钥无效")
            else:
                raise Exception(f"Stability AI SD3请求失败: {e.response.status_code}")

    def _download_and_save_image(self, url: str, save_dir: str, prompt: str) -> str:
        """下载并保存图片到本地"""
        try:
            if not os.path.exists(save_dir):
                os.makedirs(save_dir)
            
            # 生成文件名
            timestamp = int(time.time())
            safe_prompt = "".join(c for c in prompt if c.isalnum() or c in (' ', '-', '_')).rstrip()
            safe_prompt = safe_prompt[:50]  # 限制长度
            filename = f"{timestamp}_{safe_prompt}.png"
            filepath = os.path.join(save_dir, filename)
            
            # 下载图片
            resp = self.session.get(url)
            resp.raise_for_status()
            
            with open(filepath, "wb") as f:
                f.write(resp.content)
            
            return filepath
            
        except Exception as e:
            raise Exception(f"保存图片失败: {str(e)}")

    def _save_base64_image(self, b64_data: str, save_dir: str, prompt: str) -> str:
        """保存base64图片数据到本地"""
        try:
            if not os.path.exists(save_dir):
                os.makedirs(save_dir)
            
            timestamp = int(time.time())
            safe_prompt = "".join(c for c in prompt if c.isalnum() or c in (' ', '-', '_')).rstrip()
            safe_prompt = safe_prompt[:50]
            filename = f"{timestamp}_{safe_prompt}.png"
            filepath = os.path.join(save_dir, filename)
            
            # 解码base64数据
            image_data = base64.b64decode(b64_data)
            
            with open(filepath, "wb") as f:
                f.write(image_data)
            
            return filepath
            
        except Exception as e:
            raise Exception(f"保存base64图片失败: {str(e)}")

    def test_connection(self) -> bool:
        """测试API连接"""
        try:
            if self.default_model in [ImageModelType.OPENAI_DALLE3, ImageModelType.OPENAI_DALLE2]:
                response = self.session.get(f"{self.api_base}/models")
                return response.status_code == 200
            elif self.default_model in [ImageModelType.STABILITY_SDXL, ImageModelType.STABILITY_SD3]:
                response = self.session.get("https://api.stability.ai/v1/user/balance")
                return response.status_code == 200
            else:
                return False
        except Exception:
            return False

    def close(self):
        """关闭客户端"""
        self.session.close()

# 便捷函数
def create_openai_client(api_key: str) -> OnlineImageGenerationAPI:
    """创建OpenAI图片生成客户端"""
    return OnlineImageGenerationAPI(
        api_key=api_key,
        api_base="https://api.openai.com/v1",
        default_model=ImageModelType.OPENAI_DALLE3
    )

def create_stability_client(api_key: str) -> OnlineImageGenerationAPI:
    """创建Stability AI图片生成客户端"""
    return OnlineImageGenerationAPI(
        api_key=api_key,
        api_base="https://api.stability.ai/v1",
        default_model=ImageModelType.STABILITY_SDXL
    )

def quick_image_generation(prompt: str, 
                          api_key: str,
                          model: ImageModelType = ImageModelType.OPENAI_DALLE3,
                          save_local: bool = True) -> OnlineImageGenerationResponse:
    """快速图片生成"""
    if model in [ImageModelType.OPENAI_DALLE3, ImageModelType.OPENAI_DALLE2]:
        client = create_openai_client(api_key)
    elif model in [ImageModelType.STABILITY_SDXL, ImageModelType.STABILITY_SD3]:
        client = create_stability_client(api_key)
    else:
        raise Exception(f"不支持的模型类型: {model}")
    
    try:
        return client.generate_image(prompt, model, save_local=save_local)
    finally:
        client.close()
