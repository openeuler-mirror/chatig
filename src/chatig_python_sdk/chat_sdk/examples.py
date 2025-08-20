"""
ChatIG Chat Module 使用示例
展示如何使用聊天模块的各种功能
"""

import asyncio
from typing import List

from .chat_client import ChatClient, AsyncChatClient
from .models import ChatMessage, StreamOptions
from .conversation import ConversationManager


def basic_chat_example():
    """基础聊天示例"""
    print("=== 基础聊天示例 ===")
    
    # 创建客户端
    client = ChatClient(
        api_key="your-api-key",
        api_base="http://localhost:8080"
    )
    
    try:
        # 简单聊天
        response = client.chat(
            message="你好，请介绍一下你自己",
            model="Qwen/Qwen2.5-7B-Instruct",
            temperature=0.7,
            max_tokens=1024
        )
        print(f"AI回复: {response}")
        
    finally:
        client.close()


def advanced_chat_example():
    """高级聊天示例"""
    print("\n=== 高级聊天示例 ===")
    
    client = ChatClient(
        api_key="your-api-key",
        api_base="http://localhost:8080"
    )
    
    try:
        # 创建消息列表
        messages = [
            ChatMessage.create_system_message("你是一个有用的AI助手，请用中文回答问题。"),
            ChatMessage.create_user_message("请解释什么是人工智能？")
        ]
        
        # 使用完整API
        response = client.create_completion(
            messages=messages,
            model="Qwen/Qwen2.5-7B-Instruct",
            temperature=0.8,
            max_tokens=1500,
            presence_penalty=0.1,
            frequency_penalty=0.1
        )
        
        print(f"AI回复: {response.choices[0].message.content}")
        print(f"Token使用: {response.usage}")
        
    finally:
        client.close()


def conversation_example():
    """会话管理示例"""
    print("\n=== 会话管理示例 ===")
    
    client = ChatClient(
        api_key="your-api-key",
        api_base="http://localhost:8080"
    )
    
    # 创建会话管理器
    conv_manager = ConversationManager(client)
    
    try:
        # 创建新会话
        conv_id = conv_manager.create_conversation("AI助手对话")
        print(f"创建会话: {conv_id}")
        
        # 在会话中聊天
        response1 = conv_manager.chat_with_conversation(
            message="你好，请介绍一下你自己",
            conversation_id=conv_id,
            system_prompt="你是一个友好的AI助手"
        )
        print(f"第一次对话: {response1}")
        
        # 继续对话
        response2 = conv_manager.chat_with_conversation(
            message="你能做什么？",
            conversation_id=conv_id
        )
        print(f"第二次对话: {response2}")
        
        # 查看会话历史
        messages = conv_manager.get_conversation_messages(conv_id)
        print(f"会话消息数量: {len(messages)}")
        
        # 列出所有会话
        conversations = conv_manager.list_conversations()
        print(f"会话列表: {conversations}")
        
    finally:
        client.close()


async def async_chat_example():
    """异步聊天示例"""
    print("\n=== 异步聊天示例 ===")
    
    async with AsyncChatClient(
        api_key="your-api-key",
        api_base="http://localhost:8080"
    ) as client:
        # 异步聊天
        response = await client.chat(
            message="请用Python写一个简单的计算器",
            model="Qwen/Qwen2.5-7B-Instruct",
            temperature=0.7,
            max_tokens=1024
        )
        print(f"异步AI回复: {response}")


async def stream_chat_example():
    """流式聊天示例"""
    print("\n=== 流式聊天示例 ===")
    
    client = ChatClient(
        api_key="your-api-key",
        api_base="http://localhost:8080"
    )
    
    try:
        # 流式聊天
        print("开始流式对话...")
        async for chunk in client.chat_stream(
            message="请写一个关于春天的短诗",
            model="Qwen/Qwen2.5-7B-Instruct"
        ):
            print(chunk, end="", flush=True)
        print("\n流式对话完成")
        
    finally:
        client.close()


def model_validation_example():
    """模型验证示例"""
    print("\n=== 模型验证示例 ===")
    
    client = ChatClient(
        api_key="your-api-key",
        api_base="http://localhost:8080"
    )
    
    try:
        # 检查支持的模型
        supported_models = client.get_supported_models()
        print(f"支持的模型系列: {supported_models}")
        
        # 验证模型
        test_models = [
            "Qwen/Qwen2.5-7B-Instruct",
            "GLM/GLM-4",
            "meta-llama/Llama-3-8B-Instruct",
            "invalid/model"
        ]
        
        for model in test_models:
            is_valid = client.validate_model(model)
            print(f"模型 {model}: {'有效' if is_valid else '无效'}")
        
        # 健康检查
        is_healthy = client.health_check()
        print(f"服务健康状态: {'正常' if is_healthy else '异常'}")
        
    finally:
        client.close()


def error_handling_example():
    """错误处理示例"""
    print("\n=== 错误处理示例 ===")
    
    client = ChatClient(
        api_key="invalid-key",
        api_base="http://localhost:8080"
    )
    
    try:
        # 尝试使用无效的API密钥
        response = client.chat("测试消息")
        print(f"响应: {response}")
        
    except Exception as e:
        print(f"捕获到错误: {type(e).__name__}: {e}")
    
    finally:
        client.close()


def complete_workflow_example():
    """完整工作流示例"""
    print("\n=== 完整工作流示例 ===")
    
    client = ChatClient(
        api_key="your-api-key",
        api_base="http://localhost:8080"
    )
    
    conv_manager = ConversationManager(client)
    
    try:
        # 1. 健康检查
        if not client.health_check():
            print("服务不可用，退出")
            return
        
        # 2. 创建会话
        conv_id = conv_manager.create_conversation("编程助手")
        print(f"创建会话: {conv_id}")
        
        # 3. 设置系统提示词
        system_prompt = """你是一个专业的Python编程助手。
请提供清晰、准确的代码示例和解释。
代码要包含适当的注释。"""
        
        # 4. 进行多轮对话
        questions = [
            "请解释Python中的装饰器是什么？",
            "能给我一个装饰器的实际例子吗？",
            "装饰器和继承有什么区别？"
        ]
        
        for i, question in enumerate(questions, 1):
            print(f"\n--- 第{i}轮对话 ---")
            print(f"用户: {question}")
            
            response = conv_manager.chat_with_conversation(
                message=question,
                conversation_id=conv_id,
                system_prompt=system_prompt if i == 1 else None,
                model="Qwen/Qwen2.5-7B-Instruct",
                temperature=0.7,
                max_tokens=1024
            )
            print(f"AI: {response}")
        
        # 5. 查看会话统计
        stats = conv_manager.get_conversation_stats()
        print(f"\n会话统计: {stats}")
        
        # 6. 导出会话数据
        conv_data = conv_manager.export_conversation(conv_id)
        print(f"会话数据已导出，包含 {len(conv_data.get('messages', []))} 条消息")
        
    finally:
        client.close()


def main():
    """主函数 - 运行所有示例"""
    print("ChatIG Chat Module 使用示例")
    print("=" * 50)
    
    # 同步示例
    basic_chat_example()
    advanced_chat_example()
    conversation_example()
    model_validation_example()
    error_handling_example()
    complete_workflow_example()
    
    # 异步示例
    asyncio.run(async_chat_example())
    asyncio.run(stream_chat_example())


if __name__ == "__main__":
    main() 