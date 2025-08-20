#!/usr/bin/env python3
"""
在线图片生成SDK使用示例
支持OpenAI DALL-E、Stability AI等模型
"""

import os
from online_image_generation import (
    OnlineImageGenerationAPI, 
    ImageModelType, 
    ImageSize,
    create_openai_client,
    create_stability_client,
    quick_image_generation
)

def example_openai_dalle3():
    """OpenAI DALL-E 3示例"""
    print("🎨 OpenAI DALL-E 3 图片生成示例")
    print("=" * 50)
    
    # 需要设置环境变量或直接传入API密钥
    api_key = os.getenv("OPENAI_API_KEY")
    if not api_key:
        print("❌ 请设置OPENAI_API_KEY环境变量")
        print("export OPENAI_API_KEY='your_api_key_here'")
        return
    
    try:
        # 创建客户端
        client = create_openai_client(api_key)
        
        # 测试连接
        if client.test_connection():
            print("✅ OpenAI API连接成功！")
            
            # 生成图片
            response = client.generate_image(
                prompt="一只可爱的小猫坐在花园里，阳光明媚，风格温馨",
                size=ImageSize.LARGE,
                quality="hd",
                save_local=True,
                save_dir="./generated_images"
            )
            
            print(f"✅ 图片生成成功！")
            print(f"模型: {response.model}")
            print(f"提供商: {response.provider}")
            print(f"生成时间: {response.created}")
            
            for i, image_data in enumerate(response.data):
                print(f"\n图片 {i+1}:")
                if image_data.url:
                    print(f"  URL: {image_data.url}")
                if image_data.local_path:
                    print(f"  本地路径: {image_data.local_path}")
                if image_data.revised_prompt:
                    print(f"  修订提示词: {image_data.revised_prompt}")
        else:
            print("❌ OpenAI API连接失败")
            
    except Exception as e:
        print(f"❌ 图片生成失败: {e}")
    finally:
        client.close()

def example_stability_ai():
    """Stability AI示例"""
    print("\n🎨 Stability AI 图片生成示例")
    print("=" * 50)
    
    # 需要设置环境变量或直接传入API密钥
    api_key = os.getenv("STABILITY_API_KEY")
    if not api_key:
        print("❌ 请设置STABILITY_API_KEY环境变量")
        print("export STABILITY_API_KEY='your_api_key_here'")
        return
    
    try:
        # 创建客户端
        client = create_stability_client(api_key)
        
        # 测试连接
        if client.test_connection():
            print("✅ Stability AI API连接成功！")
            
            # 生成图片
            response = client.generate_image(
                prompt="A beautiful landscape painting, mountains and lake, artistic style",
                model=ImageModelType.STABILITY_SDXL,
                size=ImageSize.LARGE,
                save_local=True,
                save_dir="./generated_images"
            )
            
            print(f"✅ 图片生成成功！")
            print(f"模型: {response.model}")
            print(f"提供商: {response.provider}")
            
            for i, image_data in enumerate(response.data):
                print(f"\n图片 {i+1}:")
                if image_data.local_path:
                    print(f"  本地路径: {image_data.local_path}")
                if image_data.b64_json:
                    print(f"  包含Base64数据: 是")
        else:
            print("❌ Stability AI API连接失败")
            
    except Exception as e:
        print(f"❌ 图片生成失败: {e}")
    finally:
        client.close()

def example_quick_generation():
    """快速图片生成示例"""
    print("\n⚡ 快速图片生成示例")
    print("=" * 50)
    
    api_key = os.getenv("OPENAI_API_KEY")
    if not api_key:
        print("❌ 请设置OPENAI_API_KEY环境变量")
        return
    
    try:
        # 使用便捷函数快速生成
        response = quick_image_generation(
            prompt="一个未来科技感的机器人，霓虹灯效果",
            api_key=api_key,
            model=ImageModelType.OPENAI_DALLE3,
            save_local=True
        )
        
        print(f"✅ 快速生成成功！")
        print(f"模型: {response.model}")
        
        for i, image_data in enumerate(response.data):
            print(f"\n图片 {i+1}:")
            if image_data.local_path:
                print(f"  本地路径: {image_data.local_path}")
                
    except Exception as e:
        print(f"❌ 快速生成失败: {e}")

def example_custom_client():
    """自定义客户端示例"""
    print("\n🔧 自定义客户端示例")
    print("=" * 50)
    
    api_key = os.getenv("OPENAI_API_KEY")
    if not api_key:
        print("❌ 请设置OPENAI_API_KEY环境变量")
        return
    
    try:
        # 创建自定义客户端
        client = OnlineImageGenerationAPI(
            api_key=api_key,
            api_base="https://api.openai.com/v1",
            default_model=ImageModelType.OPENAI_DALLE2,
            timeout=90
        )
        
        # 生成多张图片
        response = client.generate_image(
            prompt="一只彩色蝴蝶在花丛中飞舞",
            model=ImageModelType.OPENAI_DALLE2,
            size=ImageSize.MEDIUM,
            n=2,
            save_local=True,
            save_dir="./custom_images"
        )
        
        print(f"✅ 自定义客户端生成成功！")
        print(f"生成了 {len(response.data)} 张图片")
        
        for i, image_data in enumerate(response.data):
            print(f"\n图片 {i+1}:")
            if image_data.local_path:
                print(f"  本地路径: {image_data.local_path}")
                
    except Exception as e:
        print(f"❌ 自定义客户端失败: {e}")
    finally:
        client.close()

def example_error_handling():
    """错误处理示例"""
    print("\n⚠️ 错误处理示例")
    print("=" * 50)
    
    # 测试无效的API密钥
    try:
        client = create_openai_client("invalid_key")
        response = client.generate_image("测试图片")
        print(f"回复: {response}")
    except Exception as e:
        print(f"预期的错误: {e}")
    finally:
        client.close()
    
    # 测试无效的模型类型
    try:
        response = quick_image_generation(
            prompt="测试",
            api_key="test_key",
            model="invalid_model"
        )
        print(f"回复: {response}")
    except Exception as e:
        print(f"预期的错误: {e}")

def main():
    """主函数"""
    print("🚀 在线图片生成SDK完整示例")
    print("=" * 60)
    
    # 检查环境变量
    print("🔍 检查环境变量:")
    openai_key = os.getenv("OPENAI_API_KEY")
    stability_key = os.getenv("STABILITY_API_KEY")
    
    print(f"  OPENAI_API_KEY: {'✅ 已设置' if openai_key else '❌ 未设置'}")
    print(f"  STABILITY_API_KEY: {'✅ 已设置' if stability_key else '❌ 未设置'}")
    
    if not openai_key and not stability_key:
        print("\n⚠️ 请至少设置一个API密钥:")
        print("export OPENAI_API_KEY='your_openai_key'")
        print("export STABILITY_API_KEY='your_stability_key'")
        return
    
    # 运行示例
    examples = []
    
    if openai_key:
        examples.extend([
            ("OpenAI DALL-E 3", example_openai_dalle3),
            ("快速生成", example_quick_generation),
            ("自定义客户端", example_custom_client)
        ])
    
    if stability_key:
        examples.append(("Stability AI", example_stability_ai))
    
    examples.append(("错误处理", example_error_handling))
    
    # 运行所有示例
    for name, func in examples:
        try:
            func()
            print(f"\n✅ {name}示例完成")
        except Exception as e:
            print(f"\n❌ {name}示例失败: {e}")
        
        print("-" * 40)
    
    print("\n🎉 所有示例运行完成！")
    print("\n💡 使用提示:")
    print("1. 设置正确的API密钥环境变量")
    print("2. 选择合适的模型和参数")
    print("3. 使用save_local=True保存图片到本地")
    print("4. 注意API使用限制和费用")

if __name__ == "__main__":
    main()
