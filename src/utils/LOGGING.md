# ChatIG 会话日志功能使用说明

> 本文介绍 ChatIG 项目中的“**会话日志（Conversation Log）**”功能：如何启用、配置、写入内容、查询方式、测试验证、常见问题与最佳实践。

---

## 功能概览

- **写入位置**：将每次 `/v1/chat/completions` 请求的关键信息以 **NDJSON（每行一个 JSON）** 形式追加写入到配置文件指定的路径（默认：`/var/log/chatig/conversation.log`）。  
- **记录内容**：
  - `conversation_id`：会话 ID（优先取请求头 `x-conversation-id`，否则自动生成 UUID）。
  - `input`：本次请求中**最后一条 user 消息**（结构化 content 会序列化为字符串）。
  - `output`：非流式为模型返回文本；流式 SSE 时为 `"<streaming>"`。
  - `start_time` / `end_time`：UTC 时间戳。
  - `duration_ms`：从进入 handler 到生成响应的耗时（毫秒）。
  - `rounds`：本次请求内 user 消息计数（粗略视为轮数）。
  - `history`：本次请求的完整 `messages`，序列化为 JSON，便于回溯上下文。
- **查询接口（可选）**：`GET /logs/query?conversation_id=<id>` 返回该会话的全部日志行（需按本文启用）。

---

## 1. 配置说明

### 1.1 应用配置（非 log4rs）

在 **`configs/configs.yaml`** 中新增/确认 `logging` 段：

```yaml
logging:
  enable_conversation_log: true
  conversation_log_path: "/var/log/chatig/conversation.log"
```

- `enable_conversation_log`：是否启用会话日志写入。
- `conversation_log_path`：会话日志文件路径（建议放到独立目录，如 `/var/log/chatig`）。

> ⚠️ 注意：**不要**把这个 `logging:` 段放到 log4rs 的 YAML 里！log4rs 只识别 `refresh_rate`/`root`/`appenders`/`loggers` 四类键。

### 1.2 log4rs 日志（access/error）区分

- 应用访问/错误日志由 **log4rs** 管理，配置文件如 `configs/log4rs.yaml`。  
- 会话日志是**业务日志**，与 log4rs 无关，路径由 `logging.conversation_log_path` 控制。

### 1.3 运行时覆盖配置（可选）

- 环境变量 `CHATIG_CONFIG` 指定应用配置文件（包含上面的 `logging:` 段）。  
- 环境变量 `CHATIG_LOG4RS` 指定 log4rs 配置文件。

示例：
```bash
CHATIG_CONFIG=/opt/chatig/configs/configs.yaml \
CHATIG_LOG4RS=/opt/chatig/configs/log4rs.yaml \
./chatig
```

---

## 2. 写入内容与格式

### 2.1 数据模型

Rust 结构体（`utils/convlog.rs`）示例：

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConversationLog {
    pub conversation_id: String,
    pub input: String,
    pub output: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration_ms: i64,
    pub rounds: u32,
    pub history: Option<serde_json::Value>, // 可选
}
```

### 2.2 示例日志行（NDJSON）

```json
{"conversation_id":"conv-001","input":"\"你好，帮我写一首五言绝句。\"","output":"松风吹古道，远水映青山……","start_time":"2025-09-02T08:00:01.123Z","end_time":"2025-09-02T08:00:01.456Z","duration_ms":333,"rounds":1,"history":[{"role":"user","content":"你好，帮我写一首五言绝句。"}]}
```

> 提示：若 `content` 是结构化对象（如 `{type:"text", text:"..."}`），`input` 会是 **JSON 字符串**（被引号包裹）。

### 2.3 流式与错误场景

- **流式（SSE）**：响应头 `Content-Type: text/event-stream` 时，`output` 记录为 `"<streaming>"`，避免截断推流。  
- **错误分支**：当底层模型报错时，`output` 记录为 `"ERROR: ..."`，其余字段照常写入。

---

## 3. 启用与集成（代码位置）

> 以下说明对应你当前合并的实现：

- **写入入口**：`apis/models_api/chat.rs`
  - 在成功和错误分支里，调用 `append_conv_log(state.clone(), conv_log).await;`。
  - `conv_log` 含本次请求的 `conversation_id/input/output/time/duration/rounds/history`。
- **工具函数**：`utils/convlog.rs`
  - 负责将一行 JSON 追加到 `conversation_log_path`；自动创建目录；使用 `spawn_blocking` 避免阻塞异步线程。
- **状态注入**：`main.rs`
  - `AppState { cfg: Config }` 通过 `.app_data(web::Data::new(app_state.clone()))` 注入 Actix 应用。

---

## 4. 查询接口（可选）

若需要通过 HTTP 查询日志（按会话 ID 检索），添加：

- `src/apis/logs.rs`
- 在 `main.rs` 注册：`.configure(|cfg| apis::logs::configure(cfg))`

接口用法：
```
GET /logs/query?conversation_id=conv-001
```

返回示例：
```json
[
  { "conversation_id":"conv-001", "...": "..." },
  { "conversation_id":"conv-001", "...": "..." }
]
```

---

## 5. 权限与目录

创建日志目录并赋予运行用户写权限：
```bash
sudo mkdir -p /var/log/chatig
sudo chown -R $(whoami) /var/log/chatig
```

> 若以 systemd 运行，请将目录权限授予服务用户（例如 `chatig`）。

---

## 6. 本地测试清单

### 6.1 健康检查
```bash
curl -sS http://127.0.0.1:8001/v1/chat/health
# 预期：OK
```

### 6.2 非流式请求 → 写日志
```bash
curl -sS http://127.0.0.1:8001/v1/chat/completions \
-H 'Content-Type: application/json' \
-d '{
"model":"Qwen3-0.6B",
"messages":[{"role":"user","content":"你好，帮我写一首五言绝句。"}],
"stream": false
}' | jq .


sudo tail -n 5 /var/log/chatig/conversation.log | jq .
```

### 6.3 流式请求（如支持）
```bash
curl -N http://127.0.0.1:8001/v1/chat/completions \
-H 'Content-Type: application/json' \
-d '{
"model":"Qwen3-0.6B",
"messages":[{"role":"user","content":"请分三行输出A/B/C"}],
"stream": true
}'
# 预期：日志里 output 为 "<streaming>"
```

### 6.4 错误分支
```bash
curl -sS -i \
  -H 'Content-Type: application/json' \
  -X POST $BASE/v1/chat/completions \
  --data '{"model":"FooBar/Whatever","messages":[{"role":"user","content":"test"}]}'
# 预期：HTTP 400；日志 output 以 "ERROR:" 开头
```

### 6.5 开关验证
```bash
# 将 configs/configs.yaml 中 enable_conversation_log 设为 false，重启服务
wc -l /var/log/chatig/conversation.log    # 预期：计数不再增长
```

---

## 7. 日志查看与分析

### 7.1 快速查看
```bash
tail -f /var/log/chatig/conversation.log
```

### 7.2 按会话 ID 聚合
```bash
CID=conv-001
grep "\"conversation_id\":\"$CID\"" /var/log/chatig/conversation.log | jq .
```

### 7.3 提取字段（jq）
```bash
# 按时间提取 input/output 概览
jq -r '.start_time + " | " + .conversation_id + " | IN: " + (.input|tostring) + " | OUT: " + (.output|tostring)' \
  /var/log/chatig/conversation.log | tail -50
```

---

## 8. 生产建议与最佳实践

- **日志切割**：使用系统 `logrotate`（示例：`/etc/logrotate.d/chatig-conv`）：
  ```
  /var/log/chatig/conversation.log {
      daily
      rotate 14
      compress
      missingok
      notifempty
      copytruncate
      dateext
  }
  ```
- **鉴权与隐私**：`/logs/query` 建议仅内网开放或加鉴权；如有敏感数据（PII），请在写入前做脱敏。
- **磁盘空间**：关注日志增长速度；必要时调整切割频率或写入策略。
- **高并发**：当前采用 `spawn_blocking` + 逐行追加，能支撑常见并发；极端场景可引入单线程落盘/异步通道缓冲。
- **统一追踪**：上游可统一传入 `x-conversation-id` 以便跨服务追踪一段对话。

---

## 9. 常见问题（FAQ）

**Q1：为什么 log4rs 报 `unknown field logging`？**  
A：把 `logging:`（会话日志配置）错放到了 log4rs 文件里。应放回应用配置 `configs.yaml`。

**Q2：为何 `input` 有时带引号？**  
A：当 `messages[i].content` 是结构化对象时，会被序列化成 JSON 字符串保存。

**Q3：为何流式 `output` 是 `"<streaming>"`？**  
A：为避免截断 SSE 推流。若需要完整输出，建议在流式实现里旁路汇总 delta 再在结束时写入。

**Q4：如何只针对特定服务/用户写日志？**  
A：在 `apis/models_api/chat.rs` 写入前增加条件判断（例如根据 service_name/user_id 过滤）。

**Q5：能否同时写入数据库？**  
A：可以。在 `append_conv_log` 旁边新增 `append_conv_log_db` 同步写 DB 即可；字段沿用 `ConversationLog`。

---

## 10. 版本记录

- `v1`：追加写文件、非流式抓取 body、流式占位、错误分支记录、可选 `history` 字段。
- `vNext`（规划）：流式完整输出聚合、更多查询维度、DB 索引与分析报表。

---

