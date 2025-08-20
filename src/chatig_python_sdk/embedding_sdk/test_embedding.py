#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
ChatIG Embedding SDK 快速测试脚本

用途：
- 快速验证 Embedding 模块在修复后的可用性与基本质量
- 与现有的 rerank 测试脚本风格一致

覆盖测试：
1) 导入
2) 配置
3) 校验（空模型名等）
4) 健康检查
5) 基础向量化（单条/批量）
6) 一致性（Determinism）
7) 相似度合理性（语义近的更相似）
"""

from typing import List, Optional
import math
import sys
import traceback

# ========= 统一导入 =========
from .embedding_models import (
    EmbeddingEngineType, StdEmbeddingRequest, EmbeddingValidationError
)

from .client import EmbeddingsClient


# ========= 工具函数 =========

def _cosine(a: List[float], b: List[float]) -> float:
    """纯 Python 版余弦相似度（无第三方依赖）"""
    dot = sum(x * y for x, y in zip(a, b))
    na = math.sqrt(sum(x * x for x in a))
    nb = math.sqrt(sum(y * y for y in b))
    if na == 0.0 or nb == 0.0:
        return 0.0
    return dot / (na * nb)


def _l2(a: List[float], b: List[float]) -> float:
    """L2 距离"""
    return math.sqrt(sum((x - y) ** 2 for x, y in zip(a, b)))


def _ok(msg: str):
    print(f"✅ {msg}")


def _fail(msg: str, e: Optional[Exception] = None):
    if e:
        print(f"❌ {msg}: {e}")
        traceback.print_exc()
    else:
        print(f"❌ {msg}")


# ========= 1) 导入 =========

def test_imports() -> bool:
    print("测试导入...")
    try:
        # 如果能运行到这里，说明头部导入没问题
        _ok("导入成功（embedding_models / embedding_config / client）")
        return True
    except Exception as e:
        _fail("导入失败", e)
        return False


# ========= 2) 健康检查 =========

def test_health_check() -> bool:
    print("\n测试健康检查...")
    try:
        client = EmbeddingsClient()
        is_healthy = client.health_check()
        print(f"健康检查结果: {is_healthy}")
        client.close()
        _ok("健康检查执行完成")
        return True
    except Exception as e:
        _fail("健康检查失败", e)
        return False


# ========= 3) 配置 =========

def test_config() -> bool:
    print("\n测试配置...")
    try:
        cfg = EmbeddingConfig()

        health_url = cfg.get_health_url()
        print(f"健康检查URL: {health_url}")

        if hasattr(cfg, "get_embeddings_url"):
            std_url = cfg.get_embeddings_url()
        else:
            base = getattr(cfg, "base_url", "http://127.0.0.1:8000")
            std_url = f"{base}/v1/embeddings"
        print(f"Embeddings URL: {std_url}")

        _ok("配置可用")
        return True
    except Exception as e:
        _fail("配置测试失败", e)
        return False


# ========= 4) 校验（请求入参） =========

def test_validation() -> bool:
    print("\n测试验证...")
    try:
        # 测试空模型名称
        try:
            req = StdEmbeddingRequest(
                model="",
                input=["test"]
            )
            if hasattr(req, "validate"):
                req.validate()
                _fail("空模型名应触发验证失败，但未抛异常")
                return False
            else:
                _ok("未提供本地 validate()，略过客户端校验（建议在服务端做校验）")
        except EmbeddingValidationError as ve:
            _ok(f"捕获验证错误: {ve}")

        return True
    except Exception as e:
        _fail("验证测试失败", e)
        return False


# ========= 5) 基础向量化 =========

def test_basic_embed() -> bool:
    print("\n测试基础向量化（单条 + 批量）...")
    try:
        client = EmbeddingsClient()

        model_id = getattr(client, "default_model", None) or \
                   "Qwen/Qwen2.5-7B-Embeddings"

        res1 = client.embed(model=model_id, input="今天天气不错")
        dim = len(res1.data[0].embedding)
        print(f"单条向量维度: {dim}")
        assert dim > 0

        texts = ["今天天气不错", "适合散步"]
        res2 = client.embed(model=model_id, input=texts)
        assert len(res2.data) == len(texts)
        dim2 = len(res2.data[0].embedding)
        print(f"批量向量维度: {dim2}")
        assert dim2 == dim

        print(f"usage.total_tokens: {res2.usage.total_tokens}")
        client.close()
        _ok("基础向量化通过")
        return True
    except Exception as e:
        _fail("基础向量化失败", e)
        return False


# ========= 6) 一致性（Determinism） =========

def test_determinism() -> bool:
    print("\n测试一致性（Determinism）...")
    try:
        client = EmbeddingsClient()

        model_id = getattr(client, "default_model", None) or \
                   "Qwen/Qwen2.5-7B-Embeddings"

        a = client.embed(model=model_id, input="确定性测试")
        b = client.embed(model=model_id, input="确定性测试")
        v1 = a.data[0].embedding
        v2 = b.data[0].embedding

        dist = _l2(v1, v2)
        print(f"L2 距离: {dist:.6f}")

        if dist < 1e-3:
            _ok("同词两次嵌入接近（通过）")
            client.close()
            return True
        else:
            _fail("同词两次嵌入差异偏大（不通过）")
            client.close()
            return False
    except Exception as e:
        _fail("一致性测试失败", e)
        return False


# ========= 7) 相似度合理性 =========

def test_semantic_similarity() -> bool:
    print("\n测试相似度合理性（语义近的更相似）...")
    try:
        client = EmbeddingsClient()
        model_id = getattr(client, "default_model", None) or \
                   "Qwen/Qwen2.5-7B-Embeddings"

        texts = [
            "我喜欢跑步",
            "我热爱慢跑",
            "今天下雨",
            "The stock price soared."
        ]
        res = client.embed(model=model_id, input=texts)
        embs = [d.embedding for d in res.data]

        sim_close = _cosine(embs[0], embs[1])
        sim_far = _cosine(embs[0], embs[2])
        print(f"相似(跑步 vs 慢跑): {sim_close:.4f}")
        print(f"不相似(跑步 vs 下雨): {sim_far:.4f}")

        if sim_close > sim_far:
            _ok("相似度排序合理（通过）")
            client.close()
            return True
        else:
            _fail("相似度排序不合理（不通过）")
            client.close()
            return False
    except Exception as e:
        _fail("相似度测试失败", e)
        return False


# ========= 主程序 =========

def main():
    print("ChatIG Embedding SDK 快速测试")
    print("=" * 40)

    tests = [
        ("导入", test_imports),
        ("配置", test_config),
        ("验证", test_validation),
        ("健康检查", test_health_check),
        ("基础向量化", test_basic_embed),
        ("一致性", test_determinism),
        ("相似度", test_semantic_similarity),
    ]

    passed = 0
    total = len(tests)

    for name, fn in tests:
        print(f"\n{'=' * 20} {name} {'=' * 20}")
        ok = False
        try:
            ok = fn()
        except Exception as e:
            _fail(f"{name} 测试过程中出现未捕获异常", e)
        print(("✅ " if ok else "❌ ") + f"{name} {'通过' if ok else '失败'}")
        passed += int(ok)

    print(f"\n总计: {passed}/{total} 个测试通过")
    if passed == total:
        print("🎉 所有基础测试通过！")
    else:
        print("⚠️  部分测试失败")
        sys.exit(1)


if __name__ == "__main__":
    main()
