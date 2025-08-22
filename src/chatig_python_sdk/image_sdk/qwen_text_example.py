#!/usr/bin/env python3
"""
Qwen3-0.6B文字生成SDK使用示例
适用于通过vLLM部署的Qwen模型
"""

from text_generation import QwenTextGenerationAPI, create_qwen_client, quick_text_generation

def example_basic_usage():
    """基本使用示例"""
    print("🚀 Qwen3-0.6B文字生成示例")
    print("=" * 50)
    
    # 方法1：使用便捷函数
    print("\n📝 方法1：快速文字生成")
    try:
        response = quick_text_generation(
            prompt="请介绍一下人工智能的发展历史",
            system_prompt="你是一个专业的技术专家"
        )
        print(f"AI回复: {response}")
    except Exception as e:
        print(f"❌ 快速生成失败: {e}")

def example_client_usage():
    """使用客户端实例的示例"""
    print("\n📝 方法2：使用客户端实例")
    
    # 创建客户端
    client = create_qwen_client(api_base="http://localhost:8001")
    
    try:
        # 测试连接
        if client.test_connection():
            print("✅ API连接成功！")
            
            # 生成文字
            response = client.generate_text(
                prompt="请解释一下什么是机器学习",
                system_prompt="你是一个机器学习专家",
                temperature=0.8,
                max_tokens=500
            )
            print(f"AI回复: {response}")
            
            # 聊天完成接口
            messages = [
                {"role": "user", "content": "什么是深度学习？"},
                {"role": "assistant", "content": "深度学习是机器学习的一个分支..."},
                {"role": "user", "content": "它与传统机器学习有什么区别？"}
            ]
            
            chat_response = client.chat_completion(messages, temperature=0.7)
            print(f"\n聊天完成回复: {chat_response.choices[0]['message']['content']}")
            
        else:
            print("❌ API连接失败，请检查服务是否运行")
            
    except Exception as e:
        print(f"❌ 客户端使用失败: {e}")
    finally:
        client.close()

def example_conversation():
    """对话示例"""
    print("\n📝 方法3：多轮对话示例")
    
    client = create_qwen_client(api_base="http://localhost:8001")
    
    try:
        # 第一轮对话
        response1 = client.generate_text(
            prompt="请介绍一下Python编程语言",
            system_prompt="你是一个编程导师"
        )
        print(f"第一轮: {response1}")
        
        # 第二轮对话（基于第一轮内容）
        response2 = client.generate_text(
            prompt="Python适合初学者学习吗？为什么？",
            system_prompt="你是一个编程导师，请基于之前的回答继续"
        )
        print(f"第二轮: {response2}")
        
    except Exception as e:
        print(f"❌ 对话失败: {e}")
    finally:
        client.close()

def example_error_handling():
    """错误处理示例"""
    print("\n📝 方法4：错误处理示例")
    
    # 测试错误的API地址
    client = create_qwen_client(api_base="http://localhost:9999")
    
    try:
        response = client.generate_text("测试消息")
        print(f"回复: {response}")
    except Exception as e:
        print(f"预期的错误: {e}")
    finally:
        client.close()

def main():
    """主函数"""
    print("🎯 Qwen3-0.6B文字生成SDK完整示例")
    print("=" * 60)
    
    # 运行所有示例
    examples = [
        ("基本使用", example_basic_usage),
        ("客户端使用", example_client_usage),
        ("多轮对话", example_conversation),
        ("错误处理", example_error_handling)
    ]
    
    for name, func in examples:
        try:
            func()
            print(f"\n✅ {name}示例完成")
        except Exception as e:
            print(f"\n❌ {name}示例失败: {e}")
        
        print("-" * 40)
    
    print("\n🎉 所有示例运行完成！")
    print("\n💡 使用提示:")
    print("1. 确保vLLM服务在localhost:8001运行")
    print("2. 确保Qwen3-0.6B模型已加载")
    print("3. 可以根据需要调整temperature和max_tokens参数")
    print("4. 使用client.close()确保资源正确释放")

if __name__ == "__main__":
    main()
