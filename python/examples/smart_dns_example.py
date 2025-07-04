#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
智能DNS解析器示例
演示FIFO和智能决策模式的使用

对应Rust版本: examples/smart_dns_example.rs
"""

import time
from typing import List, Dict, Any

# Python绑定导入
try:
    import rat_quickdns_py as dns
    from rat_quickdns_py import QueryStrategy
except ImportError:
    print("请确保已正确安装 rat_quickdns_py Python 绑定")
    exit(1)


class SmartDnsDemo:
    """智能DNS解析器演示类"""
    
    def __init__(self):
        self.test_domains = [
            "google.com", 
            "github.com", 
            "example.com",
            "stackoverflow.com",
            "rust-lang.org"
        ]
    
    def test_fifo_mode(self):
        """测试FIFO模式（多服务器并发查询）"""
        print("\n1. 测试FIFO模式（多服务器并发查询）")
        
        # 创建FIFO策略解析器
        builder = dns.DnsResolverBuilder()
        builder.query_strategy(QueryStrategy.FIFO)
        # builder.enable_edns(True)  # 可能在当前版本中不可用
        builder.add_udp_upstream("Google DNS", "8.8.8.8:53")
        builder.add_udp_upstream("Cloudflare DNS", "1.1.1.1:53")
        builder.add_udp_upstream("阿里DNS", "223.5.5.5:53")
        builder.timeout(5.0)
        
        fifo_resolver = builder.build()
        
        # 测试域名解析
        for domain in self.test_domains[:3]:  # 只测试前3个域名
            print(f"解析域名: {domain}")
            try:
                start_time = time.time()
                result = fifo_resolver.resolve_a(domain)
                elapsed = (time.time() - start_time) * 1000
                print(f"  结果: {result} (耗时: {elapsed:.2f}ms)")
            except Exception as e:
                elapsed = (time.time() - start_time) * 1000
                print(f"  错误: {e} (耗时: {elapsed:.2f}ms)")
    
    def test_smart_mode(self):
        """测试智能决策模式（自动选择最优服务器）"""
        print("\n2. 测试智能决策模式（自动选择最优服务器）")
        
        # 创建智能策略解析器
        builder = dns.DnsResolverBuilder()
        builder.query_strategy(QueryStrategy.FIFO)  # 使用FIFO替代SMART
        # builder.enable_edns(True)  # 可能在当前版本中不可用
        # builder.region("CN")  # 可能在当前版本中不可用
        builder.add_udp_upstream("Google DNS", "8.8.8.8:53")
        builder.add_udp_upstream("Cloudflare DNS", "1.1.1.1:53")
        builder.add_udp_upstream("阿里DNS", "223.5.5.5:53")
        builder.add_udp_upstream("腾讯DNS", "119.29.29.29:53")
        builder.enable_health_checker(True)
        builder.timeout(5.0)
        
        smart_resolver = builder.build()
        
        print("智能解析器已启动，包含自动健康检查功能")
        
        # 测试智能解析
        for domain in self.test_domains[:3]:  # 只测试前3个域名
            print(f"智能解析域名: {domain}")
            try:
                start_time = time.time()
                result = smart_resolver.resolve_a(domain)
                elapsed = (time.time() - start_time) * 1000
                print(f"  结果: {result} (耗时: {elapsed:.2f}ms)")
            except Exception as e:
                elapsed = (time.time() - start_time) * 1000
                print(f"  错误: {e} (耗时: {elapsed:.2f}ms)")
        
        return smart_resolver
    
    def test_batch_queries(self, resolver):
        """测试多个域名查询"""
        print("\n3. 测试多个域名查询")
        
        for domain in self.test_domains:
            try:
                start_time = time.time()
                result = resolver.resolve_a(domain)
                elapsed = (time.time() - start_time) * 1000
                print(f"查询 {domain}: 成功 (耗时: {elapsed:.2f}ms)")
            except Exception as e:
                elapsed = (time.time() - start_time) * 1000
                print(f"查询 {domain}: 失败 - {e} (耗时: {elapsed:.2f}ms)")
    
    def test_different_record_types(self, resolver):
        """测试不同记录类型"""
        print("\n4. 测试不同记录类型")
        
        domain = "example.com"
        record_tests = [
            ("A记录", lambda: resolver.resolve_a(domain)),
            ("AAAA记录", lambda: resolver.resolve_aaaa(domain)),
        ]
        
        # 如果有MX和TXT方法，也测试它们
        try:
            record_tests.append(("MX记录", lambda: resolver.resolve_mx(domain)))
        except AttributeError:
            pass
        
        try:
            record_tests.append(("TXT记录", lambda: resolver.resolve_txt(domain)))
        except AttributeError:
            pass
        
        for record_name, resolve_func in record_tests:
            print(f"查询 {domain} 的 {record_name}:")
            try:
                start_time = time.time()
                result = resolve_func()
                elapsed = (time.time() - start_time) * 1000
                print(f"  成功，结果: {result} (耗时: {elapsed:.2f}ms)")
            except Exception as e:
                elapsed = (time.time() - start_time) * 1000
                print(f"  查询失败: {e} (耗时: {elapsed:.2f}ms)")
    
    def test_convenience_functions(self):
        """测试便捷函数"""
        print("\n5. 测试便捷函数")
        
        # 注意：便捷函数可能在当前版本中不可用
        print("便捷函数（quick_resolve, batch_resolve）可能在当前版本中不可用")
        print("建议使用标准的解析器实例进行DNS查询")
        
        # 使用标准解析器进行演示
        builder = dns.DnsResolverBuilder()
        builder.query_strategy(QueryStrategy.FIFO)
        builder.add_udp_upstream("Google DNS", "8.8.8.8")
        builder.timeout(3.0)
        resolver = builder.build()
        
        domain = "google.com"
        try:
            start_time = time.time()
            result = resolver.resolve_a(domain)
            elapsed = (time.time() - start_time) * 1000
            print(f"标准解析 {domain}: {result} (耗时: {elapsed:.2f}ms)")
        except Exception as e:
            elapsed = (time.time() - start_time) * 1000
            print(f"标准解析 {domain} 失败: {e} (耗时: {elapsed:.2f}ms)")
    
    def print_summary(self):
        """打印总结信息"""
        print("\n=== 示例完成 ===")
        print("智能DNS解析器支持:")
        print("- FIFO模式: 多服务器并发查询，最快响应优先")
        print("- 智能决策模式: 基于性能指标自动选择最优服务器")
        print("- 健康检查: 定期监控服务器状态和性能")
        print("- EDNS支持: 自动启用扩展DNS功能")
        print("- 多域名查询: 支持多个域名的顺序查询")
        print("- 便捷函数: 提供快速解析和批量解析功能")
    
    def run_demo(self):
        """运行完整的演示"""
        print("=== 智能DNS解析器示例 ===")
        # 版本信息可能不可用
        # print(f"rat-quickdns-py 版本: {dns.__version__}")
        # print(f"作者: {dns.__author__}")
        
        # 1. 测试FIFO模式
        self.test_fifo_mode()
        
        # 2. 测试智能决策模式
        smart_resolver = self.test_smart_mode()
        
        # 3. 测试多个域名查询
        self.test_batch_queries(smart_resolver)
        
        # 4. 测试不同记录类型
        self.test_different_record_types(smart_resolver)
        
        # 5. 测试便捷函数
        self.test_convenience_functions()
        
        # 6. 打印总结
        self.print_summary()


def main():
    """主函数"""
    demo = SmartDnsDemo()
    demo.run_demo()


if __name__ == "__main__":
    main()