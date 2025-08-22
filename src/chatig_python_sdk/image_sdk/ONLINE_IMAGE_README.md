# 在线图片生成 SDK 使用指南

## 概述

这个 SDK 专门为连接各种在线图片生成服务而设计，支持 OpenAI DALL-E、Stability AI、Midjourney 等主流图片生成模型。由于本地的 Qwen3-0.6B 模型不支持图片生成，这个 SDK 提供了完整的在线图片生成解决方案。

## 🎯 支持的模型

### 1. **OpenAI DALL-E 系列**

- **DALL-E 3**: 最新版本，支持高质量图片生成
- **DALL-E 2**: 稳定版本，支持多种尺寸和风格
- **特点**: 质量高，支持自然语言描述，有修订提示词功能

### 2. **Stability AI Stable Diffusion**

- **SDXL**: 高质量图片生成，支持多种尺寸
- **SD3**: 最新版本，改进的生成质量
- **特点**: 开源基础，支持艺术风格，有免费额度

### 3. **其他模型**

- **Midjourney**: 艺术风格图片生成（需要 API 访问）
- **Hugging Face**: 开源模型集合

## 🚀 快速开始

### 1. 安装依赖

```bash
pip install httpx pydantic
```

### 2. 设置 API 密钥

#### 方法 1：环境变量（推荐）

```bash
# OpenAI
export OPENAI_API_KEY="your_openai_api_key_here"

# Stability AI
export STABILITY_API_KEY="your_stability_api_key_here"

# 其他
export MIDJOURNEY_API_KEY="your_midjourney_api_key_here"
export HUGGINGFACE_API_KEY="your_huggingface_api_key_here"
```

#### 方法 2：配置文件

```python
from image_sdk.config import create_config_file, set_api_key

# 创建配置文件
create_config_file("./my_config.json")

# 设置API密钥
set_api_key("openai", "your_openai_key")
set_api_key("stability", "your_stability_key")
```

### 3. 基本使用

#### 使用 OpenAI DALL-E 3

```python
from image_sdk import create_openai_client, ImageSize

# 创建客户端
client = create_openai_client("your_openai_api_key")

try:
    # 生成图片
    response = client.generate_image(
        prompt="一只可爱的小猫坐在花园里，阳光明媚",
        size=ImageSize.LARGE,
        quality="hd",
        save_local=True
    )

    print(f"✅ 图片生成成功！")
    for image_data in response.data:
        if image_data.local_path:
            print(f"本地路径: {image_data.local_path}")

finally:
    client.close()
```

#### 使用 Stability AI

```python
from image_sdk import create_stability_client, ImageModelType

# 创建客户端
client = create_stability_client("your_stability_api_key")

try:
    # 生成图片
    response = client.generate_image(
        prompt="A beautiful landscape painting, mountains and lake",
        model=ImageModelType.STABILITY_SDXL,
        size=ImageSize.LARGE,
        save_local=True
    )

    print(f"✅ 图片生成成功！")

finally:
    client.close()
```

#### 快速生成

```python
from image_sdk import quick_image_generation, ImageModelType

# 快速生成图片
response = quick_image_generation(
    prompt="一个未来科技感的机器人，霓虹灯效果",
    api_key="your_api_key",
    model=ImageModelType.OPENAI_DALLE3,
    save_local=True
)

print(f"生成了 {len(response.data)} 张图片")
```

## 📚 API 参考

### OnlineImageGenerationAPI 类

#### 初始化参数

```python
OnlineImageGenerationAPI(
    api_key: str = "",                    # API密钥
    api_base: str = "https://api.openai.com/v1",  # API基础地址
    default_model: ImageModelType = ImageModelType.OPENAI_DALLE3,  # 默认模型
    timeout: int = 60                     # 超时时间（秒）
)
```

#### 主要方法

##### generate_image()

```python
def generate_image(
    self,
    prompt: str,                          # 图片描述提示词
    model: Optional[ImageModelType] = None,  # 模型类型
    size: Optional[ImageSize] = None,     # 图片尺寸
    quality: str = "standard",            # 图片质量
    style: Optional[str] = None,          # 图片风格
    save_local: bool = False,             # 是否保存到本地
    save_dir: str = "./generated_images"  # 本地保存目录
) -> OnlineImageGenerationResponse:
```

**支持的尺寸选项:**

- `ImageSize.SMALL`: 256x256
- `ImageSize.MEDIUM`: 512x512
- `ImageSize.LARGE`: 1024x1024
- `ImageSize.WIDE`: 1792x1024
- `ImageSize.TALL`: 1024x1792

**支持的模型类型:**

- `ImageModelType.OPENAI_DALLE3`: OpenAI DALL-E 3
- `ImageModelType.OPENAI_DALLE2`: OpenAI DALL-E 2
- `ImageModelType.STABILITY_SDXL`: Stability AI SDXL
- `ImageModelType.STABILITY_SD3`: Stability AI SD3

### 便捷函数

#### create_openai_client()

```python
def create_openai_client(api_key: str) -> OnlineImageGenerationAPI:
    """创建OpenAI图片生成客户端"""
```

#### create_stability_client()

```python
def create_stability_client(api_key: str) -> OnlineImageGenerationAPI:
    """创建Stability AI图片生成客户端"""
```

#### quick_image_generation()

```python
def quick_image_generation(
    prompt: str,
    api_key: str,
    model: ImageModelType = ImageModelType.OPENAI_DALLE3,
    save_local: bool = True
) -> OnlineImageGenerationResponse:
    """快速图片生成"""
```

## 🔧 配置管理

### 配置状态检查

```python
from image_sdk import print_config_status, check_config

# 打印配置状态
print_config_status()

# 检查API密钥状态
status = check_config()
print(status)
```

### 动态设置 API 密钥

```python
from image_sdk import set_api_key, get_api_key

# 设置API密钥
set_api_key("openai", "new_openai_key")
set_api_key("stability", "new_stability_key")

# 获取API密钥
openai_key = get_api_key("openai")
stability_key = get_api_key("stability")
```

### 配置文件管理

```python
from image_sdk import create_config_file, load_config_from_file

# 创建配置文件
create_config_file("./my_config.json")

# 从文件加载配置
load_config_from_file("./my_config.json")
```

## 💡 使用示例

### 1. 批量生成图片

```python
from image_sdk import create_openai_client, ImageSize

client = create_openai_client("your_api_key")

prompts = [
    "一只可爱的小猫",
    "一只忠诚的小狗",
    "一只彩色的小鸟"
]

try:
    for prompt in prompts:
        response = client.generate_image(
            prompt=prompt,
            size=ImageSize.MEDIUM,
            save_local=True
        )
        print(f"✅ {prompt}: 生成成功")

finally:
    client.close()
```

### 2. 不同风格和尺寸

```python
from image_sdk import create_openai_client, ImageSize

client = create_openai_client("your_api_key")

try:
    # 高质量大图
    response1 = client.generate_image(
        prompt="一个未来城市，霓虹灯闪烁",
        size=ImageSize.WIDE,
        quality="hd",
        style="vivid"
    )

    # 标准尺寸
    response2 = client.generate_image(
        prompt="一个宁静的乡村风景",
        size=ImageSize.LARGE,
        quality="standard"
    )

finally:
    client.close()
```

### 3. 错误处理和重试

```python
from image_sdk import create_openai_client
import time

client = create_openai_client("your_api_key")

max_retries = 3
retry_delay = 2

for attempt in range(max_retries):
    try:
        response = client.generate_image(
            prompt="一个复杂的科幻场景",
            save_local=True
        )
        print("✅ 图片生成成功！")
        break

    except Exception as e:
        print(f"❌ 尝试 {attempt + 1} 失败: {e}")
        if attempt < max_retries - 1:
            print(f"等待 {retry_delay} 秒后重试...")
            time.sleep(retry_delay)
        else:
            print("❌ 所有重试都失败了")

finally:
    client.close()
```

## ⚠️ 注意事项

### 1. **API 限制和费用**

- OpenAI DALL-E 3: 每张图片约$0.04-0.08
- Stability AI: 有免费额度，超出后按使用量计费
- 注意 API 调用频率限制

### 2. **图片质量**

- DALL-E 3 质量最高，但价格较贵
- SDXL 性价比高，适合批量生成
- 根据需求选择合适的模型

### 3. **提示词优化**

- 使用详细、具体的描述
- 指定风格、色调、构图等
- 避免过于复杂或矛盾的描述

### 4. **本地存储**

- 使用`save_local=True`保存图片到本地
- 默认保存目录：`./generated_images`
- 可以自定义保存路径

## 🚨 故障排除

### 常见问题

#### 1. API 密钥无效

```
❌ OpenAI API密钥无效，请检查密钥是否正确
```

**解决方案:**

- 检查 API 密钥是否正确
- 确认账户余额充足
- 验证 API 密钥权限

#### 2. 请求频率超限

```
❌ API请求频率超限，请稍后重试
```

**解决方案:**

- 降低请求频率
- 实现重试机制
- 使用队列管理请求

#### 3. 图片生成失败

```
❌ 图片生成失败: 请求参数错误
```

**解决方案:**

- 检查提示词是否合适
- 验证尺寸参数
- 确认模型支持的功能

### 调试技巧

```python
# 启用详细日志
import logging
logging.basicConfig(level=logging.DEBUG)

# 测试连接
client = create_openai_client("your_key")
print(f"连接测试: {client.test_connection()}")
client.close()

# 检查配置
from image_sdk import print_config_status
print_config_status()
```

## 📁 文件结构

```
chatig_python_sdk/image_sdk/
├── __init__.py                    # 模块导出
├── online_image_generation.py     # 在线图片生成核心
├── config.py                      # 配置管理
├── online_image_example.py        # 使用示例
├── image_generation.py            # 原有图片生成（本地）
├── text_generation.py             # 文字生成（Qwen）
└── README文件
```

## 🔄 更新日志

- **v2.0.0**: 添加在线图片生成支持

  - 支持 OpenAI DALL-E 系列
  - 支持 Stability AI Stable Diffusion
  - 完整的配置管理系统
  - 本地图片保存功能
  - 错误处理和重试机制

- **v1.0.0**: 基础版本
  - 本地图片生成
  - Qwen 文字生成

## 📞 技术支持

如果遇到问题，请检查：

1. API 密钥是否正确设置
2. 网络连接是否正常
3. 账户余额是否充足
4. 提示词是否合适

## 📄 许可证

遵循项目主许可证
