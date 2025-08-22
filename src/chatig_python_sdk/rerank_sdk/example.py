"""
Chatig 重排序 SDK 的使用示例

本模块演示了如何使用重排序SDK进行不同的
重排序操作。
"""

from typing import List
from .rerank_client import RerankClient
from .config import RerankConfig
from .models import RerankEngineType, RerankParameters


def basic_rerank_example():
    """使用标准引擎的基本重排序示例"""
    print("=== 基本重排序示例 ===")
    
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
    
    try:
        # 执行重排序
        response = client.rerank_std(
            model="bge-reranker-v2-m3",
            query=query,
            documents=documents,
            top_n=2
        )
        
        print(f"查询: {query}")
        print("顶部结果:")
        for i, result in enumerate(response.results[:2]):
            print(f"  {i+1}. 分数: {result.score:.4f}, 文档: {documents[result.index]}")
            
    except Exception as e:
        print(f"错误: {e}")
    finally:
        client.close()


def vllm_rerank_example():
    """VLLM重排序示例"""
    print("\n=== VLLM重排序示例 ===")
    
    # 使用自定义配置创建客户端
    config = RerankConfig(
        base_url="http://localhost:8000",
        timeout=60.0
    )
    client = RerankClient(config)
    
    # 定义文档和查询
    documents = [
        "机器学习是人工智能的一个子集。",
        "深度学习使用多层神经网络。",
        "自然语言处理帮助计算机理解文本。",
        "计算机视觉使机器能够解释视觉信息。"
    ]
    query = "神经网络"
    
    try:
        # 执行VLLM重排序
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
            
    except Exception as e:
        print(f"错误: {e}")
    finally:
        client.close()


def llamabox_rerank_example():
    """LlamaBox重排序示例"""
    print("\n=== LlamaBox重排序示例 ===")
    
    # 创建客户端
    client = RerankClient()
    
    # 定义文档和查询
    documents = [
        "Python是一种高级编程语言。",
        "JavaScript用于网络开发。",
        "Java是一种面向对象的编程语言。",
        "C++是一种强大的系统编程语言。"
    ]
    query = "网络开发"
    
    try:
        # 执行LlamaBox重排序
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
            
    except Exception as e:
        print(f"错误: {e}")
    finally:
        client.close()


def batch_rerank_example():
    """多个查询的批量重排序示例"""
    print("\n=== 批量重排序示例 ===")
    
    # 创建客户端
    client = RerankClient()
    
    # 定义文档
    documents = [
        "今天天气晴朗。",
        "外面下着大雨。",
        "温度是25摄氏度。",
        "暴风雨即将来临。",
        "天空清澈湛蓝。"
    ]
    
    # 定义多个查询
    queries = [
        "晴朗天气",
        "雨暴风",
        "温度"
    ]
    
    try:
        for query in queries:
            print(f"\n查询: {query}")
            response = client.rerank_std(
                model="bge-reranker-v2-m3",
                query=query,
                documents=documents,
                top_n=2
            )
            
            print("前2个结果:")
            for i, result in enumerate(response.results[:2]):
                print(f"  {i+1}. 分数: {result.score:.4f}, 文档: {documents[result.index]}")
                
    except Exception as e:
        print(f"错误: {e}")
    finally:
        client.close()


def context_manager_example():
    """使用上下文管理器的示例"""
    print("\n=== 上下文管理器示例 ===")
    
    # 定义文档和查询
    documents = [
        "猫在沙发上睡觉。",
        "狗在公园里玩耍。",
        "鸟儿在树上唱歌。",
        "鱼在池塘里游泳。"
    ]
    query = "睡觉的动物"
    
    # 使用上下文管理器进行自动清理
    with RerankClient() as client:
        try:
            response = client.rerank_std(
                model="bge-reranker-v2-m3",
                query=query,
                documents=documents
            )
            
            print(f"查询: {query}")
            print("所有结果:")
            for i, result in enumerate(response.results):
                print(f"  {i+1}. 分数: {result.score:.4f}, 文档: {documents[result.index]}")
                
        except Exception as e:
            print(f"错误: {e}")


def health_check_example():
    """健康检查示例"""
    print("\n=== 健康检查示例 ===")
    
    client = RerankClient()
    
    try:
        is_healthy = client.health_check()
        if is_healthy:
            print("✅ 重排序服务健康")
        else:
            print("❌ 重排序服务不健康")
    except Exception as e:
        print(f"检查健康状态时出错: {e}")
    finally:
        client.close()


def main():
    """运行所有示例"""
    print("Chatig 重排序 SDK 示例")
    print("=" * 50)
    
    # 运行示例
    basic_rerank_example()
    vllm_rerank_example()
    llamabox_rerank_example()
    batch_rerank_example()
    context_manager_example()
    health_check_example()
    
    print("\n" + "=" * 50)
    print("示例完成！")


if __name__ == "__main__":
    main() 