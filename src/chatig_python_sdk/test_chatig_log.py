#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import argparse
import json
import os
import sys
import time
import uuid
from typing import Dict, Any, List, Optional

import requests

def read_tail_lines(path: str, max_bytes: int = 2 * 1024 * 1024) -> List[str]:
    """从日志文件末尾读取最多 max_bytes，再按行返回（utf-8 容错解码）"""
    try:
        size = os.path.getsize(path)
    except FileNotFoundError:
        return []
    start = max(0, size - max_bytes)
    with open(path, "rb") as f:
        if start:
            f.seek(start)
        data = f.read()
    return data.decode("utf-8", errors="replace").splitlines()

def find_log_entry_by_marker(log_path: str, marker: str, tail_bytes: int) -> Optional[Dict[str, Any]]:
    """在日志尾部查找包含 marker 的 JSON 行，返回最后一条匹配的 dict"""
    lines = read_tail_lines(log_path, tail_bytes)
    match: Optional[Dict[str, Any]] = None
    for line in lines:
        if marker in line:
            try:
                obj = json.loads(line)
                match = obj  # 继续遍历，取最后一条
            except Exception:
                # 不是 JSON 行就跳过
                pass
    return match

def build_headers(api_key: Optional[str]) -> Dict[str, str]:
    headers = {"Content-Type": "application/json"}
    if api_key:
        headers["Authorization"] = f"Bearer {api_key}"
    return headers

def extract_assistant_text_from_nonstream(resp_json: Dict[str, Any]) -> str:
    # 适配多种返回格式
    try:
        c0 = resp_json.get("choices", [{}])[0]
        msg = c0.get("message") or {}
        if isinstance(msg, dict):
            content = msg.get("content")
            if isinstance(content, str):
                return content
    except Exception:
        pass
    for key in ("output", "text"):
        v = resp_json.get(key)
        if isinstance(v, str):
            return v
    # 最差情况：整个 json 转成字符串
    return json.dumps(resp_json, ensure_ascii=False)

def extract_text_from_sse_line(data_obj: Dict[str, Any]) -> Optional[str]:
    """
    从 SSE 的 data: <json> 中提取 delta 文本:
    - OpenAI Chat: choices[0].delta.content
    - 有些实现一次性：choices[0].message.content
    """
    try:
        ch0 = data_obj.get("choices", [{}])[0]
        delta = ch0.get("delta") or {}
        if "content" in delta and isinstance(delta["content"], str):
            return delta["content"]
        msg = ch0.get("message") or {}
        if "content" in msg and isinstance(msg["content"], str):
            return msg["content"]
    except Exception:
        pass
    return None

def post_nonstream(service_url: str, model: str, messages: List[Dict[str, str]], headers: Dict[str, str]) -> Dict[str, Any]:
    payload = {"model": model, "messages": messages, "stream": False}
    r = requests.post(service_url, headers=headers, data=json.dumps(payload), timeout=60)
    r.raise_for_status()
    return r.json()

def post_stream_sse(service_url: str, model: str, messages: List[Dict[str, str]], headers: Dict[str, str]) -> str:
    payload = {"model": model, "messages": messages, "stream": True}
    # 为了更兼容，Accept 不强制 text/event-stream
    r = requests.post(service_url, headers=headers, data=json.dumps(payload), stream=True, timeout=120)
    r.raise_for_status()

    full_text_parts: List[str] = []
    for line in r.iter_lines(decode_unicode=True):
        if not line:
            continue
        # SSE 格式通常是 "data: {...}" 或 "data: [DONE]"
        if line.startswith("data:"):
            data = line[len("data:"):].strip()
            if data == "[DONE]":
                break
            try:
                obj = json.loads(data)
            except Exception:
                # 不是 JSON，就当作纯文本片段
                full_text_parts.append(data)
                continue
            piece = extract_text_from_sse_line(obj)
            if piece:
                full_text_parts.append(piece)
    return "".join(full_text_parts)

def main():
    parser = argparse.ArgumentParser(description="ChatIG 多轮对话 + 会话日志一致性测试（非流式 & 流式）")
    parser.add_argument("--service-url", default="http://127.0.0.1:8001/v1/chat/completions", help="ChatIG 网关 completions 地址")
    parser.add_argument("--model", default="Qwen3-0.6B", help="模型名（需与服务 active_model 一致）")
    parser.add_argument("--rounds", type=int, default=3, help="每种模式对话轮数")
    parser.add_argument("--log-path", default="/var/log/chatig/conversation.log", help="会话日志文件路径")
    parser.add_argument("--tail-bytes", type=int, default=2*1024*1024, help="从日志尾部读取的最大字节数")
    parser.add_argument("--api-key", default=None, help="（可选）给 8001 网关的 Authorization Bearer")
    parser.add_argument("--prompt-base", default="多轮会话日志测试", help="每轮用户提示的前缀")
    args = parser.parse_args()

    headers = build_headers(args.api_key)
    session_id = f"{time.strftime('%Y%m%d-%H%M%S')}-{uuid.uuid4().hex[:8]}"
    print(f"SERVICE_URL = {args.service_url}")
    print(f"MODEL       = {args.model}")
    print(f"ROUNDS      = {args.rounds}")
    print(f"LOG_PATH    = {args.log_path}")
    print(f"SESSION_ID  = {session_id}\n")

    # -------- 非流式 --------
    print("====== 非流式测试 开始 ======")
    nonstream_msgs: List[Dict[str, str]] = []
    for i in range(1, args.rounds + 1):
        marker = f"[SID:{session_id}][MODE:nonstream][ROUND:{i}]"
        user_text = f"{args.prompt_base}（非流式第{i}轮） {marker}"
        nonstream_msgs.append({"role": "user", "content": user_text})

        try:
            resp_json = post_nonstream(args.service_url, args.model, nonstream_msgs, headers)
        except Exception as e:
            print(f"[非流式 第{i}轮] 请求失败：{e}")
            break

        asst_text = extract_assistant_text_from_nonstream(resp_json)
        # 将助手回复加入上下文，进入下一轮
        nonstream_msgs.append({"role": "assistant", "content": asst_text})

        print(f"非流式回复(第{i}轮)：{asst_text[:120]}{'...' if len(asst_text)>120 else ''}")

        # 从日志匹配并打印 rounds 与 input
        time.sleep(0.3)  # 给日志一点写入时间
        log_obj = find_log_entry_by_marker(args.log_path, marker, args.tail_bytes)
        if log_obj:
            rounds = log_obj.get("rounds")
            input_str = log_obj.get("input")
            print(f'LOG[nonstream][第{i}轮]: rounds={rounds} input={input_str}')
        else:
            print(f'LOG[nonstream][第{i}轮]: 未匹配到日志标记 {marker}（可增大 --tail-bytes 或检查日志权限）')
    print("====== 非流式测试 结束 ======\n")

    # -------- 流式（SSE）--------
    print("====== 流式测试 开始 ======")
    stream_msgs: List[Dict[str, str]] = []
    for i in range(1, args.rounds + 1):
        marker = f"[SID:{session_id}][MODE:stream][ROUND:{i}]"
        user_text = f"{args.prompt_base}（流式第{i}轮） {marker}"
        stream_msgs.append({"role": "user", "content": user_text})

        try:
            asst_stream_text = post_stream_sse(args.service_url, args.model, stream_msgs, headers)
        except Exception as e:
            print(f"[流式 第{i}轮] 请求失败：{e}")
            break

        # 将助手回复加入上下文，进入下一轮
        stream_msgs.append({"role": "assistant", "content": asst_stream_text})

        print(f"流式回复(第{i}轮)：{asst_stream_text[:120]}{'...' if len(asst_stream_text)>120 else ''}")

        # 从日志匹配并打印 rounds 与 input（你当前服务端对流式的 output 会写成 \"<streaming>\"，属预期）
        time.sleep(0.5)
        log_obj = find_log_entry_by_marker(args.log_path, marker, args.tail_bytes)
        if log_obj:
            rounds = log_obj.get("rounds")
            input_str = log_obj.get("input")
            print(f'LOG[stream][第{i}轮]: rounds={rounds} input={input_str}')
        else:
            print(f'LOG[stream][第{i}轮]: 未匹配到日志标记 {marker}（可增大 --tail-bytes 或检查日志权限）')
    print("====== 流式测试 结束 ======")

if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\n中断退出")
