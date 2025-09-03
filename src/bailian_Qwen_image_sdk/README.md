# Qwen Image SDK

一个用于调用阿里云百炼图像生成和编辑 API 的 Python 客户端库，支持直连百炼 API 和通过智能体平台调用。

## 📁 目录结构

```
Qwen_API/
├── app/                    # 智能体平台服务端代码
│   ├── main.py            # FastAPI路由和服务
│   ├── bailian_client.py  # 服务端百炼客户端
│   └── ...
├── image_sdk/             # SDK客户端库（本目录）
│   ├── __init__.py
│   ├── bailian_client.py  # 直连百炼API客户端
│   ├── platform_client.py # 智能体平台客户端
│   ├── types.py           # 数据类型定义
│   ├── examples/          # 使用示例
│   │   ├── generate.py
│   │   ├── edit_bailian.py
│   │   └── generate_via_platform.py
│   └── tests/             # 单元测试
│       ├── test_bailian_generate.py
│       └── test_platform_generate.py
└── test_sdk.py           # SDK测试脚本
```

## 🚀 快速开始

### 安装依赖

```bash
pip install requests
```

### 直连百炼 API

```python
from image_sdk.bailian_client import BailianClient

# 方式1：通过环境变量设置API Key
import os
os.environ["BAILIAN_API_KEY"] = "your-api-key-here"

client = BailianClient()
img = client.generate("一只猫坐在椅子上")
with open("cat.png", "wb") as f:
    f.write(img)

# 方式2：直接传入API Key
client = BailianClient(api_key="your-api-key-here")
img = client.generate("一只猫坐在椅子上")
```

### 通过智能体平台调用

```python
from image_sdk.platform_client import PlatformClient

# 连接到你的智能体平台
client = PlatformClient(base_url="http://localhost:8000")

# 生成图片
img = client.generate("星空下的狐狸")
with open("fox.png", "wb") as f:
    f.write(img)

# 编辑图片
with open("input.jpg", "rb") as f:
    image_bytes = f.read()
edited_img = client.edit_bailian(image_bytes, "将图中的人物改为站立姿势")
with open("edited.png", "wb") as f:
    f.write(edited_img)
```

## 📖 API 文档

### BailianClient（直连百炼）

#### 初始化

```python
BailianClient(
    api_key: str = None,           # 百炼API密钥，默认从环境变量BAILIAN_API_KEY读取
    base_url: str = "https://dashscope.aliyuncs.com/api/v1/services/aigc/multimodal-generation/generation",
    timeout: float = 60.0,         # 请求超时时间（秒）
    max_retries: int = 2           # 最大重试次数
)
```

#### 方法

**generate(prompt, size="1328\*1328", watermark=False) -> bytes**

- 根据文本描述生成图片
- 参数：
  - `prompt`: 图片描述文本
  - `size`: 图片尺寸，支持 "1024*1024", "1328*1328", "1792*1024", "1024*1792"
  - `watermark`: 是否添加水印
- 返回：图片字节数据

**edit(image, prompt, negative_prompt="", watermark=False) -> bytes**

- 编辑现有图片
- 参数：
  - `image`: 输入图片，可以是 bytes 或图片 URL 字符串
  - `prompt`: 编辑指令
  - `negative_prompt`: 负面提示词（可选）
  - `watermark`: 是否添加水印
- 返回：编辑后的图片字节数据

### PlatformClient（智能体平台）

#### 初始化

```python
PlatformClient(
    base_url: str,                 # 智能体平台地址
    api_key: str = None,           # 平台API密钥（可选）
    timeout: float = 30.0,         # 请求超时时间（秒）
    max_retries: int = 2           # 最大重试次数
)
```

#### 方法

**generate(prompt, size="1328\*1328", watermark=False) -> bytes**

- 通过平台生成图片（JSON 格式返回）

**generate_raw(prompt, size="1328\*1328", watermark=False) -> bytes**

- 通过平台生成图片（二进制格式返回）

**edit_bailian(image_bytes, prompt, negative_prompt="", watermark=False) -> bytes**

- 通过平台编辑图片（JSON 格式返回）

**edit_bailian_raw(image_bytes, prompt, negative_prompt="", watermark=False) -> bytes**

- 通过平台编辑图片（二进制格式返回）

**create_session() -> str**

- 创建会话，返回会话 ID

## 🧪 运行示例

### 1. 直连百炼生成图片

```bash
# 设置API Key
export BAILIAN_API_KEY="your-api-key-here"

# 运行示例
python examples/generate.py
```

### 2. 直连百炼编辑图片

```bash
# 准备输入图片 input.jpg
python examples/edit_bailian.py
```

### 3. 通过平台生成图片

```bash
# 确保智能体平台正在运行
# 启动平台：uvicorn app.main:app --reload

python examples/generate_via_platform.py
```

## 🧪 运行测试

```bash
# 安装测试依赖
pip install pytest requests-mock

# 运行所有测试
pytest tests/

# 运行特定测试
pytest tests/test_bailian_generate.py
```

## 📦 部署说明

### 智能体平台部署

1. **服务端代码**位于 `app/` 目录
2. **启动服务**：
   ```bash
   cd Qwen_API
   uvicorn app.main:app --host 0.0.0.0 --port 8000
   ```
3. **访问 API 文档**：http://localhost:8000/docs

### SDK 集成

1. **复制 SDK 目录**：将 `image_sdk/` 目录复制到你的项目中
2. **确保目录结构**：SDK 应与智能体平台的 `app/` 目录同级
3. **导入使用**：
   ```python
   from image_sdk.bailian_client import BailianClient
   from image_sdk.platform_client import PlatformClient
   ```

## 🔧 配置

### 环境变量

- `BAILIAN_API_KEY`: 百炼 API 密钥（用于直连百炼）
- `QWEN_IMAGE_EDIT_DIR`: 本地模型目录（智能体平台使用）

### 错误处理

SDK 提供统一的异常类型：

- `BailianError`: 百炼 API 相关错误
- `PlatformError`: 智能体平台相关错误

```python
from image_sdk.bailian_client import BailianClient, BailianError

try:
    client = BailianClient()
    img = client.generate("一只猫")
except BailianError as e:
    print(f"百炼API错误: {e}")
```

## 📝 注意事项

1. **API 配额**：确保百炼 API 密钥有足够的调用配额
2. **网络连接**：直连百炼需要能够访问阿里云 DashScope 服务
3. **图片格式**：生成的图片为 PNG 格式
4. **文件大小**：建议输入图片不超过 10MB
5. **并发限制**：注意 API 的并发调用限制

## 🤝 贡献

欢迎提交 Issue 和 Pull Request 来改进这个 SDK。

## 📄 许可证

本项目采用 MIT 许可证。

## 🔗 相关链接

- [阿里云百炼平台](https://bailian.console.aliyun.com/)
- [DashScope API 文档](https://help.aliyun.com/zh/dashscope/)
- [智能体平台文档](./README.md)
