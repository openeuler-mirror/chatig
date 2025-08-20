
"""
使用你的Stability AI API密钥测试图片生成
"""

from image_sdk import create_stability_client, ImageModelType, ImageSize
#!/usr/bin/env python3
def test_with_your_key():
    """使用你的API密钥测试"""
    
    # 你的Stability AI API密钥
    YOUR_API_KEY = "sk-x7PcgaZEUNuf5GExcrenQybRPLScVyg16yEmlS4ICnfaG4Ae"
    
    print("🚀 使用你的Stability AI API密钥测试")
    print("=" * 50)
    print(f"🔑 API密钥: {YOUR_API_KEY[:10]}...")
    
    # 创建客户端
    client = create_stability_client(YOUR_API_KEY)
    
    try:
        # 测试连接
        print("\n🔍 测试API连接...")
        if client.test_connection():
            print("✅ Stability AI API连接成功！")
            
            # 生成测试图片
            print("\n🎨 开始生成测试图片...")
            response = client.generate_image(
                prompt="A harmonious and happy family trip to Beijing",
                model=ImageModelType.STABILITY_SDXL,
                size=ImageSize.MEDIUM,  # 使用中等尺寸
                save_local=True,
                save_dir="./your_test_images"
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
                    
            print(f"\n�� 测试完成！图片已保存到 ./your_test_images 目录")
            
        else:
            print("❌ Stability AI API连接失败")
            print("请检查网络连接和API密钥")
            
    except Exception as e:
        print(f"❌ 测试失败: {e}")
        print("\n💡 可能的原因:")
        print("1. 网络连接问题")
        print("2. API密钥权限问题")
        print("3. 账户余额不足")
        
    finally:
        client.close()

def generate_custom_image():
    """生成自定义图片"""
    
    YOUR_API_KEY = "sk-x7PcgaZEUNuf5GExcrenQybRPLScVyg16yEmlS4ICnfaG4Ae"
    
    print("\n🎨 生成自定义图片")
    print("=" * 30)
    
    # 你可以修改这里的提示词
    custom_prompt = "A cute cartoon cat sitting in a garden, sunny day, colorful flowers"
    
    client = create_stability_client(YOUR_API_KEY)
    
    try:
        response = client.generate_image(
            prompt=custom_prompt,
            model=ImageModelType.STABILITY_SDXL,
            size=ImageSize.LARGE,  # 大尺寸，高质量
            save_local=True,
            save_dir="./custom_images"
        )
        
        print(f"✅ 自定义图片生成成功！")
        print(f"提示词: {custom_prompt}")
        
        for i, image_data in enumerate(response.data):
            if image_data.local_path:
                print(f"图片 {i+1}: {image_data.local_path}")
                
    except Exception as e:
        print(f"❌ 自定义图片生成失败: {e}")
    finally:
        client.close()

def main():
    """主函数"""
    print("🎯 Stability AI 图片生成测试")
    print("=" * 60)
    
    # 运行测试
    test_with_your_key()
    
    # 询问是否生成自定义图片
    print("\n" + "="*50)
    choice = input("是否生成自定义图片？(y/n): ").lower().strip()
    
    if choice in ['y', 'yes', '是']:
        generate_custom_image()
    
    print("\n🎉 所有测试完成！")

if __name__ == "__main__":
    main()