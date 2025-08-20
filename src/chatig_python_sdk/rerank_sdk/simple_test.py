#!/usr/bin/env python3
"""
简单的导入测试脚本

用于验证所有模块是否可以正常导入。
"""

def test_imports():
    """测试所有模块的导入"""
    print("测试模块导入...")
    
    try:
        # 测试基础模块导入
        print("✅ 导入 models...")
        from models import RerankEngineType, StdRerankRequest
        
        print("✅ 导入 exceptions...")
        from exceptions import RerankError, RerankValidationError
        
        print("✅ 导入 config...")
        from config import RerankConfig
        
        print("✅ 导入 rerank_client...")
        from rerank_client import RerankClient
        
        print("✅ 所有模块导入成功！")
        return True
        
    except ImportError as e:
        print(f"❌ 导入失败: {e}")
        return False
    except Exception as e:
        print(f"❌ 其他错误: {e}")
        return False


def test_basic_functionality():
    """测试基本功能"""
    print("\n测试基本功能...")
    
    try:
        # 重新导入需要的类
        from config import RerankConfig
        from rerank_client import RerankClient
        
        # 测试配置
        config = RerankConfig()
        print(f"✅ 配置创建成功: {config.base_url}")
        
        # 测试客户端创建
        client = RerankClient()
        print("✅ 客户端创建成功")
        
        # 测试健康检查
        is_healthy = client.health_check()
        print(f"✅ 健康检查完成: {is_healthy}")
        
        client.close()
        print("✅ 客户端关闭成功")
        
        return True
        
    except Exception as e:
        print(f"❌ 功能测试失败: {e}")
        return False


def main():
    """主函数"""
    print("Chatig 重排序 SDK 简单测试")
    print("=" * 40)
    
    # 测试导入
    import_success = test_imports()
    
    if import_success:
        # 测试基本功能
        func_success = test_basic_functionality()
        
        if func_success:
            print("\n🎉 所有测试通过！")
        else:
            print("\n⚠️  功能测试失败")
    else:
        print("\n❌ 导入测试失败")


if __name__ == "__main__":
    main() 