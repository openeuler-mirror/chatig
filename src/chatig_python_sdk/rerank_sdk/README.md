# Chatig 重排序 SDK

一个用于与Chatig重排序API端点交互的Python SDK。该SDK提供了简单直观的接口，用于使用各种重排序引擎执行文档重排序操作。

## 功能特性

- **多引擎支持**: 支持std、VLLM和LlamaBox重排序引擎
- **类型安全**: 完整的类型提示和验证
- **错误处理**: 全面的异常处理，包含特定的错误类型
- **配置管理**: 灵活的配置选项
- **重试逻辑**: 内置的失败请求重试机制
- **上下文管理器支持**: 自动资源清理

## 安装

```bash
# 克隆仓库
git clone <repository-url>
cd chatig2/sdk/rerank_sdk

# 安装依赖
pip install requests urllib3
```

## 快速开始

### 基本用法

```python
from rerank_sdk import RerankClient

# 创建客户端
client = RerankClient()

# 定义文档和查询
documents = [
    "敏捷的棕色狐狸跳过懒狗。",
    "一只懒狗在阳光下睡觉。",
    "狐狸敏捷且棕色。",
    "狗是忠诚的伙伴。"
]
query = "敏捷狐狸"

# 执行重排序
response = client.rerank_std(
    model="bge-reranker-v2-m3",
    query=query,
    documents=documents,
    top_n=2
)

# 打印结果
for i, result in enumerate(response.results):
    print(f"{i+1}. 分数: {result.score:.4f}, 文档: {documents[result.index]}")

client.close()
```

### 使用上下文管理器

```python
from rerank_sdk import RerankClient

with RerankClient() as client:
    response = client.rerank_std(
        model="bge-reranker-v2-m3",
        query="机器学习",
        documents=["文档 1", "文档 2", "文档 3"]
    )
    # 客户端自动关闭
```

## 配置

### 环境变量

```bash
export CHATIG_BASE_URL="http://localhost:8001"
export CHATIG_API_KEY="your-api-key"
export CHATIG_TIMEOUT="30.0"
export CHATIG_MAX_RETRIES="3"
export CHATIG_RETRY_DELAY="1.0"
export CHATIG_VERIFY_SSL="true"
export CHATIG_PROXY="http://proxy:8080"
```

### 程序化配置

```python
from rerank_sdk import RerankClient, RerankConfig

config = RerankConfig(
    base_url="http://localhost:8001",
    api_key="your-api-key",
    timeout=60.0,
    max_retries=5
)

client = RerankClient(config)
```

## API 参考

### RerankClient

用于与重排序API交互的主客户端类。

#### 方法

##### `rerank(model, query, documents, engine_type='std', top_n=None, **kwargs)`

使用任何支持的重排序引擎执行重排序操作。

**参数:**
- `model` (str): 用于重排序的模型名称
- `query` (str): 查询文本
- `documents` (List[str]): 要重排序的文档列表
- `engine_type` (str/RerankEngineType): 重排序引擎类型 ('std', 'vllm', 'llamabox')
- `top_n` (int, optional): 返回的顶部结果数量
- `**kwargs`: 特定引擎类型的附加参数

**返回:** 重排序响应对象

##### `rerank_std(model, query, documents, top_n=None, parameters=None)`

执行标准重排序操作。

**参数:**
- `model` (str): 模型名称
- `query` (str): 查询文本
- `documents` (List[str]): 要重排序的文档
- `top_n` (int, optional): 顶部结果数量
- `parameters` (dict, optional): 附加参数

**返回:** StdRerankResponse

##### `rerank_vllm(model, query, documents)`

执行VLLM重排序操作。

**参数:**
- `model` (str): 模型名称
- `query` (str): 查询文本
- `documents` (List[str]): 要重排序的文档

**返回:** VLLMRerankResponse

##### `rerank_llamabox(model, query, documents, top_n=None)`

执行LlamaBox重排序操作。

**参数:**
- `model` (str): 模型名称
- `query` (str): 查询文本
- `documents` (List[str]): 要重排序的文档
- `top_n` (int, optional): 顶部结果数量

**返回:** LlamaBoxRerankResponse

##### `health_check()`

检查重排序服务是否健康。

**返回:** bool

### 数据模型

#### 请求模型

- `StdRerankRequest`: 标准重排序请求
- `LlamaBoxRerankRequest`: LlamaBox重排序请求
- `VLLMRerankRequest`: VLLM重排序请求

#### 响应模型

- `StdRerankResponse`: 标准重排序响应
- `LlamaBoxRerankResponse`: LlamaBox重排序响应
- `VLLMRerankResponse`: VLLM重排序响应

#### 通用模型

- `RerankResult`: 基础重排序结果结构
- `Document`: 文档结构
- `Usage`: 使用统计
- `RerankParameters`: 重排序参数

### 异常

- `RerankError`: 所有重排序错误的基础异常
- `RerankValidationError`: 请求验证错误
- `RerankConnectionError`: 连接错误
- `RerankTimeoutError`: 超时错误
- `RerankAuthenticationError`: 认证错误
- `RerankRateLimitError`: 速率限制错误
- `RerankModelNotFoundError`: 模型未找到错误

## 示例

### 标准重排序

```python
from rerank_sdk import RerankClient

client = RerankClient()

response = client.rerank_std(
    model="bge-reranker-v2-m3",
    query="人工智能",
    documents=[
        "机器学习是人工智能的一个子集。",
        "深度学习使用神经网络。",
        "自然语言处理很重要。"
    ],
    top_n=2
)

for result in response.results:
    print(f"分数: {result.score:.4f}")
```

### VLLM重排序

```python
response = client.rerank_vllm(
    model="llama-2-7b-chat",
    query="神经网络",
    documents=[
        "深度学习使用神经网络。",
        "机器学习算法。",
        "计算机视觉应用。"
    ]
)

print(f"模型: {response.model}")
print(f"总令牌数: {response.usage.total_tokens}")
```

### LlamaBox重排序

```python
response = client.rerank_llamabox(
    model="llama-2-7b-chat",
    query="网络开发",
    documents=[
        "JavaScript用于网络开发。",
        "Python是一种编程语言。",
        "HTML是一种标记语言。"
    ],
    top_n=3
)

for result in response.results:
    print(f"分数: {result.relevance_score:.4f}")
    print(f"文档: {result.document.text}")
```

### 批量处理

```python
documents = ["文档 1", "文档 2", "文档 3", "文档 4"]
queries = ["查询 1", "查询 2", "查询 3"]

for query in queries:
    response = client.rerank_std(
        model="bge-reranker-v2-m3",
        query=query,
        documents=documents,
        top_n=2
    )
    print(f"查询: {query}")
    for result in response.results:
        print(f"  分数: {result.score:.4f}")
```

## 错误处理

```python
from rerank_sdk import RerankClient, RerankError, RerankValidationError

client = RerankClient()

try:
    response = client.rerank_std(
        model="",
        query="",
        documents=[]
    )
except RerankValidationError as e:
    print(f"验证错误: {e}")
except RerankError as e:
    print(f"重排序错误: {e}")
finally:
    client.close()
```

## 健康检查

```python
client = RerankClient()

if client.health_check():
    print("✅ 服务健康")
else:
    print("❌ 服务不健康")

client.close()
```

## 贡献

1. Fork 仓库
2. 创建功能分支
3. 进行更改
4. 添加测试
5. 提交拉取请求

## 许可证

本项目采用与主Chatig项目相同的许可证。

## 支持

如需支持和问题解答，请参考主Chatig文档或在仓库中创建issue。 