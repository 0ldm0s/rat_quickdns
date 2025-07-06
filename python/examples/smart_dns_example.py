#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
DnsResolverBuilder 统一架构示例

本示例展示如何使用 DnsResolverBuilder 统一架构进行DNS解析
包括FIFO、智能决策和轮询三种查询策略的使用方法

主要特性:
- 统一的构建接口，支持所有DNS协议(UDP/TCP/DoH/DoT)
- 多种查询策略：FIFO、Smart、RoundRobin
- 自动健康检查和性能监控
- 详细的统计信息和日志输出

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
    """DnsResolverBuilder 统一架构演示类"""
    
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
        builder.query_strategy(QueryStrategy.SMART)  # 现在SMART策略已修复
        builder.enable_edns(True)  # 启用EDNS支持
        builder.region("CN")  # 设置区域为中国
        builder.add_udp_upstream("Google DNS", "8.8.8.8:53")
        builder.add_udp_upstream("Cloudflare DNS", "1.1.1.1:53")
        builder.add_udp_upstream("阿里DNS", "223.5.5.5:53")
        builder.add_udp_upstream("腾讯DNS", "119.29.29.29:53")
        builder.enable_upstream_monitoring(True)  # 启用上游监控
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
    
    def test_mixed_protocols(self):
        """测试混合协议模式（UDP/DoH/DoT）"""
        print("\n3. 测试混合协议模式（UDP/DoH/DoT）")
        
        # 创建混合协议解析器
        builder = dns.DnsResolverBuilder()
        builder.query_strategy(QueryStrategy.SMART)
        builder.enable_edns(True)
        builder.region("global")
        
        # 添加不同协议的上游服务器
        builder.add_udp_upstream("Google UDP", "8.8.8.8:53")
        builder.add_tcp_upstream("Cloudflare TCP", "1.1.1.1:53")
        builder.add_doh_upstream("Google DoH", "https://dns.google/dns-query")
        builder.add_dot_upstream("Cloudflare DoT", "1.1.1.1:853")
        
        builder.enable_upstream_monitoring(True)
        builder.timeout(5.0)
        
        mixed_resolver = builder.build()
        
        print("混合协议解析器已启动，支持UDP/TCP/DoH/DoT")
        
        # 测试混合协议解析
        for domain in self.test_domains[:3]:  # 只测试前3个域名
            print(f"混合协议解析域名: {domain}")
            try:
                start_time = time.time()
                result = mixed_resolver.resolve_a(domain)
                elapsed = (time.time() - start_time) * 1000
                print(f"  结果: {result} (耗时: {elapsed:.2f}ms)")
            except Exception as e:
                elapsed = (time.time() - start_time) * 1000
                print(f"  错误: {e} (耗时: {elapsed:.2f}ms)")
        
        return mixed_resolver
    
    def test_round_robin_mode(self):
        """测试轮询模式（负载均衡）"""
        print("\n4. 测试轮询模式（负载均衡）")
        
        # 创建轮询策略解析器
        builder = dns.DnsResolverBuilder()
        builder.query_strategy(QueryStrategy.ROUND_ROBIN)
        builder.enable_edns(True)
        builder.region("global")
        
        # 添加多个上游服务器进行负载均衡
        builder.add_udp_upstream("Google DNS", "8.8.8.8:53")
        builder.add_udp_upstream("Cloudflare DNS", "1.1.1.1:53")
        builder.add_udp_upstream("阿里DNS", "223.5.5.5:53")
        builder.add_udp_upstream("腾讯DNS", "119.29.29.29:53")
        
        # 启用轮询优化
        builder.optimize_for_round_robin()
        builder.enable_upstream_monitoring(True)
        builder.timeout(5.0)
        
        round_robin_resolver = builder.build()
        
        print("轮询解析器已创建，启用轮询优化")
        
        # 测试轮询解析
        batch_domains = ["google.com", "github.com", "stackoverflow.com", "rust-lang.org"]
        for i, domain in enumerate(batch_domains):
            print(f"轮询解析域名 {i+1}: {domain}")
            try:
                start_time = time.time()
                result = round_robin_resolver.resolve_a(domain)
                elapsed = (time.time() - start_time) * 1000
                print(f"  结果: {result} (耗时: {elapsed:.2f}ms)")
            except Exception as e:
                elapsed = (time.time() - start_time) * 1000
                print(f"  错误: {e} (耗时: {elapsed:.2f}ms)")
        
        return round_robin_resolver
    
    def test_batch_queries(self, resolver):
        """测试多个域名查询"""
        print("\n5. 测试多个域名查询")
        
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
        print("\n6. 测试不同记录类型")
        
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
        print("\n7. 测试便捷函数")
        
        # 注意：便捷函数可能在当前版本中不可用
        print("注意：便捷函数已被移除，请使用构建器模式创建解析器")
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
        print("- 轮询模式: 负载均衡，轮流使用各个上游服务器")
        print("- 混合协议: 支持UDP/TCP/DoH/DoT混合使用")
        print("- 健康检查: 定期监控服务器状态和性能")
        print("- EDNS支持: 自动启用扩展DNS功能")
        print("- 多域名查询: 支持多个域名的顺序查询")
        print("- 便捷函数: 提供快速解析和批量解析功能")
        print("- 统一接口: 所有协议(UDP/DoH/DoT)使用相同的构建方式")
    
    def run_demo(self):
        """运行完整的演示"""
        print("=== DnsResolverBuilder 统一架构示例 ===")
        # 版本信息可能不可用
        # print(f"rat-quickdns-py 版本: {dns.__version__}")
        # print(f"作者: {dns.__author__}")
        
        try:
            # 1. 测试FIFO模式
            print("\n=== 测试1: FIFO模式 ===")
            self.test_fifo_mode()
            
            # 强制垃圾回收，确保前一个解析器被清理
            import gc
            gc.collect()
            import time
            time.sleep(0.1)  # 给Rust端一点时间清理资源
            
            # 2. 测试智能决策模式
            print("\n=== 测试2: 智能决策模式 ===")
            smart_resolver = self.test_smart_mode()
            
            # 5. 测试多个域名查询
            print("\n=== 测试5: 多域名查询 ===")
            self.test_batch_queries(smart_resolver)
            
            # 清理智能解析器
            del smart_resolver
            gc.collect()
            time.sleep(0.1)
            
            # 3. 测试混合协议模式
            print("\n=== 测试3: 混合协议模式 ===")
            mixed_resolver = self.test_mixed_protocols()
            
            # 6. 测试不同记录类型
            print("\n=== 测试6: 不同记录类型 ===")
            self.test_different_record_types(mixed_resolver)
            
            # 清理混合协议解析器
            del mixed_resolver
            gc.collect()
            time.sleep(0.1)
            
            # 4. 测试轮询模式
            print("\n=== 测试4: 轮询模式 ===")
            round_robin_resolver = self.test_round_robin_mode()
            
            # 清理轮询解析器
            del round_robin_resolver
            gc.collect()
            time.sleep(0.1)
            
            # 7. 测试便捷函数
            print("\n=== 测试7: 便捷函数 ===")
            self.test_convenience_functions()
            
            # 8. 打印总结
            self.print_summary()
            
        except Exception as e:
            print(f"\n演示过程中发生错误: {e}")
            import traceback
            traceback.print_exc()
        finally:
            # 最终清理
            print("\n=== 最终清理 ===")
            import gc
            gc.collect()
            print("资源清理完成")


def main():
    """主函数"""
    import gc
    import sys
    
    try:
        demo = SmartDnsDemo()
        demo.run_demo()
    except KeyboardInterrupt:
        print("\n程序被用户中断")
    except Exception as e:
        print(f"\n程序执行出错: {e}")
    finally:
        print("\n=== 最终清理 ===")
        # 删除demo实例
        if 'demo' in locals():
            del demo
        
        # 强制垃圾回收
        print("执行垃圾回收...")
        gc.collect()
        
        # 给一点时间让Rust清理资源
        import time
        time.sleep(0.5)
        
        print("程序即将退出")
        sys.exit(0)


if __name__ == "__main__":
    main()