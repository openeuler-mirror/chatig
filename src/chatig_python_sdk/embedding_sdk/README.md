# Embedding SDK

`embedding_sdk` 是 ChatIG 项目的 Python SDK，用于调用服务端 `/v1/embeddings` 接口，提供与 **OpenAI Embeddings** 兼容的请求与响应格式，支持文本向量化、批量请求、语义相似度计算等功能。

---

## 环境参数

SDK 的行为可通过以下环境变量控制：

| 变量名                 | 默认值                    | 说明                                   |
|------------------------|---------------------------|----------------------------------------|
| `CHATIG_BASE_URL`      | `http://127.0.0.1:8000`   | ChatIG 服务端基础地址                  |
| `CHATIG_API_KEY`       | 空                         | API Key（如服务端要求鉴权时设置）      |
| `CHATIG_TIMEOUT`       | `60` (秒)                 | HTTP 请求超时时间                      |

---

## 功能特性

- **标准接口封装**
  - `/v1/embeddings` → `EmbeddingsClient.embed()`

- **兼容 OpenAI Embeddings 格式**
  - 输入 `input` 可为字符串或字符串列表
  - 响应为 `EmbeddingResponse`，包含 `data`, `model`, `usage`

- **批量向量化**
  - 一次请求支持多条文本批量生成向量

- **一致性验证**
  - 相同输入 → 相同向量（Determinism）

- **语义相似度计算**
  - 内置 Cosine Similarity 示例，支持语义检索

- **健康检查**
  - `EmbeddingsClient.health_check()` 调用服务端 `/health` 检测可用性

---

## 使用方法

### 1. 安装依赖
```bash
pip install requests pydantic


cd chatig/src/chatig_python_sdk
python3 -m embedding_sdk.test_embedding
