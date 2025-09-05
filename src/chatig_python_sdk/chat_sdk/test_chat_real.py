#!/usr/bin/env python3
"""
ChatIG Chat Module 真实服务测试
用于验证能否真正调用接口、返回响应、真正的AI回复结果

主要功能：
1. 验证能否真正调用接口
2. 验证返回响应格式
3. 验证真正的AI回复结果（接口返回的content）
4. 检查接口是否挂了
5. 检查模型是否出错
6. 检查权限是否配置对了

需要ChatIG服务运行，支持本地和远程测试
"""

import sys
import os
import time
import argparse
import asyncio

# 添加父目录到路径，以便导入模块
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

try:
    from chat_sdk.models import ChatMessage, ChatCompletionRequest
    from chat_sdk.exceptions import ChatIGError, AuthenticationError, ServerError
    from chat_sdk.chat_client import ChatClient
    from chat_sdk.conversation import ConversationManager
except ImportError as e:
    print(f"❌ 无法导入 chat 模块: {e}")
    sys.exit(1)


def test_service_connection(base_url, api_key):
    """测试服务连接状态"""
    print("=== 测试服务连接状态 ===")
    
    try:
        print(f"🔗 连接地址: {base_url}")
        print(f"🔑 API密钥: {api_key}")
        
        client = ChatClient(api_key=api_key, api_base=base_url)
        
        # 测试健康检查
        print("🔍 检查服务健康状态...")
        try:
            is_healthy = client.health_check()
            if is_healthy:
                print("✅ 服务健康检查通过")
                print("✅ 接口没有挂掉")
            else:
                print("❌ 服务健康检查失败")
                print("❌ 接口可能有问题")
                return False
        except Exception as e:
            print(f"❌ 健康检查异常: {e}")
            print("❌ 接口可能挂了或配置错误")
            return False
        
        client.close()
        return True
        
    except Exception as e:
        print(f"❌ 服务连接测试失败: {e}")
        return False


def test_api_permissions(base_url, api_key):
    """测试API权限配置"""
    print("\n=== 测试API权限配置 ===")
    
    try:
        client = ChatClient(api_key=api_key, api_base=base_url)
        
        # 测试无效API密钥
        print("🧪 测试无效API密钥...")
        invalid_client = ChatClient(api_key="invalid-key", api_base=base_url)
        
        try:
            messages = [ChatMessage(role="user", content="测试")]
            response = invalid_client.create_completion(
                model="Qwen3-0.6B",
                messages=messages,
                max_tokens=10
            )
            print("❌ 无效API密钥应该被拒绝")
            return False
        except AuthenticationError as e:
            print(f"✅ 正确拒绝无效API密钥: {e}")
        except Exception as e:
            print(f"⚠️  其他错误（可能是网络问题）: {e}")
        
        # 测试有效API密钥
        print("🧪 测试有效API密钥...")
        try:
            messages = [ChatMessage(role="user", content="你好")]
            response = client.create_completion(
                model="Qwen3-0.6B",
                messages=messages,
                max_tokens=50,
                temperature=0.7
            )
            print("✅ 有效API密钥认证成功")
            print(f"  响应ID: {response.id}")
            print(f"  模型: {response.model}")
        except AuthenticationError as e:
            print(f"❌ API密钥认证失败: {e}")
            print("❌ 权限配置可能有问题")
            return False
        except Exception as e:
            print(f"⚠️  其他错误: {e}")
        
        invalid_client.close()
        client.close()
        return True
        
    except Exception as e:
        print(f"❌ API权限测试失败: {e}")
        return False


def test_chat_completion(base_url, api_key):
    """测试聊天完成功能"""
    print("\n=== 测试聊天完成功能 ===")
    
    try:
        client = ChatClient(api_key=api_key, api_base=base_url)
        
        # 测试基本聊天
        messages = [
            ChatMessage(role="user", content="你好，请简单介绍一下自己")
        ]
        
        print("💬 发送聊天请求...")
        print(f"👤 用户消息: {messages[0].content}")
        
        response = client.create_completion(
            model="Qwen3-0.6B",
            messages=messages,
            max_tokens=100,
            temperature=0.7
        )
        
        print("✅ 聊天完成请求成功")
        print(f"📝 响应ID: {response.id}")
        print(f"🤖 模型: {response.model}")
        print(f"💭 AI回复内容: {response.choices[0].message.content}")
        print(f"📊 Token使用: {response.usage.total_tokens}")
        
        # 验证AI回复内容
        ai_content = response.choices[0].message.content
        if ai_content and len(ai_content.strip()) > 0:
            print("✅ AI回复内容有效")
            print("✅ 真正的AI回复结果正常")
        else:
            print("❌ AI回复内容为空")
            print("❌ 模型可能出错")
            return False
        
        client.close()
        return True
        
    except Exception as e:
        print(f"❌ 聊天完成测试失败: {e}")
        return False


def test_streaming_chat(base_url, api_key):
    """测试流式聊天功能"""
    print("\n=== 测试流式聊天功能 ===")
    
    try:
        client = ChatClient(api_key=api_key, api_base=base_url)
        
        messages = [
            ChatMessage(role="user", content="请用中文写一首关于春天的短诗")
        ]
        
        print("🌊 发送流式聊天请求...")
        print(f"👤 用户消息: {messages[0].content}")
        
        # 测试流式聊天
        print("🤖 AI流式回复: ", end="", flush=True)
        
        full_response = ""
        async def test_stream():
            nonlocal full_response
            async for chunk in client.stream_completion(
                model="Qwen/Qwen2.5-7B-Instruct",
                messages=messages,
                max_tokens=200,
                temperature=0.8
            ):
                if chunk.choices and chunk.choices[0].delta.content:
                    content = chunk.choices[0].delta.content
                    print(content, end="", flush=True)
                    full_response += content
        
        # 运行异步测试
        asyncio.run(test_stream())
        print()
        
        if full_response and len(full_response.strip()) > 0:
            print("✅ 流式聊天测试成功")
            print(f"📝 完整回复长度: {len(full_response)} 字符")
            print("✅ 流式AI回复结果正常")
        else:
            print("❌ 流式回复内容为空")
            print("❌ 流式模型可能出错")
            return False
        
        client.close()
        return True
        
    except Exception as e:
        print(f"❌ 流式聊天测试失败: {e}")
        return False


def test_model_validation(base_url, api_key):
    """测试模型验证"""
    print("\n=== 测试模型验证 ===")
    
    try:
        client = ChatClient(api_key=api_key, api_base=base_url)
        
        # 测试有效模型
        valid_models = [
            "Qwen/Qwen2.5-7B-Instruct",
            "GLM/GLM-4",
            "meta-llama/Llama-3-8B-Instruct"
        ]
        
        for model in valid_models:
            print(f"🧪 测试模型: {model}")
            try:
                messages = [ChatMessage(role="user", content="测试")]
                response = client.create_completion(
                    model=model,
                    messages=messages,
                    max_tokens=50,
                    temperature=0.7
                )
                print(f"✅ 模型 {model} 工作正常")
                print(f"  回复: {response.choices[0].message.content[:50]}...")
            except Exception as e:
                print(f"❌ 模型 {model} 出错: {e}")
                print("❌ 模型可能配置有问题")
                return False
        
        # 测试无效模型
        print("🧪 测试无效模型...")
        try:
            messages = [ChatMessage(role="user", content="测试")]
            response = client.create_completion(
                model="invalid/model",
                messages=messages,
                max_tokens=10
            )
            print("❌ 无效模型应该被拒绝")
            return False
        except Exception as e:
            print(f"✅ 正确拒绝无效模型: {e}")
        
        client.close()
        print("✅ 模型验证测试通过")
        return True
        
    except Exception as e:
        print(f"❌ 模型验证测试失败: {e}")
        return False


def test_conversation_flow(base_url, api_key):
    """测试对话流程"""
    print("\n=== 测试对话流程 ===")
    
    try:
        client = ChatClient(api_key=api_key, api_base=base_url)
        
        # 第一轮对话
        messages = [
            ChatMessage(role="user", content="你好，我叫小明")
        ]
        
        print("👤 第一轮对话:")
        print(f"   用户: {messages[0].content}")
        
        response1 = client.create_completion(
            model="Qwen/Qwen2.5-7B-Instruct",
            messages=messages,
            max_tokens=50,
            temperature=0.7
        )
        
        assistant_reply1 = response1.choices[0].message.content
        print(f"🤖 助手: {assistant_reply1}")
        
        # 第二轮对话
        messages.extend([
            ChatMessage(role="assistant", content=assistant_reply1),
            ChatMessage(role="user", content="我今年18岁，你呢？")
        ])
        
        print("\n👤 第二轮对话:")
        print(f"   用户: {messages[-1].content}")
        
        response2 = client.create_completion(
            model="Qwen/Qwen2.5-7B-Instruct",
            messages=messages,
            max_tokens=50,
            temperature=0.7
        )
        
        assistant_reply2 = response2.choices[0].message.content
        print(f"🤖 助手: {assistant_reply2}")
        
        # 验证对话连贯性
        if assistant_reply1 and assistant_reply2:
            print("✅ 对话流程测试成功")
            print(f"📊 对话轮数: 2")
            print(f"📊 总Token使用: {response1.usage.total_tokens + response2.usage.total_tokens}")
            print("✅ 真正的AI对话结果正常")
        else:
            print("❌ 对话回复内容异常")
            return False
        
        client.close()
        return True
        
    except Exception as e:
        print(f"❌ 对话流程测试失败: {e}")
        return False


def test_conversation_manager(base_url, api_key):
    """测试对话管理器"""
    print("\n=== 测试对话管理器 ===")
    
    try:
        client = ChatClient(api_key=api_key, api_base=base_url)
        
        # 测试对话管理器
        manager = ConversationManager(client, system_prompt="你是一个助手")
        
        # 测试对话
        print("💬 开始对话...")
        response = manager.chat("你好，请介绍一下自己")
        print(f"🤖 助手回复: {response}")
        
        # 测试多轮对话
        response2 = manager.chat("我今年18岁，你呢？")
        print(f"🤖 助手回复: {response2}")
        
        # 测试对话摘要
        summary = manager.get_conversation_summary()
        print(f"📊 对话摘要: {summary}")
        
        # 测试导出对话
        export_data = manager.export_conversation("json")
        print(f"📤 导出对话: {len(export_data)} 字符")
        
        if response and response2:
            print("✅ 对话管理器测试成功")
            print("✅ 真正的AI对话管理结果正常")
        else:
            print("❌ 对话管理器回复异常")
            return False
        
        client.close()
        return True
        
    except Exception as e:
        print(f"❌ 对话管理器测试失败: {e}")
        return False


def test_error_scenarios(base_url, api_key):
    """测试错误场景"""
    print("\n=== 测试错误场景 ===")
    
    try:
        client = ChatClient(api_key=api_key, api_base=base_url)
        
        # 测试空消息
        print("🧪 测试空消息...")
        try:
            response = client.create_completion(
                model="Qwen/Qwen2.5-7B-Instruct",
                messages=[],
                max_tokens=10
            )
            print("❌ 空消息应该被拒绝")
            return False
        except Exception as e:
            print(f"✅ 正确拒绝空消息: {e}")
        
        # 测试超长消息
        print("🧪 测试超长消息...")
        long_content = "测试" * 10000  # 超长内容
        messages = [ChatMessage(role="user", content=long_content)]
        
        try:
            response = client.create_completion(
                model="Qwen/Qwen2.5-7B-Instruct",
                messages=messages,
                max_tokens=10
            )
            print("✅ 超长消息处理正常")
        except Exception as e:
            print(f"⚠️  超长消息被拒绝: {e}")
        
        # 测试无效参数
        print("🧪 测试无效参数...")
        try:
            messages = [ChatMessage(role="user", content="测试")]
            response = client.create_completion(
                model="Qwen/Qwen2.5-7B-Instruct",
                messages=messages,
                max_tokens=-1,  # 无效参数
                temperature=2.0  # 无效参数
            )
            print("❌ 无效参数应该被拒绝")
            return False
        except Exception as e:
            print(f"✅ 正确拒绝无效参数: {e}")
        
        client.close()
        print("✅ 错误场景测试通过")
        return True
        
    except Exception as e:
        print(f"❌ 错误场景测试失败: {e}")
        return False


def main():
    """主函数"""
    parser = argparse.ArgumentParser(description='ChatIG Chat Module 真实服务测试')
    parser.add_argument('--base-url', default='http://127.0.0.1:8001', 
                       help='ChatIG服务地址 (默认: http://127.0.0.1:8000)')
    parser.add_argument('--api-key', default='chatig', 
                       help='API密钥 (默认: chatig)')
    parser.add_argument('--test-type', choices=['all', 'connection', 'permissions', 'chat', 'stream', 'model', 'conversation', 'manager', 'error'],
                       default='all', help='测试类型 (默认: all)')
    
    args = parser.parse_args()
    
    print("ChatIG Chat Module 真实服务测试")
    print("=" * 60)
    print("📋 测试目标:")
    print("  - 验证能否真正调用接口")
    print("  - 验证返回响应格式")
    print("  - 验证真正的AI回复结果（接口返回的content）")
    print("  - 检查接口是否挂了")
    print("  - 检查模型是否出错")
    print("  - 检查权限是否配置对了")
    print(f"  - 服务地址: {args.base_url}")
    print(f"  - API密钥: {args.api_key}")
    print("=" * 60)
    
    # 运行测试
    tests = []
    
    if args.test_type in ['all', 'connection']:
        tests.append(("服务连接状态", lambda: test_service_connection(args.base_url, args.api_key)))
    
    if args.test_type in ['all', 'permissions']:
        tests.append(("API权限配置", lambda: test_api_permissions(args.base_url, args.api_key)))
    
    if args.test_type in ['all', 'chat']:
        tests.append(("聊天完成功能", lambda: test_chat_completion(args.base_url, args.api_key)))
    
    if args.test_type in ['all', 'stream']:
        tests.append(("流式聊天功能", lambda: test_streaming_chat(args.base_url, args.api_key)))
    
    if args.test_type in ['all', 'model']:
        tests.append(("模型验证", lambda: test_model_validation(args.base_url, args.api_key)))
    
    if args.test_type in ['all', 'conversation']:
        tests.append(("对话流程", lambda: test_conversation_flow(args.base_url, args.api_key)))
    
    if args.test_type in ['all', 'manager']:
        tests.append(("对话管理器", lambda: test_conversation_manager(args.base_url, args.api_key)))
    
    if args.test_type in ['all', 'error']:
        tests.append(("错误场景", lambda: test_error_scenarios(args.base_url, args.api_key)))
    
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
        print("🎉 所有真实服务测试通过！")
        print("✅ 接口调用正常")
        print("✅ 返回响应格式正确")
        print("✅ 真正的AI回复结果正常")
        print("✅ 接口没有挂掉")
        print("✅ 模型工作正常")
        print("✅ 权限配置正确")
    else:
        print(f"⚠️  部分测试失败 ({total-passed}/{total})")
        print("请检查:")
        print("1. ChatIG服务是否正在运行")
        print("2. 服务地址和API密钥是否正确")
        print("3. 网络连接是否正常")
        print("4. 模型配置是否正确")
    
    print("\n📝 使用说明:")
    print("1. 本地测试: python chat/test_chat_real.py")
    print("2. 远程测试: python chat/test_chat_real.py --base-url http://remote-server:port")
    print("3. 指定测试: python chat/test_chat_real.py --test-type chat")


if __name__ == "__main__":
    main() 