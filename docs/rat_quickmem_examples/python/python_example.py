#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
RatQuickMem Python 绑定示例

展示如何使用高性能的 bincode 编码解码功能
"""

import time
from typing import List, Dict, Any

try:
    import rat_quickmem_py as rat_quickmem
except ImportError:
    print("❌ 请先编译并安装 rat_quickmem_py Python 模块")
    print("   运行: cd ../python && maturin develop --release")
    exit(1)

def basic_example():
    """基本编码解码示例"""
    print("\n📝 基本编码解码示例:")
    
    # 测试数据 - Python 原生数据类型
    test_data = {
        "id": 12345,
        "name": "Python测试数据",
        "values": [1.1, 2.2, 3.3, 4.4, 5.5],
        "metadata": {
            "type": "benchmark",
            "version": "1.0",
            "language": "python"
        },
        "active": True,
        "tags": ["test", "benchmark", "python"]
    }
    
    # 直接 bincode 编码
    start_time = time.time()
    encoded = rat_quickmem.encode(test_data)
    encode_time = time.time() - start_time
    
    print(f"✅ bincode 编码完成: {len(encoded)} 字节, 耗时: {encode_time*1000:.3f}ms")
    
    # 直接 bincode 解码
    start_time = time.time()
    decoded = rat_quickmem.decode(encoded)
    decode_time = time.time() - start_time
    
    print(f"✅ bincode 解码完成, 耗时: {decode_time*1000:.3f}ms")
    print(f"✅ 数据验证: {'通过' if test_data == decoded else '失败'}")

def security_example():
    """安全防护功能演示"""
    print("\n🔒 安全防护功能演示:")
    
    # 1. 测试批量数量限制
    print("\n1️⃣ 测试批量数量限制:")
    try:
        # 创建超过限制的批量数据 (默认限制 10000)
        large_batch = [f"item_{i}" for i in range(15000)]
        rat_quickmem.encode_batch(large_batch)
        print("❌ 应该触发批量数量限制错误")
    except ValueError as e:
        print(f"✅ 成功拦截批量数量攻击: {e}")
    
    # 2. 测试数据大小限制
    print("\n2️⃣ 测试数据大小限制:")
    try:
        # 创建超大数据 (默认限制 100MB)
        huge_data = "x" * (200 * 1024 * 1024)  # 200MB 字符串
        rat_quickmem.encode(huge_data)
        print("❌ 应该触发数据大小限制错误")
    except ValueError as e:
        print(f"✅ 成功拦截超大数据攻击: {e}")
    
    # 3. 演示正常使用
    print("\n3️⃣ 正常数据处理:")
    try:
        normal_data = {"message": "这是正常大小的数据", "count": 100}
        encoded = rat_quickmem.encode(normal_data)
        decoded = rat_quickmem.decode(encoded)
        print(f"✅ 正常数据处理成功: {len(encoded)} 字节")
        print(f"✅ 数据验证: {'通过' if normal_data == decoded else '失败'}")
    except Exception as e:
        print(f"❌ 正常数据处理失败: {e}")
    
    # 4. 批量操作安全演示
    print("\n4️⃣ 安全批量操作:")
    try:
        safe_batch = [{"id": i, "data": f"item_{i}"} for i in range(100)]
        encoded_batch = rat_quickmem.encode_batch(safe_batch)
        decoded_batch = rat_quickmem.decode_batch(encoded_batch)
        print(f"✅ 安全批量操作成功: {len(safe_batch)} 项")
        print(f"✅ 批量数据验证: {'通过' if safe_batch == decoded_batch else '失败'}")
    except Exception as e:
        print(f"❌ 批量操作失败: {e}")
    
    return encoded, decoded

def codec_class_example():
    """使用编码解码器类的示例"""
    print("\n🔧 编码解码器类示例:")
    
    # 创建编码解码器
    codec = rat_quickmem.QuickCodec()
    
    test_data = {
        "message": "Hello from Python!",
        "timestamp": time.time(),
        "data": list(range(100))
    }
    
    # 使用编码器 - 直接 bincode 编码
    encoded = codec.encode(test_data)
    print(f"📦 编码器 bincode 编码: {len(encoded)} 字节")
    
    # 使用解码器 - 直接 bincode 解码
    decoded = codec.decode(encoded)
    print(f"📦 解码器 bincode 解码: {'成功' if decoded == test_data else '失败'}")
    
    # 往返测试 - 直接 bincode 往返
    roundtrip_result = codec.roundtrip(test_data)
    print(f"🔄 bincode 往返测试: {'通过' if roundtrip_result == test_data else '失败'}")

def batch_processing_example():
    """批量处理示例"""
    print("\n📦 批量处理示例:")
    
    # 生成测试数据
    test_data_list = []
    for i in range(1000):
        data = {
            "id": i,
            "name": f"测试数据_{i}",
            "values": [j * 0.1 for j in range(i % 10 + 1)],
            "metadata": {
                "index": i,
                "category": f"cat_{i % 5}"
            }
        }
        test_data_list.append(data)
   # 使用模块级别的批量编码解码函数
    
    # 批量 bincode 编码
    start_time = time.time()
    batch_encoded = rat_quickmem.encode_batch(test_data_list)
    batch_encode_time = time.time() - start_time
    
    print(f"📊 批量 bincode 编码 {len(test_data_list)} 个对象:")
    print(f"   耗时: {batch_encode_time*1000:.3f}ms")
    total_size = sum(len(encoded) for encoded in batch_encoded)
    print(f"   大小: {total_size/1024:.2f} KB")
    print(f"   速度: {total_size/1024/1024/batch_encode_time:.2f} MB/s")
    
    # 批量 bincode 解码
    start_time = time.time()
    batch_decoded = rat_quickmem.decode_batch(batch_encoded)
    batch_decode_time = time.time() - start_time
    
    print(f"📊 批量 bincode 解码:")
    print(f"   耗时: {batch_decode_time*1000:.3f}ms")
    print(f"   速度: {total_size/1024/1024/batch_decode_time:.2f} MB/s")
    print(f"   验证: {'通过' if test_data_list == batch_decoded else '失败'}")

def bincode_showcase():
    """展示 RatQuickMem 的核心优势场景"""
    print("\n🎯 RatQuickMem 核心优势展示:")
    print("   专为 Rust ↔ Python 高性能数据交换设计")
    print("   使用 bincode 二进制协议，内存池优化")
    
    # 模拟 Rust 结构化数据（类似 gRPC 消息）
    structured_data = {
        "message_id": 12345,
        "timestamp": int(time.time() * 1000),  # 毫秒时间戳
        "payload": {
            "sensor_readings": [
                {"id": i, "value": i * 3.14159, "status": i % 3}
                for i in range(1000)  # 大量传感器数据
            ],
            "metadata": {
                "device_id": "sensor_array_001",
                "location": [116.3974, 39.9093],  # GPS 坐标
                "batch_size": 1000
            }
        }
    }
    
    print(f"\n📦 测试数据: {len(structured_data['payload']['sensor_readings'])} 个传感器读数")
    
    # 预热内存池
    for _ in range(3):
        rat_quickmem.encode({"warmup": True})
    
    # 批量编码测试（模拟 Rust 服务发送数据）
    iterations = 20
    batch_data = [structured_data for _ in range(10)]  # 10 个消息批次
    
    print(f"\n🚀 批量处理测试 ({len(batch_data)} 个消息, 平均 {iterations} 次):")
    
    # RatQuickMem 批量编码测试
    
    encode_times = []
    decode_times = []
    total_data_size = 0
    
    for _ in range(iterations):
        # bincode 编码
        start_time = time.time()
        encoded_batch = rat_quickmem.encode_batch(batch_data)
        encode_times.append(time.time() - start_time)
        
        # 计算数据大小（只在第一次迭代时计算）
        if total_data_size == 0:
            total_data_size = sum(len(encoded) for encoded in encoded_batch)
        
        # bincode 解码
        start_time = time.time()
        decoded_batch = rat_quickmem.decode_batch(encoded_batch)
        decode_times.append(time.time() - start_time)
    
    avg_encode = sum(encode_times) / iterations
    avg_decode = sum(decode_times) / iterations
    
    # 计算吞吐量
    throughput_encode = total_data_size / avg_encode / 1024 / 1024  # MB/s
    throughput_decode = total_data_size / avg_decode / 1024 / 1024  # MB/s
    
    print(f"📊 批量编码性能:")
    print(f"   平均耗时: {avg_encode*1000:.3f}ms")
    print(f"   数据大小: {total_data_size/1024:.2f} KB ({len(batch_data)} 个复杂消息)")
    print(f"   吞吐量: {throughput_encode:.2f} MB/s")
    
    print(f"📊 批量解码性能:")
    print(f"   平均耗时: {avg_decode*1000:.3f}ms")
    print(f"   吞吐量: {throughput_decode:.2f} MB/s")
    
    # 内存池效率测试
    print(f"\n🏊 内存池效率测试:")
    initial_stats = rat_quickmem.get_pool_stats()
    
    # 大量小数据编码（模拟高频消息）
    small_messages = [{"id": i, "value": i} for i in range(1000)]
    
    start_time = time.time()
    for msg in small_messages:
        rat_quickmem.encode(msg)
    pool_test_time = time.time() - start_time
    
    final_stats = rat_quickmem.get_pool_stats()
    
    print(f"   处理 {len(small_messages)} 个小消息: {pool_test_time*1000:.3f}ms")
    print(f"   平均每消息: {pool_test_time/len(small_messages)*1000000:.2f}μs")
    
    # 内存池复用率计算：如果池中有缓冲区可用，说明发生了复用
    # 理想情况下，处理大量小消息后池中应该有缓冲区积累
    if final_stats.total_buffers > 0:
        reuse_efficiency = min(100.0, (final_stats.total_buffers / max(1, len(small_messages) // 10)) * 100)
        print(f"   内存池效率: {reuse_efficiency:.1f}% (池中缓冲区: {final_stats.total_buffers})")
    else:
        print(f"   内存池效率: 0.0% (无缓冲区复用)")
    
    # 与原生 JSON 的适用场景对比
    print(f"\n💡 适用场景分析:")
    print(f"   ✅ RatQuickMem 优势场景:")
    print(f"      • Rust ↔ Python 数据交换")
    print(f"      • 大批量结构化数据传输")
    print(f"      • 内存敏感的高频操作")
    print(f"      • 网络传输（数据更紧凑）")
    print(f"   ⚠️  JSON 更适合:")
    print(f"      • 人类可读的配置文件")
    print(f"      • Web API 接口")
    print(f"      • 调试和开发阶段")
    
    print(f"\n🎯 总结: RatQuickMem 专注于高性能二进制数据交换，不是 JSON 的替代品")

def pool_management_example():
    """内存池管理示例"""
    print("\n🏊 内存池管理示例:")
    
    # 获取初始池统计
    initial_stats = rat_quickmem.get_pool_stats()
    print(f"📈 初始池统计:")
    print(f"   小缓冲区: {initial_stats.small_buffers}")
    print(f"   中等缓冲区: {initial_stats.medium_buffers}")
    print(f"   大缓冲区: {initial_stats.large_buffers}")
    print(f"   总缓冲区: {initial_stats.total_buffers}")
    
    # 预热池 - 通过执行一些编码操作来预热
    print("🔥 预热内存池...")
    for i in range(10):
        test_data = {"warmup": i, "data": "x" * 100}
        encoded = rat_quickmem.encode(test_data)
        decoded = rat_quickmem.decode(encoded)
    
    warmed_stats = rat_quickmem.get_pool_stats()
    print(f"📈 预热后统计:")
    print(f"   小缓冲区: {warmed_stats.small_buffers}")
    print(f"   中等缓冲区: {warmed_stats.medium_buffers}")
    print(f"   大缓冲区: {warmed_stats.large_buffers}")
    print(f"   总缓冲区: {warmed_stats.total_buffers}")
    
    # 执行一些操作
    encoder = rat_quickmem.QuickEncoder()
    test_data = [{"id": i, "data": list(range(100))} for i in range(50)]
    
    start_time = time.time()
    for data in test_data:
        encoder.encode(data)
    operation_time = time.time() - start_time
    
    after_stats = rat_quickmem.get_pool_stats()
    print(f"📈 操作后统计:")
    print(f"   小缓冲区: {after_stats.small_buffers}")
    print(f"   中等缓冲区: {after_stats.medium_buffers}")
    print(f"   大缓冲区: {after_stats.large_buffers}")
    print(f"   总缓冲区: {after_stats.total_buffers}")
    print(f"   操作耗时: {operation_time*1000:.3f}ms")

def bytes_handling_example():
    """字节数据处理示例"""
    print("\n🔢 字节数据处理示例:")
    
    # 原始字节数据
    raw_data = b"\x00\x01\x02\x03" * 1000  # 4KB 的二进制数据
   # 使用模块级别的批量编码解码函数
    
    # 编码字节数据
    start_time = time.time()
    encoded_bytes = rat_quickmem.encode(raw_data)
    encode_time = time.time() - start_time
    
    print(f"📦 字节编码:")
    print(f"   原始大小: {len(raw_data)} 字节")
    print(f"   编码大小: {len(encoded_bytes)} 字节")
    print(f"   编码耗时: {encode_time*1000:.3f}ms")
    
    # 解码字节数据
    start_time = time.time()
    decoded_bytes = rat_quickmem.decode(encoded_bytes)
    decode_time = time.time() - start_time
    
    print(f"📦 字节解码:")
    print(f"   解码耗时: {decode_time*1000:.3f}ms")
    print(f"   数据验证: {'通过' if raw_data == decoded_bytes else '失败'}")

def main():
    """主函数"""
    print("🚀 RatQuickMem Python 绑定示例")
    print("=" * 50)
    
    try:
        # 运行各种示例
        basic_example()
        codec_class_example()
        batch_processing_example()
        bincode_showcase()
        pool_management_example()
        bytes_handling_example()
        security_example()
        
        print("\n🎉 所有示例运行完成!")
        
    except Exception as e:
        print(f"❌ 运行出错: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    main()