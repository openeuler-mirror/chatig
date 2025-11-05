# generate_sdk/test_generate.py
from __future__ import annotations
import os
import sys
import time
from dataclasses import dataclass
from typing import List, Dict, Any, Optional

from generate_sdk.generate_config import GenerateConfig
from generate_sdk.client import GenerateClient


@dataclass
class TestResult:
    name: str
    ok: bool
    detail: str = ""


class ConversationTester:
    """
    一个极简会话测试器：
    - 维护 messages 历史，使用 GenerateClient.generate()/generate_stream()
    - 提供若干用例：单轮、多轮、流式
    """

    def __init__(self, cli: GenerateClient, system_prompt: Optional[str] = None):
        self.cli = cli
        self.messages: List[Dict[str, str]] = []
        if system_prompt:
            self.messages.append({"role": "system", "content": system_prompt})

    def ask(self, user_text: str, **kwargs) -> str:
        """非流式提问，自动记录对话历史。"""
        self.messages.append({"role": "user", "content": user_text})
        resp = self.cli.generate(messages=self.messages, **kwargs)
        if not resp.choices or not resp.choices[0].message:
            raise RuntimeError("响应不含有效的 assistant 消息")
        answer = resp.choices[0].message.content or ""
        self.messages.append({"role": "assistant", "content": answer})
        # 打印可选 usage
        if resp.usage:
            print(f"[usage] prompt={resp.usage.prompt_tokens} "
                  f"completion={resp.usage.completion_tokens} "
                  f"total={resp.usage.total_tokens}")
        return answer

    def ask_stream(self, user_text: str, **kwargs) -> str:
        """流式提问，逐块打印，自动记录历史。"""
        self.messages.append({"role": "user", "content": user_text})
        print("[stream] ", end="", flush=True)
        chunks: List[str] = []
        for piece in self.cli.generate_stream(messages=self.messages, **kwargs):
            print(piece, end="", flush=True)
            chunks.append(piece)
        print()  # 换行
        answer = "".join(chunks)
        # 记录 assistant 回复
        self.messages.append({"role": "assistant", "content": answer})
        return answer


def run_health_check(cli: GenerateClient) -> TestResult:
    ok = cli.health_check()
    return TestResult("健康检查", ok, "服务端 /health 未通过" if not ok else "ok")


def run_single_turn(cli: GenerateClient) -> TestResult:
    tester = ConversationTester(cli, system_prompt="你是一个简洁、可靠的助手。")
    try:
        text = tester.ask("用一句话介绍一下你自己", temperature=0.7, max_tokens=128)
        ok = isinstance(text, str) and len(text.strip()) > 0
        return TestResult("单轮对话可用性", ok, text if ok else "空响应")
    except Exception as e:
        return TestResult("单轮对话可用性", False, str(e))


def run_multi_turn_memory(cli: GenerateClient) -> TestResult:
    tester = ConversationTester(cli, system_prompt="请尽量在上下文中保持一致。")
    try:
        _ = tester.ask("从现在开始记住：我的名字叫小李。请只回复“已记住”。", temperature=0)
        ans = tester.ask("我叫什么名字？", temperature=0)
        ok = "小李" in ans
        detail = f"模型回答：{ans}"
        return TestResult("多轮对话-上下文记忆", ok, detail)
    except Exception as e:
        return TestResult("多轮对话-上下文记忆", False, str(e))


def run_streaming(cli: GenerateClient) -> TestResult:
    tester = ConversationTester(cli, system_prompt="请分点作答。")
    try:
        ans = tester.ask_stream("用三点概述你能做什么", temperature=0.7, max_tokens=256)
        ok = isinstance(ans, str) and len(ans.strip()) > 0
        return TestResult("流式增量生成", ok, "ok" if ok else "空响应")
    except Exception as e:
        return TestResult("流式增量生成", False, str(e))


def run_stop_and_sampling(cli: GenerateClient) -> TestResult:
    tester = ConversationTester(cli)
    try:
        # 示例：设置 stop，期待在“END”处尽快停止
        ans = tester.ask("连续输出三个词，每个词后面跟一个逗号，最后输出 END。",
                         temperature=0.2, top_p=0.95, max_tokens=64, stop=["END"])
        ok = "END" not in ans  # 被 stop 掉才算预期内
        return TestResult("采样/停止词", ok, f"模型回答：{ans}")
    except Exception as e:
        return TestResult("采样/停止词", False, str(e))


def main():
    # 允许用环境变量覆盖配置，便于 CI/多环境
    base_url = os.getenv("CHATIG_BASE_URL", "http://127.0.0.1:8001")
    api_key = os.getenv("CHATIG_API_KEY")  # 可选
    timeout = float(os.getenv("CHATIG_TIMEOUT", "60"))

    cfg = GenerateConfig(
        base_url=base_url,
        api_key=api_key,
        timeout=timeout,
        # 如果需要额外头（例如 x-api-key），放开这行：
        # extra_headers={"x-api-key": "your-key"}
    )

    print("=== ChatIG Generate SDK 多轮对话检测 ===")
    print(f"[cfg] base_url={cfg.base_url} generate={cfg.get_generate_url()} stream={cfg.get_generate_stream_url()}")
    if api_key:
        print("[cfg] api_key: (set)")
    print("=" * 60)

    results: List[TestResult] = []
    with GenerateClient(cfg) as cli:
        # 1. 健康检查
        r = run_health_check(cli)
        print(f"[{r.name}] {'✅' if r.ok else '❌'} {r.detail}")
        results.append(r)

        # 2. 单轮
        r = run_single_turn(cli)
        print(f"[{r.name}] {'✅' if r.ok else '❌'} {r.detail}")
        results.append(r)

        # 3. 多轮上下文记忆
        r = run_multi_turn_memory(cli)
        print(f"[{r.name}] {'✅' if r.ok else '❌'} {r.detail}")
        results.append(r)

        # 4. 流式
        r = run_streaming(cli)
        print(f"[{r.name}] {'✅' if r.ok else '❌'} {r.detail}")
        results.append(r)

        # 5. 采样/停止词
        r = run_stop_and_sampling(cli)
        print(f"[{r.name}] {'✅' if r.ok else '❌'} {r.detail}")
        results.append(r)

    # 汇总
    passed = sum(1 for x in results if x.ok)
    total = len(results)
    print("\n=== 汇总 ===")
    for x in results:
        print(f"{'✅' if x.ok else '❌'} {x.name}")
    print(f"总计: {passed}/{total} 通过")
    if passed != total:
        sys.exit(1)


if __name__ == "__main__":
    main()
