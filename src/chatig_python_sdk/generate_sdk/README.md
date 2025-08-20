# Generate SDK

`generate_sdk` 是 ChatIG 项目的 Python SDK，用于调用服务端 `/v1/tgi/generate` 与 `/v1/tgi/generate_stream` 接口，提供与 **OpenAI Chat Completions** 兼容的请求与响应格式，支持单轮和多轮对话、流式响应等功能。

---

## 环境参数

SDK 的行为可通过以下环境变量控制：

| 变量名              | 默认值                    | 说明                                   |
|---------------------|---------------------------|----------------------------------------|
| `CHATIG_BASE_URL`   | `http://127.0.0.1:8000`   | ChatIG 服务端基础地址                  |
| `CHATIG_API_KEY`    | 空                         | API Key（如服务端要求鉴权时设置）      |
| `CHATIG_TIMEOUT`    | `60` (秒)                 | HTTP 请求超时时间                      |

---

## 功能特性

- **标准接口封装**
  - `/v1/tgi/generate` → `GenerateClient.generate()`
  - `/v1/tgi/generate_stream` → `GenerateClient.generate_stream()`

- **兼容 OpenAI Chat 格式**
  - 请求消息 `messages=[{"role": "user", "content": "..."}]`
  - 响应包含 `choices[0].message.content`

- **多轮对话支持**
  - 自动维护消息历史，实现上下文语义连续性

- **流式响应 (SSE)**
  - `generate_stream()` 可逐块接收模型输出，适合长文本/实时场景

- **健康检查**
  - `GenerateClient.health_check()` 调用服务端 `/health` 检测可用性

- **参数控制**
  - `temperature`、`top_p`、`max_tokens`、`stop` 等采样参数与 OpenAI 接口一致

---

## 使用方法

### 1. 安装依赖
```bash
pip install requests


cd chatig/src/chatig_python_sdk
python3 -m generate_sdk.test_generate
