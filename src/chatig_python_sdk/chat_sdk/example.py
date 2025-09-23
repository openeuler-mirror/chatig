#!/usr/bin/env python3
"""
ChatIG Chat Module 使用示例
演示如何使用ChatIG Python SDK进行聊天
"""

import os
import sys

# 添加父目录到路径
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

# 设置新的API配置
base_url = "http://127.0.0.1:8001"
api_key = "chatig"

# 设置环境变量
os.environ['CHATIG_API_KEY'] = api_key
os.environ['CHATIG_API_BASE'] = base_url

print(f"📋 API配置:")
print(f"  Base URL: {base_url}")
print(f"  API Key: {api_key}")

from chat_sdk import ChatClient, ChatMessage


def basic_chat_example():
    """基本聊天示例"""
    print("=== 基本聊天示例 ===")
    
    # 创建客户端
    client = ChatClient(
        api_key=os.environ.get('CHATIG_API_KEY', 'sk-culinux'),
        api_base=os.environ.get('CHATIG_API_BASE', 'http://localhost:8081')
    )
    
    try:
        # 检查服务健康状态
        if not client.health_check():
            print("❌ 服务不可用，请确保ChatIG服务正在运行")
            return
        
        print("✅ 服务健康检查通过")
        
        # 创建消息
        messages = [
            ChatMessage(role="user", content="你好，请介绍一下自己")
        ]
        
        # 发送聊天请求
        print("💬 发送聊天请求...")
        response = client.create_completion(
            model="Qwen/Qwen2.5-7B-Instruct",
            messages=messages,
            max_tokens=100,
            temperature=0.7
        )
        
        # 显示结果
        print(f"🤖 助手回复: {response.choices[0].message.content}")
        print(f"📊 Token使用: {response.usage.total_tokens}")
        
    except Exception as e:
        print(f"❌ 聊天失败: {e}")
    finally:
        client.close()


def conversation_example():
    """多轮对话示例"""
    print("\n=== 多轮对话示例 ===")
    
    client = ChatClient(
        api_key=os.environ.get('CHATIG_API_KEY', 'sk-culinux'),
        api_base=os.environ.get('CHATIG_API_BASE', 'http://localhost:8081')
    )
    
    try:
        # 对话历史
        messages = []
        
        # 第一轮对话
        messages.append(ChatMessage(role="user", content="你好，我叫小明"))
        print(f"👤 用户: {messages[-1].content}")
        
        response1 = client.create_completion(
            model="Qwen/Qwen2.5-7B-Instruct",
            messages=messages,
            max_tokens=50,
            temperature=0.7
        )
        
        assistant_reply1 = response1.choices[0].message.content
        messages.append(ChatMessage(role="assistant", content=assistant_reply1))
        print(f"🤖 助手: {assistant_reply1}")
        
        # 第二轮对话
        messages.append(ChatMessage(role="user", content="我今年18岁，你呢？"))
        print(f"👤 用户: {messages[-1].content}")
        
        response2 = client.create_completion(
            model="Qwen/Qwen2.5-7B-Instruct",
            messages=messages,
            max_tokens=50,
            temperature=0.7
        )
        
        assistant_reply2 = response2.choices[0].message.content
        print(f"🤖 助手: {assistant_reply2}")
        
        print(f"📊 总Token使用: {response1.usage.total_tokens + response2.usage.total_tokens}")
        
    except Exception as e:
        print(f"❌ 对话失败: {e}")
    finally:
        client.close()


def model_validation_example():
    """模型验证示例"""
    print("\n=== 模型验证示例 ===")
    
    client = ChatClient(
        api_key=os.environ.get('CHATIG_API_KEY', 'sk-culinux'),
        api_base=os.environ.get('CHATIG_API_BASE', 'http://localhost:8081')
    )
    
    # 测试有效模型
    valid_models = [
        "Qwen/Qwen2.5-7B-Instruct",
        "GLM/GLM-4",
        "meta-llama/Llama-3-8B-Instruct"
    ]
    
    print("✅ 有效模型:")
    for model in valid_models:
        try:
            is_valid = client.validate_model(model)
            print(f"  {model}: {'有效' if is_valid else '无效'}")
        except Exception as e:
            print(f"  {model}: 验证失败 - {e}")
    
    # 测试无效模型
    invalid_models = [
        "invalid/model",
        "Qwen",
        "model/name/extra"
    ]
    
    print("\n❌ 无效模型:")
    for model in invalid_models:
        try:
            is_valid = client.validate_model(model)
            print(f"  {model}: {'应该无效' if is_valid else '正确识别为无效'}")
        except Exception as e:
            print(f"  {model}: 正确抛出异常 - {e}")
    
    client.close()


def main():
    """主函数"""
    print("🚀 ChatIG Chat Module 使用示例")
    print("=" * 40)
    
    # 显示配置
    print(f"📋 配置信息:")
    print(f"  API密钥: {os.environ.get('CHATIG_API_KEY', 'sk-culinux')[:10]}...")
    print(f"  API基础URL: {os.environ.get('CHATIG_API_BASE', 'http://localhost:8000')}")
    print()
    
    # 运行示例
    basic_chat_example()
    conversation_example()
    model_validation_example()
    
    print("\n" + "=" * 40)
    print("✅ 示例运行完成！")
    print("\n📝 使用提示:")
    print("1. 确保ChatIG服务正在运行: cargo run")
    print("2. 根据需要调整模型参数")
    print("3. 在生产环境中使用真实的API密钥")


if __name__ == "__main__":
    main() 