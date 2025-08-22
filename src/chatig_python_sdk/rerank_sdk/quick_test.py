#!/usr/bin/env python3
"""
快速测试脚本

用于快速验证修复后的功能。
"""

def test_imports():
    """测试导入"""
    print("测试导入...")
    try:
        from models import RerankEngineType, StdRerankRequest, RerankValidationError
        from config import RerankConfig
        from rerank_client import RerankClient
        print("✅ 导入成功")
        return True
    except Exception as e:
        print(f"❌ 导入失败: {e}")
        return False


def test_health_check():
    """测试健康检查"""
    print("\n测试健康检查...")
    try:
        from rerank_client import RerankClient
        client = RerankClient()
        is_healthy = client.health_check()
        print(f"健康检查结果: {is_healthy}")
        client.close()
        return True
    except Exception as e:
        print(f"❌ 健康检查失败: {e}")
        return False


def test_validation():
    """测试验证"""
    print("\n测试验证...")
    try:
        from models import StdRerankRequest, RerankValidationError
        
        # 测试空模型名称
        try:
            request = StdRerankRequest(
                model="",
                query="test",
                documents=["test"]
            )
            request.validate()
            print("❌ 应该抛出异常但没有")
            return False
        except RerankValidationError as e:
            print(f"✅ 捕获验证错误: {e}")
        
        return True
    except Exception as e:
        print(f"❌ 验证测试失败: {e}")
        return False


def test_config():
    """测试配置"""
    print("\n测试配置...")
    try:
        from config import RerankConfig
        
        config = RerankConfig()
        health_url = config.get_health_url()
        print(f"健康检查URL: {health_url}")
        
        std_url = config.get_rerank_url("std")
        print(f"STD重排序URL: {std_url}")
        
        return True
    except Exception as e:
        print(f"❌ 配置测试失败: {e}")
        return False


def main():
    """主函数"""
    print("Chatig 重排序 SDK 快速测试")
    print("=" * 40)
    
    tests = [
        ("导入", test_imports),
        ("配置", test_config),
        ("验证", test_validation),
        ("健康检查", test_health_check),
    ]
    
    passed = 0
    total = len(tests)
    
    for test_name, test_func in tests:
        print(f"\n{'='*20} {test_name} {'='*20}")
        if test_func():
            print(f"✅ {test_name} 通过")
            passed += 1
        else:
            print(f"❌ {test_name} 失败")
    
    print(f"\n总计: {passed}/{total} 个测试通过")
    
    if passed == total:
        print("🎉 所有基础测试通过！")
    else:
        print("⚠️  部分测试失败")


if __name__ == "__main__":
    main() 