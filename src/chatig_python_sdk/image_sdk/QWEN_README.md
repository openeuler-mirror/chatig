# Qwen3-0.6B 文字生成 SDK 使用指南

## 概述

这个 SDK 专门为连接 vLLM 部署的 Qwen3-0.6B 模型而设计，提供简洁易用的 Python 接口来生成文字内容。由于 Qwen3-0.6B 模型不支持图片生成，本 SDK 专注于文字生成功能。

## 特性

- ✅ 专为 Qwen3-0.6B 模型优化
- ✅ 支持系统提示词和用户提示词
- ✅ 可调节的生成参数（temperature、max_tokens 等）
- ✅ 完整的错误处理
- ✅ 支持多轮对话
- ✅ 简洁的 API 设计

## 安装依赖

```bash
pip install httpx pydantic
```

## 快速开始

### 1. 基本使用

```python
from image_sdk.text_generation import quick_text_generation

# 快速生成文字
response = quick_text_generation(
    prompt="请介绍一下人工智能",
    system_prompt="你是一个技术专家"
)
print(response)
```

### 2. 使用客户端实例

```python
from image_sdk.text_generation import create_qwen_client

# 创建客户端
client = create_qwen_client(api_base="http://localhost:8001")

try:
    # 生成文字
    response = client.generate_text(
        prompt="什么是机器学习？",
        system_prompt="你是一个机器学习专家",
        temperature=0.8,
        max_tokens=500
    )
    print(response)
finally:
    client.close()
```

## API 参考

### QwenTextGenerationAPI 类

#### 初始化参数

- `api_base`: API 基础地址，默认"http://localhost:8001"
- `api_key`: API 密钥，Qwen 模型通常不需要
- `timeout`: 超时时间（秒），默认 30

#### 主要方法

##### generate_text()

生成文字内容

```python
def generate_text(self,
                 prompt: str,
                 system_prompt: Optional[str] = None,
                 temperature: float = 0.7,
                 max_tokens: int = 1024,
                 stream: bool = False) -> str:
```

**参数：**

- `prompt`: 用户提示词
- `system_prompt`: 系统提示词（可选）
- `temperature`: 温度参数，控制随机性（0.0-1.0）
- `max_tokens`: 最大生成 token 数
- `stream`: 是否流式输出（目前简化处理）

**返回：** 生成的文字内容

##### chat_completion()

聊天完成接口

```python
def chat_completion(self,
                   messages: List[Dict[str, str]],
                   temperature: float = 0.7,
                   max_tokens: int = 1024) -> TextGenerationResponse:
```

**参数：**

- `messages`: 消息列表，格式为[{"role": "user", "content": "..."}]
- `temperature`: 温度参数
- `max_tokens`: 最大 token 数

**返回：** 完整的响应对象

##### test_connection()

测试 API 连接

```python
def test_connection(self) -> bool:
```

**返回：** 连接是否成功

## 使用示例

### 1. 简单问答

```python
from image_sdk.text_generation import create_qwen_client

client = create_qwen_client()

try:
    response = client.generate_text(
        prompt="Python是什么编程语言？",
        system_prompt="你是一个编程导师"
    )
    print(response)
finally:
    client.close()
```

### 2. 多轮对话

```python
from image_sdk.text_generation import create_qwen_client

client = create_qwen_client()

try:
    # 第一轮
    response1 = client.generate_text(
        prompt="什么是深度学习？",
        system_prompt="你是一个AI专家"
    )
    print(f"第一轮: {response1}")

    # 第二轮
    response2 = client.generate_text(
        prompt="它与传统机器学习有什么区别？",
        system_prompt="你是一个AI专家，请基于之前的回答继续"
    )
    print(f"第二轮: {response2}")
finally:
    client.close()
```

### 3. 使用聊天完成接口

```python
from image_sdk.text_generation import create_qwen_client

client = create_qwen_client()

try:
    messages = [
        {"role": "user", "content": "请介绍一下Python"},
        {"role": "assistant", "content": "Python是一种高级编程语言..."},
        {"role": "user", "content": "它有什么特点？"}
    ]

    response = client.chat_completion(messages, temperature=0.7)
    content = response.choices[0]['message']['content']
    print(content)
finally:
    client.close()
```

## 配置说明

### vLLM 服务配置

确保你的 vLLM 服务正确运行：

```bash
# 启动vLLM服务
python -m vllm.entrypoints.openai.api_server \
    --model Qwen/Qwen3-0.6B \
    --host 0.0.0.0 \
    --port 8001 \
    --trust-remote-code
```

### 网络配置

- 默认端口：8001
- 确保防火墙允许该端口访问
- 如果通过 SSH 连接，可能需要端口转发

## 错误处理

SDK 提供了完整的错误处理机制：

```python
from image_sdk.text_generation import create_qwen_client

client = create_qwen_client()

try:
    response = client.generate_text("测试消息")
    print(response)
except Exception as e:
    print(f"生成失败: {e}")
    # 处理具体错误类型
    if "连接" in str(e):
        print("网络连接问题")
    elif "参数" in str(e):
        print("请求参数错误")
finally:
    client.close()
```

## 测试连接

使用提供的测试脚本验证连接：

```bash
cd chatig_python_sdk/image_sdk
python test_qwen_connection.py
```

## 注意事项

1. **模型限制**：Qwen3-0.6B 是轻量级模型，生成质量可能不如更大模型
2. **资源管理**：使用完客户端后记得调用`close()`方法
3. **参数调优**：根据具体需求调整 temperature 和 max_tokens 参数
4. **错误重试**：网络不稳定时可以实现重试机制

## 故障排除

### 常见问题

1. **连接超时**

   - 检查网络延迟
   - 增加 timeout 参数
   - 确认 vLLM 服务状态

2. **模型不存在**

   - 确认 vLLM 中加载的模型名称
   - 检查模型是否正确下载

3. **端口不可达**
   - 检查防火墙设置
   - 确认 vLLM 监听地址
   - 验证网络配置

### 调试技巧

```python
# 启用详细日志
import logging
logging.basicConfig(level=logging.DEBUG)

# 测试基本连接
client = create_qwen_client()
print(f"连接测试: {client.test_connection()}")
client.close()
```

## 更新日志

- v1.0.0: 初始版本，支持基本的文字生成功能
- 适配 Qwen3-0.6B 模型
- 完整的错误处理和参数验证

## 许可证

遵循项目主许可证
