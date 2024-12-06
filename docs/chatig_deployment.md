# Chatig api 开发文档



## 零、 接口测试

#### 1. chatig测试
```bash
curl -X GET http://localhost:8081
```

### （一）chatchat

#### 1. v1/chat/completions
```bash
curl --no-buffer -X POST http://localhost:8081/v1/chat/completions -H "Content-Type: application/json" -H "Authorization: Bearer sk-culinux" -d '{
  "model": "chatchat",
  "messages": [
    {"role": "user", "content": "CULinux的稳定性怎么样？"}
  ],
  "max_tokens": 0,
  "temperature": 0.7,
  "stream": true
}'
```


## 一、chatig

#### 1. Chat API

为给定的聊天对话创建模型响应。

**端点**: `POST /v1/chat/completions`

**（1）请求体:**

  ```json
  {
    "model": "",      
    "messages": [
      {"role": "system", "content": "You are a helpful assistant."},
      {"role": "user", "content": "What is the weather like today?"}
    ],
    "temperature": 0.7,
    "top_p": 1,
    "n": 1,
    "stream": false,
    "stop": null,
    "max_tokens": 100,
    "presence_penalty": 0,
    "frequency_penalty": 0,
    "logit_bias": null,
    "user": "user-1234"
  }
  ```

请求参数解释：

- **model**: （必须项）使用的模型名称，如 `"gpt-4"`, `"gpt-3.5-turbo"` 等。

- **messages**: （必须项）消息列表，其中每条消息必须包含 `role`和 `content`。`role` 可以是`system`（系统）、`user`（用户）、或 `assistant`（助手）。
  - `system`：通常用于设定助手的行为，如“你是一个帮助用户解决问题的助手”。
  - `user`：用户的输入内容。
  - `assistant`：助手的回复内容（可选，通常是之前对话中的助手响应）。

- **temperature**: 控制生成的文本的创造性。较高的值（如 `0.8`）将使输出更随机，而较低的值（如 `0.2`）则更加集中和确定。

- **top_p**: 一种替代 `temperature` 的采样方法。`top_p` 会根据累积概率选择 token，`1` 表示考虑所有 token。

- **n**: 生成的回复数量。

- **stream**: 是否开启流式响应。若为 `true`，响应会逐步返回部分内容。

- **stop**: 停止生成的字符串，支持字符串数组（如 `["\n"]`）。

- **max_tokens**: 单次请求生成的最大 token 数。

- **presence_penalty**: 是否鼓励生成新主题。值为 `-2.0` 至 `2.0` 之间，正值增加生成新话题的可能性。

- **frequency_penalty**: 控制生成重复 token 的可能性。值为 `-2.0` 至 `2.0` 之间，正值减少重复。

- **logit_bias**: 用于调整特定 token 出现的概率。接受一个 map，key 是 token 的 ID，值是从 `-100` 到 `100` 的数值。

- **user**: 用户 ID，用于标识请求来源的用户。



**（2）响应体:**

```json
// 非流式返回
{
  "id": "chatcmpl-abc1234567890",
  "object": "chat.completion",
  "created": 1686243012,
  "model": "gpt-4",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "The weather is sunny with a chance of clouds."
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 10,
    "completion_tokens": 9,
    "total_tokens": 19
  }
}

// 流式返回
data: {
    "id": "chatcmpl-bdsRtsUcMyGGgZx9zJT3qQ",
    "model": "qwen1.5",
    "choices": [
        {
            "index": 0,
            "delta": {
                "role": "assistant"
            },
            "finish_reason": null
        }
    ]
}
...
data: {
    "id": "chatcmpl-bdsRtsUcMyGGgZx9zJT3qQ",
    "object": "chat.completion.chunk",
    "created": 1727265100,
    "model": "qwen1.5",
    "choices": [
        {
            "index": 0,
            "delta": {

            },
            "finish_reason": "stop"
        }
    ]
}

data: [DONE]
```

响应参数解释：

- **id**: 每次生成的回复的唯一标识符。

- **object**: 响应对象类型，如 `"chat.completion"`。

- **created**: 响应的生成时间戳。

- **model**: 使用的模型名称。

- **choices:** 返回的生成文本选项列表。
  - `index`: 选项的索引。
  - `message`: 包含生成的消息，`role` 为 `"assistant"`，`content` 是生成的回复内容。
  - `finish_reason`: 生成结束的原因，如 `stop`（正常结束）、`length`（达到最大 token 长度）。
  
- **usage**: token 使用情况，包含 `prompt_tokens`（提示 token 数量）、`completion_tokens`（生成内容 token 数量）、`total_tokens`（总 token 数量）。


#### 2. File Chat API

**端点**: `POST /v1/chat/completions`

**（1）请求体:**  
与对话共用一个api，当有file_id时表示使用文件对话。

  ```json
  {
    ...
    "file_id": "2620346334574bb09ca73edcecc847d5",
    ...
  }
  ```

请求参数解释：

- **file_id**: （作为选填项，在文件对话中必须填）在文件上传api中获取到的id。


**（2）响应体:**

```json
// 非流式返回
{
    "choices": [
        {
            "finish_reason": "stop",
            "index": 0,
            "message": {
                "content": "李刚，张三，赵四。",
                "role": "assistant"
            }
        }
    ],
    "created": 1729065056,
    "id": "chatchat-12345678",
    "model": "chatchat",
    "object": "chat.completion",
    "usage": {
        "completion_tokens": 0,
        "prompt_tokens": 0,
        "total_tokens": 0
    }
}

// 流式返回
data: {
	"choices": [{
		"delta": {
			"role": "assistant"
		},
		"finish_reason": "",
		"index": 0
	}],
	"id": "chatchat-12345678",
	"model": "chatchat"
}

data: {
	"id": "chatchat-12345678",
	"model": "chatchat",
	"choices": [{
		"index": 0,
		"delta": {
			"content": "李"
		},
		"finish_reason": ""
	}]
}

...

data: {
	"choices": [{
		"delta": {},
		"finish_reason": "stop",
		"index": 0
	}],
	"created": 1729065646,
	"id": "chatchat-12345678",
	"model": "chatchat",
	"object": "chat.completion.chunk"
}

data: [DONE]
```

响应参数解释：
- 与对话api一致。


## 二、chatchat API测试

#### 1. 知识库对话

**（1）请求体**

```bash
curl -X POST http://127.0.0.1:7861/chat/kb_chat -H "Content-Type: application/json" -d '{
  "query": "恼羞成怒",
  "mode": "local_kb",
  "kb_name": "samples",
  "top_k": 3,
  "score_threshold": 2,
  "history": [
    {
      "content": "我们来玩成语接龙，我先来，生龙活虎",
      "role": "user"
    },
    {
      "content": "虎头虎脑",
      "role": "assistant"
    }
  ],
  "stream": false,
  "model": "qwen1.5-chat",
  "temperature": 0.7,
  "max_tokens": 0,
  "prompt_name": "default",
  "return_direct": false
}'

```



**（2）响应体**

```json
// 当stream设置为false的时候
{
    "id": "chat4fbd7892-0c7d-4b72-af3f-b306390fc545",
    "object": "chat.completion",
    "model": "Qwen1.5-7B-CUDT",
    "created": 1726302693,
    "status": null,
    "message_type": 1,
    "message_id": null,
    "is_ref": false,
    "choices": [
        {
            "message": {
                "role": "assistant",
                "content": "To develop an application, you need to have a good understanding of programming concepts, choose a suitable programming language, learn its syntax and best practices, and then start writing your code. It's also important to utilize libraries and frameworks to make development easier and efficient. Once you've written your code, you will need to test it thoroughly to ensure it functions correctly and meets all requirements before deploying it.",
                "finish_reason": null,
                "tool_calls": []
            }
        }
    ]
}

{
    "id": "chat74dcf3db-bf3a-492d-b2d9-ca8ced55b409",
    "object": "chat.completion.chunk",
    "model": "Qwen1.5-7B-CUDT",
    "created": 1727249138,
    "status": null,
    "message_type": 1,
    "message_id": null,
    "is_ref": false,
    "choices": [
        {
            "delta": {
                "content": " application,",
                "tool_calls": [

                ]
            },
            "role": "assistant"
        }
    ]
}

// 当stream设置为true的时候
{
    "id": "chat15cb427b-ef73-4c09-9f7c-3c391b7f555e",
    "object": "chat.completion.chunk",
    "model": "Qwen1.5-7B-CUDT",
    "created": 1727263671,
    "status": null,
    "message_type": 1,
    "message_id": null,
    "is_ref": false,
    "docs": [String],
    "choices": [
        {
            "delta": {
                "content": "",
                "tool_calls": [

                ]
            },
            "role": "assistant"
        }
    ]
}

{
    "id": "chat74dcf3db-bf3a-492d-b2d9-ca8ced55b409",
    "object": "chat.completion.chunk",
    "model": "Qwen1.5-7B-CUDT",
    "created": 1727249138,
    "status": null,
    "message_type": 1,
    "message_id": null,
    "is_ref": false,
    "choices": [
        {
            "delta": {
                "content": " application,",
                "tool_calls": [

                ]
            },
            "role": "assistant"
        }
    ]
}
```

响应参数解释：

- **id**: 每次生成的回复的唯一标识符。
- **object**: 响应对象类型，如 `"chat.completion"`。
- **model**: 使用的模型名称。
- **created**: 响应的生成时间戳。
- **status**: 
- **message_type**: 1, 
- **message_id**: null, 
- **is_ref**: false, 
- **choices:** 返回的生成文本选项列表。

  - `message`: 包含生成的消息，`role` 为 `"assistant"`，`content` 是生成的回复内容。
  - `finish_reason`: 生成结束的原因，如 `stop`（正常结束）、`length`（达到最大 token 长度）。



## 三、EulerCopilot

**framework**:

集群内访问的IP：http://framework-service-llm.euler-copilot.svc.cluster.local:8002

集群外访问的IP：http://10.192.128.28:30690



#### 1. Create Api key

**（1）请求体**

```bash
curl -X POST http://10.192.128.28:30690/authorize/api_key\ 
-H "Accept: application/json"\
-H "X-CSRF-Token: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VyX3N1YiI6IkNvcGlsb3QiLCJleHAiOjE3MjY3MzYxNDgsIl91IjoiZDRiOWQyYjItZDYzNS00M2YwLWFlMzgtZjI1Yzg2MmIyMzZmIn0.VaVcx49ISyA0L2RZ8OlPGU6wStJ6s-KnSigSadnnrl8"

curl -X POST http://10.192.128.28:30690/authorize/api_key/revoke\
-H "Accept: application/json"
```

**（2）响应体**

```bash
# 没有curl通
```



#### 2. Login

Login | 登录

**（1）请求体**

```bash
curl -X POST http://10.192.128.28:30690/authorize/login \
-H "Accept: application/json" \
-H "Content-Type: application/json" \
-d '{
    "account": "Copilot",
    "passwd": "Copilot"
}'
```

**（2）响应体**

```json
{
    "code": 200,
    "message": "success",
    "result": {
        "csrf_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VyX3N1YiI6IkNvcGlsb3QiLCJleHAiOjE3MjY3MzYxNDgsIl91IjoiZDRiOWQyYjItZDYzNS00M2YwLWFlMzgtZjI1Yzg2MmIyMzZmIn0.VaVcx49ISyA0L2RZ8OlPGU6wStJ6s-KnSigSadnnrl8"
    }
}
```



#### 3. 自然语言对话

Natural Language Post | 自然语言对话(向大模型提问)

**（1）请求体**

```bash
# 在rag的pod内：http://localhost:8005/kb/get_answer
# 直接curl：http://10.192.128.28:32230/kb/get_answer
curl -k -X POST "http://10.192.128.28:32230/kb/get_answer" \
-H "Accept: application/json" \
-H "Content-Type: application/json" \
-d '{
    "question": "who are you?",
    "kb_sn": "default test",
    "session_id": "",
    "qa_record_id": "",
    "fetch_source":true,
    "user_selected_plugins": [
        {
            "plugin_name": "",
            "plugin_auth": ""
        }
    ]
}'

curl -k -X POST "http://10.192.128.28:32230/kb/get_answer" -H "Content-Type: application/json" -d '{
	"question":"你是谁？",
	"kb_sn": "default test" 
}'
```

**（2）响应体**

```rust
{
    "answer": "我叫NeoCopilot，是openEuler社区的助手。",
    "sources": [],
    "source_contents": [],
    "scores": null
}
```





#### 4. get plugin

（1）请求体

```bash
curl -X GET http://10.192.128.28:30690/plugin\
-H "Accept: application/json"
```

**（2）响应体**

```bash
# 没有curl通
```









































