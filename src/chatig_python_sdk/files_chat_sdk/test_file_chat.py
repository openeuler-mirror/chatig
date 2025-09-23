# file_chat_sdk/test_file_chat.py
from __future__ import annotations
import os
from typing import List
from .file_chat_config import FileChatConfig
from .client import FileChatClient

def main():
    print("File Chat SDK 快速测试")
    print("=" * 40)

    base_url = os.getenv("CHATIG_BASE_URL", "http://127.0.0.1:8001")
    api_key = os.getenv("CHATIG_API_KEY", "sk-e66bfb20df094dfab5c675383ee927d8")  # 可选
    timeout = float(os.getenv("CHATIG_TIMEOUT", "60"))

    cfg = FileChatConfig(base_url=base_url, api_key=api_key, timeout=timeout)
    print(f"[cfg] base_url={cfg.base_url}")
    print(f"[cfg] upload={cfg.get_upload_url()}")
    print(f"[cfg] completions={cfg.get_completions_url()}")

    with FileChatClient(cfg) as cli:
        # 1. 健康检查（可选）
        print("\n[健康检查]")
        print("healthy:", cli.health_check())

        # 2. 上传文件（演示：上传一段内存中的文本当作 .txt）
        print("\n[上传文件]")
        sample_text = "This is a file-chat demo content.\n用于测试文件对话。"
        up = cli.upload_files(
            file_tuples=[
                ("files", sample_text.encode("utf-8"), "demo.txt"),  # field_name 根据后端UploadForm对齐
            ],
            form_fields={"scene": "test"}  # 如果后端需要额外字段，在此传入
        )
        print("upload ok:", up.ok, "doc_ids:", up.doc_ids, "files:", [f.filename for f in (up.files or [])])

        # 3. 基于文件的对话
        print("\n[文件对话 completions]")
        messages = [
            {"role": "user", "content": "这份文档的主旨是什么？请简单概述。"}
        ]
        resp = cli.file_completions(
            model=cfg.default_file_chat_model,
            messages=messages,
            doc_ids=up.doc_ids,         # 将上传得到的文档ID传给后端
            session_id="demo-session",  # 可选
            temperature=0.3,
            max_tokens=256,
        )
        text = resp.choices[0].message.content if resp.choices and resp.choices[0].message else ""
        print("模型回复:", text)

if __name__ == "__main__":
    main()
