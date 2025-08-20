#!/usr/bin/env python3
"""
测试Qwen3-0.6B模型连接的简单脚本
"""

import requests
import json

def test_qwen_connection():
    """测试Qwen模型连接"""
    print("🔍 测试Qwen3-0.6B模型连接")
    print("=" * 40)
    
    # 测试1：检查模型列表
    print("\n📋 测试1：获取模型列表")
    try:
        response = requests.get("http://localhost:8001/v1/models", timeout=10)
        if response.status_code == 200:
            models = response.json()
            print("✅ 模型列表获取成功")
            print(f"可用模型: {json.dumps(models, indent=2, ensure_ascii=False)}")
        else:
            print(f"❌ 获取模型列表失败，状态码: {response.status_code}")
            return False
    except Exception as e:
        print(f"❌ 连接模型列表接口失败: {e}")
        return False
    
    # 测试2：测试聊天接口
    print("\n💬 测试2：测试聊天接口")
    try:
        payload = {
            "model": "Qwen3-0.6B",
            "messages": [
                {"role": "user", "content": "你好，请简单介绍一下你自己"}
            ],
            "temperature": 0.7,
            "max_tokens": 100
        }
        
        response = requests.post(
            "http://localhost:8001/v1/chat/completions",
            headers={"Content-Type": "application/json"},
            json=payload,
            timeout=30
        )
        
        if response.status_code == 200:
            result = response.json()
            print("✅ 聊天接口测试成功")
            if 'choices' in result and len(result['choices']) > 0:
                content = result['choices'][0]['message']['content']
                print(f"AI回复: {content}")
            else:
                print("⚠️ 响应中没有找到AI回复内容")
        else:
            print(f"❌ 聊天接口测试失败，状态码: {response.status_code}")
            print(f"错误信息: {response.text}")
            return False
            
    except Exception as e:
        print(f"❌ 聊天接口测试失败: {e}")
        return False
    
    # 测试3：测试我们的SDK
    print("\n🔧 测试3：测试我们的SDK")
    try:
        from text_generation import quick_text_generation
        
        response = quick_text_generation(
            prompt="请用一句话总结今天的天气",
            system_prompt="你是一个天气预报员"
        )
        print("✅ SDK测试成功")
        print(f"AI回复: {response}")
        
    except Exception as e:
        print(f"❌ SDK测试失败: {e}")
        return False
    
    print("\n🎉 所有测试通过！Qwen模型连接正常")
    return True

def main():
    """主函数"""
    print("🚀 Qwen3-0.6B连接测试")
    print("=" * 50)
    
    if test_qwen_connection():
        print("\n✅ 连接测试成功！可以开始使用SDK了")
    else:
        print("\n❌ 连接测试失败！请检查以下问题：")
        print("1. vLLM服务是否在localhost:8001运行")
        print("2. Qwen3-0.6B模型是否已加载")
        print("3. 网络连接是否正常")
        print("4. 防火墙设置是否正确")

if __name__ == "__main__":
    main()
