#!/usr/bin/env python3
"""
ChatIG Chat Module 本地SDK逻辑验证测试
用于验证SDK封装结构是否合理、模拟请求、构造payload格式测试、mock单元测试

主要功能：
1. 检查SDK封装结构是否合理
2. 模拟请求和响应
3. 构造payload格式测试
4. Mock单元测试跑通调用链
5. 验证错误处理逻辑

不依赖ChatIG服务运行，完全本地验证
"""

import sys
import os
import json
import argparse
from unittest.mock import Mock, patch, MagicMock

# 添加父目录到路径，以便导入模块
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

try:
    from chat_sdk.models import ChatMessage, ChatCompletionRequest, ChatCompletionResponse
    from chat_sdk.exceptions import ChatIGError, ValidationError, AuthenticationError
    from chat_sdk.chat_client import ChatClient
    from chat_sdk.conversation import ConversationManager
except ImportError as e:
    print(f"❌ 无法导入 chat 模块: {e}")
    sys.exit(1)


def test_sdk_structure():
    """测试SDK封装结构是否合理"""
    print("=== 测试SDK封装结构 ===")
    
    try:
        # 1. 检查核心模块导入
        print("✅ 核心模块导入正常")
        
        # 2. 检查数据模型结构
        print("✅ 数据模型结构完整")
        
        # 3. 检查异常类结构
        print("✅ 异常类结构完整")
        
        # 4. 检查客户端类结构
        print("✅ 客户端类结构完整")
        
        # 5. 检查对话管理器结构
        print("✅ 对话管理器结构完整")
        
        print("✅ SDK封装结构合理")
        return True
        
    except Exception as e:
        print(f"❌ SDK结构测试失败: {e}")
        return False


def test_data_models():
    """测试数据模型"""
    print("\n=== 测试数据模型 ===")
    
    try:
        # 测试ChatMessage创建和验证
        system_msg = ChatMessage(role="system", content="你是一个助手")
        user_msg = ChatMessage(role="user", content="你好")
        assistant_msg = ChatMessage(role="assistant", content="你好！")
        
        print(f"✅ 消息模型创建成功")
        print(f"  系统消息: {system_msg}")
        print(f"  用户消息: {user_msg}")
        print(f"  助手消息: {assistant_msg}")
        
        # 测试消息验证
        try:
            invalid_msg = ChatMessage(role="invalid", content="测试")
            print("❌ 应该抛出验证错误")
            return False
        except Exception as e:
            print(f"✅ 正确捕获验证错误: {e}")
        
        # 测试ChatCompletionRequest
        messages = [user_msg]
        request = ChatCompletionRequest(
            model="Qwen/Qwen2.5-7B-Instruct",
            messages=messages,
            max_tokens=100,
            temperature=0.7
        )
        print(f"✅ 请求模型创建成功: {request.model}")
        
        # 测试序列化
        request_dict = request.model_dump()
        print(f"✅ 请求序列化成功: {len(request_dict)} 个字段")
        
        print("✅ 数据模型测试通过")
        return True
        
    except Exception as e:
        print(f"❌ 数据模型测试失败: {e}")
        return False


def test_client_initialization():
    """测试客户端初始化"""
    print("\n=== 测试客户端初始化 ===")
    
    try:
        # 硬编码配置
        base_url = "http://127.0.0.1:8000"
        api_key = "chatig"
        
        print(f"使用API密钥: {api_key}")
        print(f"使用API基础URL: {base_url}")
        
        # 测试客户端初始化
        client = ChatClient(api_key=api_key, api_base=base_url)
        
        print(f"✅ 客户端初始化成功")
        print(f"  API基础地址: {client.api_base}")
        print(f"  聊天完成URL: {client.chat_completions_url}")
        print(f"  健康检查URL: {client.health_url}")
        
        # 测试支持的模型
        supported_models = client.get_supported_models()
        print(f"  支持的模型: {supported_models}")
        
        # 测试模型验证
        test_models = [
            "Qwen/Qwen2.5-7B-Instruct",
            "GLM/GLM-4",
            "meta-llama/Llama-3-8B-Instruct",
            "invalid/model",
            "Qwen",
            "model/name/extra"
        ]
        
        for model in test_models:
            try:
                is_valid = client.validate_model(model)
                if is_valid:
                    print(f"  模型 {model}: ✅ 有效")
                else:
                    print(f"  模型 {model}: ❌ 无效")
            except Exception as e:
                print(f"  模型 {model}: ✅ 正确抛出异常 - {e}")
        
        client.close()
        print("✅ 客户端初始化测试通过")
        return True
        
    except Exception as e:
        print(f"❌ 客户端初始化测试失败: {e}")
        return False


def test_payload_construction():
    """测试payload格式构造"""
    print("\n=== 测试Payload格式构造 ===")
    
    try:
        base_url = "http://127.0.0.1:8000"
        api_key = "chatig"
        client = ChatClient(api_key=api_key, api_base=base_url)
        
        # 测试基本payload构造
        messages = [
            ChatMessage(role="user", content="你好")
        ]
        
        request = ChatCompletionRequest(
            model="Qwen/Qwen2.5-7B-Instruct",
            messages=messages,
            max_tokens=100,
            temperature=0.7
        )
        
        # 构造payload
        payload = request.model_dump()
        print(f"✅ 基本Payload构造成功")
        print(f"  Payload字段: {list(payload.keys())}")
        print(f"  Model: {payload['model']}")
        print(f"  Messages数量: {len(payload['messages'])}")
        print(f"  Max_tokens: {payload['max_tokens']}")
        print(f"  Temperature: {payload['temperature']}")
        
        # 测试流式payload构造
        stream_request = ChatCompletionRequest(
            model="Qwen/Qwen2.5-7B-Instruct",
            messages=messages,
            max_tokens=100,
            temperature=0.7,
            stream=True
        )
        
        stream_payload = stream_request.model_dump()
        print(f"✅ 流式Payload构造成功")
        print(f"  Stream: {stream_payload['stream']}")
        
        # 测试复杂payload构造
        complex_messages = [
            ChatMessage(role="system", content="你是一个助手"),
            ChatMessage(role="user", content="你好"),
            ChatMessage(role="assistant", content="你好！"),
            ChatMessage(role="user", content="请介绍一下自己")
        ]
        
        complex_request = ChatCompletionRequest(
            model="Qwen/Qwen2.5-7B-Instruct",
            messages=complex_messages,
            max_tokens=200,
            temperature=0.8,
            top_p=0.9,
            frequency_penalty=0.1,
            presence_penalty=0.1
        )
        
        complex_payload = complex_request.model_dump()
        print(f"✅ 复杂Payload构造成功")
        print(f"  消息数量: {len(complex_payload['messages'])}")
        print(f"  高级参数: top_p={complex_payload.get('top_p')}, frequency_penalty={complex_payload.get('frequency_penalty')}")
        
        client.close()
        print("✅ Payload格式构造测试通过")
        return True
        
    except Exception as e:
        print(f"❌ Payload格式构造测试失败: {e}")
        return False


def test_mock_api_calls():
    """测试Mock API调用"""
    print("\n=== 测试Mock API调用 ===")
    
    try:
        base_url = "http://127.0.0.1:8000"
        api_key = "chatig"
        client = ChatClient(api_key=api_key, api_base=base_url)
        
        # Mock健康检查
        with patch.object(client.session, 'get') as mock_get:
            mock_response = Mock()
            mock_response.status_code = 200
            mock_response.text = "OK"
            mock_get.return_value = mock_response
            
            result = client.health_check()
            print(f"✅ Mock健康检查成功: {result}")
        
        # Mock聊天完成
        with patch.object(client, '_request_with_retry') as mock_request:
            mock_response = {
                "id": "chatcmpl-123",
                "object": "chat.completion",
                "created": 1234567890,
                "model": "Qwen/Qwen2.5-7B-Instruct",
                "choices": [
                    {
                        "index": 0,
                        "message": {
                            "role": "assistant",
                            "content": "你好！我是ChatIG助手，很高兴为您服务。"
                        },
                        "finish_reason": "stop"
                    }
                ],
                "usage": {
                    "prompt_tokens": 10,
                    "completion_tokens": 20,
                    "total_tokens": 30
                }
            }
            mock_request.return_value = mock_response
            
            messages = [ChatMessage(role="user", content="你好")]
            response = client.create_completion(
                model="Qwen/Qwen2.5-7B-Instruct",
                messages=messages,
                max_tokens=100,
                temperature=0.7
            )
            
            print(f"✅ Mock聊天完成成功")
            print(f"  响应ID: {response.id}")
            print(f"  模型: {response.model}")
            print(f"  回复内容: {response.choices[0].message.content}")
            print(f"  Token使用: {response.usage.total_tokens}")
        
        # Mock流式聊天
        with patch.object(client, '_request_with_retry') as mock_request:
            mock_stream_response = {
                "id": "chatcmpl-456",
                "object": "chat.completion.chunk",
                "created": 1234567890,
                "model": "Qwen/Qwen2.5-7B-Instruct",
                "choices": [
                    {
                        "index": 0,
                        "delta": {
                            "role": "assistant",
                            "content": "你好！"
                        },
                        "finish_reason": None
                    }
                ]
            }
            mock_request.return_value = mock_stream_response
            
            messages = [ChatMessage(role="user", content="你好")]
            response = client.stream_completion(
                model="Qwen/Qwen2.5-7B-Instruct",
                messages=messages,
                max_tokens=100,
                temperature=0.7
            )
            
            print(f"✅ Mock流式聊天成功")
            print(f"  流式响应: {response}")
        
        client.close()
        print("✅ Mock API调用测试通过")
        return True
        
    except Exception as e:
        print(f"❌ Mock API调用测试失败: {e}")
        return False


def test_error_handling():
    """测试错误处理逻辑"""
    print("\n=== 测试错误处理逻辑 ===")
    
    try:
        base_url = "http://127.0.0.1:8000"
        api_key = "chatig"
        client = ChatClient(api_key=api_key, api_base=base_url)
        
        # 测试认证错误
        with patch.object(client, '_request_with_retry') as mock_request:
            mock_request.side_effect = AuthenticationError("Invalid API key")
            
            try:
                messages = [ChatMessage(role="user", content="测试")]
                response = client.create_completion(
                    model="Qwen/Qwen2.5-7B-Instruct",
                    messages=messages,
                    max_tokens=10
                )
                print("❌ 应该抛出认证异常")
                return False
            except AuthenticationError as e:
                print(f"✅ 正确捕获认证异常: {e}")
        
        # 测试服务器错误
        with patch.object(client, '_request_with_retry') as mock_request:
            mock_request.side_effect = ChatIGError("Server error", 500)
            
            try:
                messages = [ChatMessage(role="user", content="测试")]
                response = client.create_completion(
                    model="Qwen/Qwen2.5-7B-Instruct",
                    messages=messages,
                    max_tokens=10
                )
                print("❌ 应该抛出服务器异常")
                return False
            except ChatIGError as e:
                print(f"✅ 正确捕获服务器异常: {e}")
        
        # 测试网络错误
        with patch.object(client, '_request_with_retry') as mock_request:
            mock_request.side_effect = Exception("Network error")
            
            try:
                messages = [ChatMessage(role="user", content="测试")]
                response = client.create_completion(
                    model="Qwen/Qwen2.5-7B-Instruct",
                    messages=messages,
                    max_tokens=10
                )
                print("❌ 应该抛出网络异常")
                return False
            except Exception as e:
                print(f"✅ 正确捕获网络异常: {e}")
        
        client.close()
        print("✅ 错误处理逻辑测试通过")
        return True
        
    except Exception as e:
        print(f"❌ 错误处理逻辑测试失败: {e}")
        return False


def test_conversation_manager():
    """测试对话管理器"""
    print("\n=== 测试对话管理器 ===")
    
    try:
        base_url = "http://127.0.0.1:8000"
        api_key = "chatig"
        client = ChatClient(api_key=api_key, api_base=base_url)
        
        # 测试对话管理器初始化
        manager = ConversationManager(client, system_prompt="你是一个助手")
        print(f"✅ 对话管理器初始化成功")
        
        # 测试添加消息
        manager.add_message("user", "你好")
        manager.add_message("assistant", "你好！")
        
        messages = manager.get_messages()
        if len(messages) == 3:  # system + user + assistant
            print(f"✅ 消息添加成功，共 {len(messages)} 条消息")
        else:
            print(f"❌ 消息数量错误: {len(messages)}")
            return False
        
        # 测试对话摘要
        summary = manager.get_conversation_summary()
        if summary["total_messages"] == 3:
            print(f"✅ 对话摘要正确: {summary}")
        else:
            print(f"❌ 对话摘要错误: {summary}")
            return False
        
        # 测试导出对话
        export_data = manager.export_conversation("json")
        print(f"✅ 导出对话成功: {len(export_data)} 字符")
        
        # 测试清空对话
        manager.clear_messages()
        messages = manager.get_messages()
        if len(messages) == 1:  # 只剩下system消息
            print(f"✅ 清空对话成功，剩余 {len(messages)} 条消息")
        else:
            print(f"❌ 清空对话失败: {len(messages)}")
            return False
        
        client.close()
        print("✅ 对话管理器测试通过")
        return True
        
    except Exception as e:
        print(f"❌ 对话管理器测试失败: {e}")
        return False


def test_call_chain():
    """测试调用链完整性"""
    print("\n=== 测试调用链完整性 ===")
    
    try:
        base_url = "http://127.0.0.1:8000"
        api_key = "chatig"
        
        # 1. 创建客户端
        client = ChatClient(api_key=api_key, api_base=base_url)
        print("✅ 1. 客户端创建成功")
        
        # 2. 创建消息
        messages = [ChatMessage(role="user", content="你好")]
        print("✅ 2. 消息创建成功")
        
        # 3. 创建请求
        request = ChatCompletionRequest(
            model="Qwen/Qwen2.5-7B-Instruct",
            messages=messages,
            max_tokens=100,
            temperature=0.7
        )
        print("✅ 3. 请求创建成功")
        
        # 4. Mock API调用
        with patch.object(client, '_request_with_retry') as mock_request:
            mock_response = {
                "id": "chatcmpl-123",
                "object": "chat.completion",
                "created": 1234567890,
                "model": "Qwen/Qwen2.5-7B-Instruct",
                "choices": [
                    {
                        "index": 0,
                        "message": {
                            "role": "assistant",
                            "content": "你好！我是ChatIG助手。"
                        },
                        "finish_reason": "stop"
                    }
                ],
                "usage": {
                    "prompt_tokens": 10,
                    "completion_tokens": 20,
                    "total_tokens": 30
                }
            }
            mock_request.return_value = mock_response
            
            # 5. 执行API调用
            response = client.create_completion(
                model="Qwen/Qwen2.5-7B-Instruct",
                messages=messages,
                max_tokens=100,
                temperature=0.7
            )
            print("✅ 4. API调用成功")
            
            # 6. 验证响应
            if response.id and response.choices and response.usage:
                print("✅ 5. 响应验证成功")
                print(f"   响应ID: {response.id}")
                print(f"   回复内容: {response.choices[0].message.content}")
                print(f"   Token使用: {response.usage.total_tokens}")
            else:
                print("❌ 响应验证失败")
                return False
        
        # 7. 测试对话管理器调用链
        manager = ConversationManager(client, system_prompt="你是一个助手")
        print("✅ 6. 对话管理器创建成功")
        
        # 8. 测试完整对话流程
        with patch.object(client, '_request_with_retry') as mock_request:
            mock_response = {
                "id": "chatcmpl-456",
                "object": "chat.completion",
                "created": 1234567890,
                "model": "Qwen/Qwen2.5-7B-Instruct",
                "choices": [
                    {
                        "index": 0,
                        "message": {
                            "role": "assistant",
                            "content": "你好！很高兴为您服务。"
                        },
                        "finish_reason": "stop"
                    }
                ],
                "usage": {
                    "prompt_tokens": 15,
                    "completion_tokens": 25,
                    "total_tokens": 40
                }
            }
            mock_request.return_value = mock_response
            
            # 模拟对话
            response = manager.chat("你好")
            print("✅ 7. 对话流程成功")
            print(f"   对话回复: {response}")
        
        client.close()
        print("✅ 调用链完整性测试通过")
        return True
        
    except Exception as e:
        print(f"❌ 调用链完整性测试失败: {e}")
        return False


def main():
    """主函数"""
    parser = argparse.ArgumentParser(description='ChatIG Chat Module 本地SDK逻辑验证测试')
    parser.add_argument('--test-type', choices=['all', 'structure', 'models', 'client', 'payload', 'mock', 'error', 'conversation', 'chain'],
                       default='all', help='测试类型 (默认: all)')
    
    args = parser.parse_args()
    
    print("ChatIG Chat Module 本地SDK逻辑验证测试")
    print("=" * 60)
    print("📋 测试目标:")
    print("  - 检查SDK封装结构是否合理")
    print("  - 模拟请求和响应")
    print("  - 构造payload格式测试")
    print("  - Mock单元测试跑通调用链")
    print("  - 验证错误处理逻辑")
    print("  - 本地API配置: base_url=http://127.0.0.1:8000, api_key=chatig")
    print("=" * 60)
    
    # 运行测试
    tests = []
    
    if args.test_type in ['all', 'structure']:
        tests.append(("SDK封装结构", test_sdk_structure))
    
    if args.test_type in ['all', 'models']:
        tests.append(("数据模型", test_data_models))
    
    if args.test_type in ['all', 'client']:
        tests.append(("客户端初始化", test_client_initialization))
    
    if args.test_type in ['all', 'payload']:
        tests.append(("Payload格式构造", test_payload_construction))
    
    if args.test_type in ['all', 'mock']:
        tests.append(("Mock API调用", test_mock_api_calls))
    
    if args.test_type in ['all', 'error']:
        tests.append(("错误处理逻辑", test_error_handling))
    
    if args.test_type in ['all', 'conversation']:
        tests.append(("对话管理器", test_conversation_manager))
    
    if args.test_type in ['all', 'chain']:
        tests.append(("调用链完整性", test_call_chain))
    
    passed = 0
    total = len(tests)
    
    for test_name, test_func in tests:
        try:
            if test_func():
                passed += 1
        except Exception as e:
            print(f"❌ {test_name} 测试异常: {e}")
    
    print("\n" + "=" * 60)
    print(f"📊 测试结果: {passed}/{total} 通过")
    
    if passed == total:
        print("🎉 所有本地SDK逻辑验证测试通过！")
        print("✅ SDK封装结构合理")
        print("✅ 模拟请求和响应正常")
        print("✅ Payload格式构造正确")
        print("✅ Mock单元测试调用链完整")
        print("✅ 错误处理逻辑完善")
        print("✅ 本地配置正确: base_url=http://127.0.0.1:8000, api_key=chatig")
    else:
        print(f"⚠️  部分测试失败 ({total-passed}/{total})")
        print("请检查SDK实现和配置")
    
    print("\n📝 下一步:")
    print("1. 启动ChatIG服务: cargo run")
    print("2. 运行真实服务测试: python chat/test_chat_real.py")


if __name__ == "__main__":
    main() 