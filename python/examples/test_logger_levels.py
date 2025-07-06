#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
日志级别测试脚本

这个脚本专门用于测试不同的日志级别功能，包括：
- 调试级别日志（显示详细信息）
- 静默模式（无日志输出）
- 自动模式（根据环境自动选择）
"""

import time
import sys

try:
    import rat_quickdns_py as dns
    from rat_quickdns_py import QueryStrategy
except ImportError:
    print("请确保已正确安装 rat_quickdns_py Python 绑定")
    sys.exit(1)


def test_debug_logger():
    """测试调试级别日志"""
    print("\n=== 测试调试级别日志 ===")
    print("应该看到详细的DNS解析过程信息：")
    
    builder = dns.DnsResolverBuilder()
    builder.query_strategy(QueryStrategy.FIFO)
    builder.with_debug_logger_init()  # 启用调试级别日志
    builder.add_udp_upstream("Google DNS", "8.8.8.8")
    builder.timeout(3.0)
    
    resolver = builder.build()
    
    try:
        result = resolver.resolve_a("example.com")
        print(f"解析结果: {result}")
    except Exception as e:
        print(f"解析失败: {e}")
    
    del resolver
    import gc
    gc.collect()
    time.sleep(0.2)


def test_silent_logger():
    """测试静默模式"""
    print("\n=== 测试静默模式 ===")
    print("应该看不到任何调试信息，只有解析结果：")
    
    builder = dns.DnsResolverBuilder()
    builder.query_strategy(QueryStrategy.FIFO)
    builder.with_silent_logger_init()  # 启用静默模式
    builder.add_udp_upstream("Google DNS", "8.8.8.8")
    builder.timeout(3.0)
    
    resolver = builder.build()
    
    try:
        result = resolver.resolve_a("example.com")
        print(f"解析结果: {result}")
    except Exception as e:
        print(f"解析失败: {e}")
    
    del resolver
    import gc
    gc.collect()
    time.sleep(0.2)


def test_auto_logger():
    """测试自动模式"""
    print("\n=== 测试自动模式 ===")
    print("根据环境自动选择日志级别：")
    
    builder = dns.DnsResolverBuilder()
    builder.query_strategy(QueryStrategy.FIFO)
    builder.with_auto_logger_init()  # 启用自动模式
    builder.add_udp_upstream("Google DNS", "8.8.8.8")
    builder.timeout(3.0)
    
    resolver = builder.build()
    
    try:
        result = resolver.resolve_a("example.com")
        print(f"解析结果: {result}")
    except Exception as e:
        print(f"解析失败: {e}")
    
    del resolver
    import gc
    gc.collect()
    time.sleep(0.2)


def main():
    """主函数"""
    print("=== rat-quickdns-py 日志级别测试 ===")
    
    try:
        # 测试调试级别日志
        test_debug_logger()
        
        # 测试静默模式
        test_silent_logger()
        
        # 测试自动模式
        test_auto_logger()
        
        print("\n=== 测试完成 ===")
        print("说明：")
        print("- 调试模式：显示详细的DNS解析过程信息")
        print("- 静默模式：不显示任何调试信息")
        print("- 自动模式：根据环境变量自动选择合适的日志级别")
        
    except Exception as e:
        print(f"\n测试过程中发生错误: {e}")
        import traceback
        traceback.print_exc()
    finally:
        print("\n=== 最终清理 ===")
        import gc
        gc.collect()
        print("资源清理完成")


if __name__ == "__main__":
    main()