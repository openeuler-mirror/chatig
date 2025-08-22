"""
Chatig 重排序 SDK 的简单测试

本模块为重排序SDK功能提供基本测试。
"""

import sys
import os

# 将当前目录添加到路径中以导入SDK模块
current_dir = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, current_dir)

# 使用绝对导入
from rerank_client import RerankClient
from config import RerankConfig
from models import RerankEngineType
from exceptions import RerankError, RerankValidationError


def test_health_check():
    """测试健康检查功能"""
    print("测试健康检查...")
    
    client = RerankClient()
    try:
        is_healthy = client.health_check()
        print(f"健康检查结果: {is_healthy}")
        return is_healthy
    except Exception as e:
        print(f"健康检查失败: {e}")
        return False
    finally:
        client.close()


def test_std_rerank():
    """测试标准重排序功能"""
    print("\n测试标准重排序...")
    
    client = RerankClient()
    
    # 测试数据
    documents = [
        "敏捷的棕色狐狸跳过懒狗。",
        "一只懒狗在阳光下睡觉。",
        "狐狸敏捷且棕色。",
        "狗是忠诚的伙伴。"
    ]
    query = "敏捷狐狸"
    
    try:
        response = client.rerank_std(
            model="bge-reranker-v2-m3",
            query=query,
            documents=documents,
            top_n=2
        )
        
        print(f"查询: {query}")
        print("结果:")
        print(f"  索引: {response.results.index}, 分数: {response.results.score:.4f}")
        
        return True
        
    except Exception as e:
        print(f"标准重排序失败: {e}")
        return False
    finally:
        client.close()


def test_vllm_rerank():
    """测试VLLM重排序功能"""
    print("\n测试VLLM重排序...")
    
    client = RerankClient()
    
    # 测试数据
    documents = [
        "机器学习是人工智能的一个子集。",
        "深度学习使用多层神经网络。",
        "自然语言处理帮助计算机理解文本。",
        "计算机视觉使机器能够解释视觉信息。"
    ]
    query = "神经网络"
    
    try:
        response = client.rerank_vllm(
            model="llama-2-7b-chat",
            query=query,
            documents=documents
        )
        
        print(f"查询: {query}")
        print(f"模型: {response.model}")
        print(f"总令牌数: {response.usage.total_tokens}")
        print("结果:")
        for result in response.results:
            print(f"  索引: {result.index}, 分数: {result.relevance_score:.4f}")
            print(f"  文档: {result.document.text}")
        
        return True
        
    except Exception as e:
        print(f"VLLM重排序失败: {e}")
        return False
    finally:
        client.close()


def test_llamabox_rerank():
    """测试LlamaBox重排序功能"""
    print("\n测试LlamaBox重排序...")
    
    client = RerankClient()
    
    # 测试数据
    documents = [
        "Python是一种高级编程语言。",
        "JavaScript用于网络开发。",
        "Java是一种面向对象的编程语言。",
        "C++是一种强大的系统编程语言。"
    ]
    query = "网络开发"
    
    try:
        response = client.rerank_llamabox(
            model="llama-2-7b-chat",
            query=query,
            documents=documents,
            top_n=3
        )
        
        print(f"查询: {query}")
        print(f"模型: {response.model}")
        if response.usage:
            print(f"提示令牌数: {response.usage.prompt_tokens}")
            print(f"总令牌数: {response.usage.total_tokens}")
        print("前3个结果:")
        for i, result in enumerate(response.results[:3]):
            print(f"  {i+1}. 分数: {result.relevance_score:.4f}")
            print(f"     文档: {result.document.text}")
        
        return True
        
    except Exception as e:
        print(f"LlamaBox重排序失败: {e}")
        return False
    finally:
        client.close()


def test_error_handling():
    """测试错误处理"""
    print("\n测试错误处理...")
    
    client = RerankClient()
    
    try:
        # 测试验证错误
        response = client.rerank_std(
            model="",
            query="",
            documents=[]
        )
        print("验证错误测试失败 - 应该抛出异常")
        return False
        
    except RerankValidationError as e:
        print(f"✅ 捕获验证错误: {e}")
        return True
    except Exception as e:
        print(f"❌ 意外错误: {e}")
        return False
    finally:
        client.close()


def test_configuration():
    """测试配置功能"""
    print("\n测试配置...")
    
    # 测试自定义配置
    config = RerankConfig(
        base_url="http://localhost:8000",
        timeout=60.0,
        max_retries=5
    )
    
    print(f"基础URL: {config.base_url}")
    print(f"超时时间: {config.timeout}")
    print(f"最大重试次数: {config.max_retries}")
    
    # 测试URL生成
    std_url = config.get_rerank_url("std")
    vllm_url = config.get_rerank_url("vllm")
    llamabox_url = config.get_rerank_url("llamabox")
    
    print(f"STD URL: {std_url}")
    print(f"VLLM URL: {vllm_url}")
    print(f"LlamaBox URL: {llamabox_url}")
    
    return True


def main():
    """运行所有测试"""
    print("Chatig 重排序 SDK 测试")
    print("=" * 50)
    
    tests = [
        ("健康检查", test_health_check),
        ("配置", test_configuration),
        ("标准重排序", test_std_rerank),
        ("VLLM重排序", test_vllm_rerank),
        ("LlamaBox重排序", test_llamabox_rerank),
        ("错误处理", test_error_handling),
    ]
    
    results = []
    
    for test_name, test_func in tests:
        print(f"\n{'='*20} {test_name} {'='*20}")
        try:
            result = test_func()
            results.append((test_name, result))
        except Exception as e:
            print(f"测试 {test_name} 因异常失败: {e}")
            results.append((test_name, False))
    
    # 总结
    print("\n" + "=" * 50)
    print("测试结果总结:")
    print("=" * 50)
    
    passed = 0
    total = len(results)
    
    for test_name, result in results:
        status = "✅ 通过" if result else "❌ 失败"
        print(f"{test_name}: {status}")
        if result:
            passed += 1
    
    print(f"\n总计: {passed}/{total} 个测试通过")
    
    if passed == total:
        print("🎉 所有测试通过！")
    else:
        print("⚠️  部分测试失败。请检查上面的输出。")


if __name__ == "__main__":
    main() 