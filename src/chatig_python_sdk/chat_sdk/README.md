# ChatIG Chat Module - 推理网关聊天模块

ChatIG Chat Module 是专门为ChatIG推理网关设计的Python SDK聊天模块，提供了完整的聊天功能接口封装。

## 功能特性

- ✅ **完整的API封装** - 对应Rust代码中的所有聊天功能
- ✅ **多模型支持** - 支持Qwen、GLM、Llama、Bailian、DeepSeek等系列模型
- ✅ **同步/异步支持** - 提供同步和异步两种客户端
- ✅ **流式聊天** - 支持实时流式对话
- ✅ **会话管理** - 完整的对话历史管理功能
- ✅ **错误处理** - 完善的异常处理和重试机制
- ✅ **类型安全** - 使用Pydantic进行数据验证
- ✅ **上下文管理** - 支持with语句自动资源管理

## 快速开始

### 安装依赖

```bash
pip install httpx pydantic
```

### 基础使用

```python
from chatig_python_sdk.chat import ChatClient

# 创建客户端
client = ChatClient(
    api_key="your-api-key",
    api_base="http://localhost:8080"
)

# 简单聊天
response = client.chat("你好，请介绍一下你自己")
print(response)

# 关闭客户端
client.close()
```

### 使用上下文管理器

```python
from chatig_python_sdk.chat import ChatClient

with ChatClient(api_key="your-api-key") as client:
    response = client.chat("你好")
    print(response)
```

## 核心组件

### 1. ChatClient - 聊天客户端

主要的聊天客户端类，提供与ChatIG推理网关的完整交互功能。

#### 初始化参数

- `api_key` (str): API密钥
- `api_base` (str): API基础地址，默认 "http://localhost:8080"
- `timeout` (float): 请求超时时间，默认 60.0秒
- `max_retries` (int): 最大重试次数，默认 3次
- `retry_delay` (float): 重试延迟时间，默认 1.0秒

#### 主要方法

##### 基础聊天

```python
def chat(
    self,
    message: str,
    system_prompt: Optional[str] = None,
    model: str = "Qwen/Qwen2.5-7B-Instruct",
    temperature: float = 0.7,
    max_tokens: int = 1024
) -> str
```

##### 完整API调用

```python
def create_completion(
    self,
    messages: List[ChatMessage],
    model: str = "Qwen/Qwen2.5-7B-Instruct",
    temperature: Optional[float] = None,
    top_p: Optional[int] = None,
    n: Optional[int] = None,
    stream: bool = False,
    stop: Optional[List[str]] = None,
    max_tokens: Optional[int] = None,
    presence_penalty: Optional[int] = None,
    frequency_penalty: Optional[int] = None,
    logit_bias: Optional[int] = None,
    user: Optional[str] = None,
    stream_options: Optional[StreamOptions] = None,
    file_id: Optional[str] = None
) -> Union[ChatCompletionResponse, CompletionsResponse]
```

##### 流式聊天

```python
def chat_stream(
    self,
    message: str,
    system_prompt: Optional[str] = None,
    model: str = "Qwen/Qwen2.5-7B-Instruct",
    temperature: float = 0.7,
    max_tokens: int = 1024
) -> AsyncGenerator[str, None]
```

##### 健康检查

```python
def health_check(self) -> bool
```

##### 模型验证

```python
def validate_model(self, model: str) -> bool
def get_supported_models(self) -> Dict[str, str]
```

### 2. AsyncChatClient - 异步聊天客户端

异步版本的聊天客户端，提供非阻塞的API调用。

```python
from chatig_python_sdk.chat import AsyncChatClient
import asyncio

async def main():
    async with AsyncChatClient(api_key="your-api-key") as client:
        response = await client.chat("你好")
        print(response)

asyncio.run(main())
```

### 3. ConversationManager - 会话管理器

提供对话历史管理和会话状态维护功能。

```python
from chatig_python_sdk.chat import ChatClient, ConversationManager

client = ChatClient(api_key="your-api-key")
conv_manager = ConversationManager(client)

# 创建会话
conv_id = conv_manager.create_conversation("我的对话")

# 在会话中聊天
response = conv_manager.chat_with_conversation(
    message="你好",
    conversation_id=conv_id,
    system_prompt="你是一个友好的助手"
)

# 查看会话历史
messages = conv_manager.get_conversation_messages(conv_id)
```

#### 会话管理方法

- `create_conversation(title, metadata)` - 创建新会话
- `get_conversation(conversation_id)` - 获取指定会话
- `list_conversations()` - 列出所有会话
- `delete_conversation(conversation_id)` - 删除会话
- `set_active_conversation(conversation_id)` - 设置活跃会话
- `chat_with_conversation(message, ...)` - 在会话中聊天
- `export_conversation(conversation_id)` - 导出会话数据
- `import_conversation(conversation_data)` - 导入会话数据

### 4. 数据模型

#### ChatMessage - 聊天消息

```python
from chatig_python_sdk.chat import ChatMessage

# 创建不同类型的消息
system_msg = ChatMessage.create_system_message("你是一个助手")
user_msg = ChatMessage.create_user_message("你好")
assistant_msg = ChatMessage.create_assistant_message("你好！有什么可以帮助你的吗？")
```

#### 响应模型

- `ChatCompletionResponse` - 标准聊天完成响应
- `CompletionsResponse` - 增强版聊天完成响应
- `CompletionsStreamResponse` - 流式响应

### 5. 异常处理

模块提供了完整的异常处理机制：

```python
from chatig_python_sdk.chat import (
    ChatIGError,
    AuthenticationError,
    RateLimitError,
    ServerError,
    ValidationError,
    ModelNotFoundError
)

try:
    response = client.chat("测试消息")
except AuthenticationError:
    print("认证失败")
except RateLimitError:
    print("请求频率超限")
except ModelNotFoundError:
    print("模型不存在")
except ChatIGError as e:
    print(f"其他错误: {e}")
```

## 使用示例

### 基础聊天

```python
from chatig_python_sdk.chat import ChatClient

client = ChatClient(api_key="your-api-key")

# 简单聊天
response = client.chat("你好")
print(response)

# 带系统提示词的聊天
response = client.chat(
    message="请解释什么是机器学习",
    system_prompt="你是一个AI专家，请用通俗易懂的语言解释概念"
)
print(response)
```

### 高级聊天

```python
from chatig_python_sdk.chat import ChatClient, ChatMessage

client = ChatClient(api_key="your-api-key")

# 创建消息列表
messages = [
    ChatMessage.create_system_message("你是一个编程助手"),
    ChatMessage.create_user_message("请用Python写一个排序算法")
]

# 使用完整API
response = client.create_completion(
    messages=messages,
    model="Qwen/Qwen2.5-7B-Instruct",
    temperature=0.8,
    max_tokens=1500,
    presence_penalty=0.1,
    frequency_penalty=0.1
)

print(response.choices[0].message.content)
print(f"Token使用: {response.usage}")
```

### 流式聊天

```python
import asyncio
from chatig_python_sdk.chat import ChatClient

async def stream_chat():
    client = ChatClient(api_key="your-api-key")
    
    print("开始流式对话...")
    async for chunk in client.chat_stream("请写一个关于春天的诗"):
        print(chunk, end="", flush=True)
    print("\n对话完成")
    
    client.close()

asyncio.run(stream_chat())
```

### 会话管理

```python
from chatig_python_sdk.chat import ChatClient, ConversationManager

client = ChatClient(api_key="your-api-key")
conv_manager = ConversationManager(client)

# 创建会话
conv_id = conv_manager.create_conversation("编程学习")

# 多轮对话
questions = [
    "什么是Python？",
    "Python有哪些特点？",
    "如何安装Python？"
]

for question in questions:
    response = conv_manager.chat_with_conversation(
        message=question,
        conversation_id=conv_id,
        system_prompt="你是一个Python编程老师"
    )
    print(f"Q: {question}")
    print(f"A: {response}\n")

# 查看会话统计
stats = conv_manager.get_conversation_stats()
print(f"会话统计: {stats}")
```

### 错误处理

```python
from chatig_python_sdk.chat import ChatClient, ModelNotFoundError

client = ChatClient(api_key="your-api-key")

try:
    # 健康检查
    if not client.health_check():
        print("服务不可用")
        exit(1)
    
    # 验证模型
    if not client.validate_model("Qwen/Qwen2.5-7B-Instruct"):
        print("模型不支持")
        exit(1)
    
    # 聊天
    response = client.chat("测试消息")
    print(response)
    
except ModelNotFoundError as e:
    print(f"模型错误: {e}")
except Exception as e:
    print(f"其他错误: {e}")
finally:
    client.close()
```

## 支持的模型

ChatIG Chat Module 支持以下模型系列：

- **Qwen** - Qwen系列模型
- **GLM** - GLM系列模型
- **meta-llama** - Llama系列模型
- **Bailian** - Bailian系列模型
- **deepseek-ai** - DeepSeek系列模型

模型名称格式：`series/model-name`

例如：
- `Qwen/Qwen2.5-7B-Instruct`
- `GLM/GLM-4`
- `meta-llama/Llama-3-8B-Instruct`

## 配置说明

### 环境变量

可以通过环境变量配置客户端：

```bash
export CHATIG_API_KEY="your-api-key"
export CHATIG_API_BASE="http://localhost:8080"
export CHATIG_TIMEOUT="60.0"
export CHATIG_MAX_RETRIES="3"
```

### 配置文件

也可以使用配置文件进行配置（需要自行实现配置加载逻辑）。

## 最佳实践

### 1. 资源管理

始终使用上下文管理器或手动关闭客户端：

```python
# 推荐
with ChatClient(api_key="your-api-key") as client:
    response = client.chat("你好")

# 或者手动关闭
client = ChatClient(api_key="your-api-key")
try:
    response = client.chat("你好")
finally:
    client.close()
```

### 2. 错误处理

实现完善的错误处理机制：

```python
try:
    response = client.chat("测试消息")
except AuthenticationError:
    # 处理认证错误
    pass
except RateLimitError:
    # 处理频率限制
    pass
except ServerError:
    # 处理服务器错误
    pass
except Exception as e:
    # 处理其他错误
    pass
```

### 3. 会话管理

合理使用会话管理功能：

```python
# 为不同类型的对话创建不同的会话
coding_conv = conv_manager.create_conversation("编程问题")
general_conv = conv_manager.create_conversation("一般对话")

# 在相应会话中进行对话
conv_manager.chat_with_conversation("Python问题", coding_conv)
conv_manager.chat_with_conversation("日常聊天", general_conv)
```

### 4. 流式处理

对于长对话，使用流式处理提供更好的用户体验：

```python
async def handle_stream_chat():
    async for chunk in client.chat_stream("长问题"):
        # 实时显示内容
        print(chunk, end="", flush=True)
```

## 故障排除

### 常见问题

1. **认证失败**
   - 检查API密钥是否正确
   - 确认API密钥是否有效

2. **连接超时**
   - 检查网络连接
   - 调整timeout参数
   - 确认服务地址是否正确

3. **模型不支持**
   - 检查模型名称格式
   - 确认模型是否在支持列表中

4. **频率限制**
   - 降低请求频率
   - 实现重试机制

### 调试模式

启用详细日志输出：

```python
import logging

logging.basicConfig(level=logging.DEBUG)
```

## 更新日志

### v1.0.0
- 初始版本发布
- 支持基础聊天功能
- 支持流式聊天
- 支持会话管理
- 支持多模型系列

## 贡献指南

欢迎提交Issue和Pull Request来改进这个模块。

## 许可证

本项目采用MIT许可证。 