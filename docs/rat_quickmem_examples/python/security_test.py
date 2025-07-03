#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
RatQuickMem 安全防护测试脚本

专门测试 Python 绑定中的安全防护功能
"""

import sys
import time
from typing import List, Dict, Any

try:
    import rat_quickmem_py as rat_quickmem
except ImportError:
    print("❌ 请先编译并安装 rat_quickmem_py Python 模块")
    print("   运行: cd ../python && maturin develop --release")
    sys.exit(1)

def test_batch_count_protection():
    """测试批量数量防护"""
    print("\n🔒 测试批量数量防护:")
    
    try:
        # 创建超过限制的批量数据 (默认限制 10000)
        print("   创建 15000 项数据 (超过 10000 限制)...")
        large_batch = [f"item_{i}" for i in range(15000)]
        
        start_time = time.time()
        rat_quickmem.encode_batch(large_batch)
        print("❌ 错误: 应该触发批量数量限制")
        return False
        
    except ValueError as e:
        elapsed = time.time() - start_time
        print(f"✅ 成功拦截批量数量攻击 ({elapsed*1000:.2f}ms)")
        print(f"   错误信息: {e}")
        return True
    except Exception as e:
        print(f"❌ 意外错误: {e}")
        return False

def test_size_protection():
    """测试数据大小防护"""
    print("\n🔒 测试数据大小防护:")
    
    try:
        # 创建超大数据 (默认限制 100MB)
        size_mb = 200
        print(f"   创建 {size_mb}MB 数据 (超过 100MB 限制)...")
        huge_data = "x" * (size_mb * 1024 * 1024)
        
        start_time = time.time()
        rat_quickmem.encode(huge_data)
        print("❌ 错误: 应该触发数据大小限制")
        return False
        
    except ValueError as e:
        elapsed = time.time() - start_time
        print(f"✅ 成功拦截超大数据攻击 ({elapsed*1000:.2f}ms)")
        print(f"   错误信息: {e}")
        return True
    except Exception as e:
        print(f"❌ 意外错误: {e}")
        return False

def test_malicious_decode():
    """测试恶意解码数据防护"""
    print("\n🔒 测试恶意解码数据防护:")
    
    try:
        # 创建看似合法但恶意的编码数据
        # 这里模拟一个声称包含大量数据的恶意载荷
        malicious_data = b"\xff" * 1000  # 1KB 的恶意数据
        
        start_time = time.time()
        rat_quickmem.decode(malicious_data)
        print("❌ 错误: 应该触发解码安全检查")
        return False
        
    except (ValueError, RuntimeError) as e:
        elapsed = time.time() - start_time
        print(f"✅ 成功拦截恶意解码攻击 ({elapsed*1000:.2f}ms)")
        print(f"   错误信息: {e}")
        return True
    except Exception as e:
        print(f"❌ 意外错误: {e}")
        return False

def test_normal_operations():
    """测试正常操作不受影响"""
    print("\n✅ 测试正常操作:")
    
    success_count = 0
    total_tests = 3
    
    # 1. 正常单项编码解码
    try:
        normal_data = {
            "id": 12345,
            "name": "正常测试数据",
            "values": [1.1, 2.2, 3.3],
            "metadata": {"type": "test"}
        }
        
        start_time = time.time()
        encoded = rat_quickmem.encode(normal_data)
        decoded = rat_quickmem.decode(encoded)
        elapsed = time.time() - start_time
        
        if normal_data == decoded:
            print(f"   ✅ 单项操作成功 ({elapsed*1000:.2f}ms, {len(encoded)} 字节)")
            success_count += 1
        else:
            print("   ❌ 单项操作数据不匹配")
            
    except Exception as e:
        print(f"   ❌ 单项操作失败: {e}")
    
    # 2. 正常批量操作
    try:
        batch_data = [{"id": i, "value": f"item_{i}"} for i in range(100)]
        
        start_time = time.time()
        encoded_batch = rat_quickmem.encode_batch(batch_data)
        decoded_batch = rat_quickmem.decode_batch(encoded_batch)
        elapsed = time.time() - start_time
        
        if batch_data == decoded_batch:
            print(f"   ✅ 批量操作成功 ({elapsed*1000:.2f}ms, {len(batch_data)} 项)")
            success_count += 1
        else:
            print("   ❌ 批量操作数据不匹配")
            
    except Exception as e:
        print(f"   ❌ 批量操作失败: {e}")
    
    # 3. 边界数据测试
    try:
        # 接近但不超过限制的数据
        boundary_batch = [f"item_{i}" for i in range(9999)]  # 接近 10000 限制
        
        start_time = time.time()
        encoded = rat_quickmem.encode_batch(boundary_batch)
        decoded = rat_quickmem.decode_batch(encoded)
        elapsed = time.time() - start_time
        
        if boundary_batch == decoded:
            print(f"   ✅ 边界数据成功 ({elapsed*1000:.2f}ms, {len(boundary_batch)} 项)")
            success_count += 1
        else:
            print("   ❌ 边界数据不匹配")
            
    except Exception as e:
        print(f"   ❌ 边界数据失败: {e}")
    
    return success_count == total_tests

def test_performance_impact():
    """测试安全检查对性能的影响"""
    print("\n⚡ 测试性能影响:")
    
    test_data = {str(i): {"id": i, "data": f"test_item_{i}"} for i in range(1000)}
    iterations = 100
    
    try:
        # 多次编码解码测试
        start_time = time.time()
        for _ in range(iterations):
             encoded = rat_quickmem.encode(test_data)
             decoded = rat_quickmem.decode(encoded)
        total_time = time.time() - start_time
        
        avg_time = (total_time / iterations) * 1000
        print(f"   ✅ 平均操作时间: {avg_time:.3f}ms ({iterations} 次迭代)")
        print(f"   ✅ 总耗时: {total_time:.3f}s")
        print(f"   ✅ 吞吐量: {iterations/total_time:.1f} 操作/秒")
        
        return avg_time < 10.0  # 期望平均时间小于 10ms
        
    except Exception as e:
        print(f"   ❌ 性能测试失败: {e}")
        return False

def main():
    """主测试函数"""
    print("🔒 RatQuickMem Python 安全防护测试")
    print("=" * 50)
    
    test_results = []
    
    # 运行所有安全测试
    test_results.append(("批量数量防护", test_batch_count_protection()))
    test_results.append(("数据大小防护", test_size_protection()))
    test_results.append(("恶意解码防护", test_malicious_decode()))
    test_results.append(("正常操作测试", test_normal_operations()))
    test_results.append(("性能影响测试", test_performance_impact()))
    
    # 汇总结果
    print("\n📊 测试结果汇总:")
    print("=" * 30)
    
    passed = 0
    total = len(test_results)
    
    for test_name, result in test_results:
        status = "✅ 通过" if result else "❌ 失败"
        print(f"   {test_name}: {status}")
        if result:
            passed += 1
    
    print(f"\n🎯 总体结果: {passed}/{total} 测试通过")
    
    if passed == total:
        print("🎉 所有安全防护测试通过!")
        return 0
    else:
        print("⚠️  部分测试失败，请检查安全防护实现")
        return 1

if __name__ == "__main__":
    sys.exit(main())