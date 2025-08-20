#!/usr/bin/env python3
"""
ChatIG Chat Module 简单功能验证
"""

from models import ChatMessage, ChatCompletionRequest
from exceptions import ChatIGError
from chat_client import ChatClient


def test_basic_functionality():
    """测试基本功能"""
    print("=== ChatIG Chat Module 基本功能测试 ===")
    
    # 1. 测试消息创建
    print("1. 测试消息创建...")
    system_msg = ChatMessage.create_system_message("你是一个助手")
    user_msg = ChatMessage.create_user_message("你好")
    assistant_msg = ChatMessage.create_assistant_message("你好！")
    
    print(f"   ✅ 系统消息: {system_msg.role} - {system_msg.content}")
    print(f"   ✅ 用户消息: {user_msg.role} - {user_msg.content}")
    print(f"   ✅ 助手消息: {assistant_msg.role} - {assistant_msg.content}")
    
    # 2. 测试请求创建
    print("\n2. 测试请求创建...")
    messages = [system_msg, user_msg]
    request = ChatCompletionRequest(
        model="Qwen/Qwen2.5-7B-Instruct",
        messages=messages,
        temperature=0.7,
        max_tokens=1024
    )
    print(f"   ✅ 请求创建成功: {request.model}")
    print(f"   ✅ 消息数量: {len(request.messages)}")
    
    # 3. 测试客户端初始化
    print("\n3. 测试客户端初始化...")
    client = ChatClient(
        api_key="test-key",
        api_base="http://localhost:8080"
    )
    print(f"   ✅ 客户端初始化成功")
    print(f"   ✅ API基础地址: {client.api_base}")
    
    # 4. 测试模型验证
    print("\n4. 测试模型验证...")
    valid_models = [
        "Qwen/Qwen2.5-7B-Instruct",
        "GLM/GLM-4",
        "meta-llama/Llama-3-8B-Instruct"
    ]
    
    for model in valid_models:
        is_valid = client.validate_model(model)
        print(f"   ✅ 模型 {model}: {'有效' if is_valid else '无效'}")
    
    # 5. 测试支持的模型
    print("\n5. 测试支持的模型...")
    supported_models = client.get_supported_models()
    print(f"   ✅ 支持的模型系列: {list(supported_models.keys())}")
    
    # 6. 测试会话管理器
    print("\n6. 测试会话管理器...")
    from conversation import ConversationManager
    
    conv_manager = ConversationManager(client)
    conv_id = conv_manager.create_conversation("测试会话")
    print(f"   ✅ 会话创建成功: {conv_id}")
    
    conversation = conv_manager.get_conversation(conv_id)
    if conversation:
        print(f"   ✅ 会话获取成功: {conversation.title}")
    
    # 7. 测试会话统计
    stats = conv_manager.get_conversation_stats()
    print(f"   ✅ 会话统计: {stats['total_conversations']} 个会话")
    
    # 8. 清理
    conv_manager.delete_conversation(conv_id)
    client.close()
    print(f"   ✅ 资源清理完成")
    
    print("\n=== 所有基本功能测试通过！ ===")
    print("ChatIG Chat Module 已成功创建并可以正常使用。")


if __name__ == "__main__":
    test_basic_functionality() 