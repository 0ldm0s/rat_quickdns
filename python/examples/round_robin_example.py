#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
轮询策略DNS解析器示例
演示ROUND_ROBIN模式的使用和优化

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


class RoundRobinDnsDemo:
    """轮询DNS解析器演示类"""
    
    def __init__(self):
        self.test_domains = [
            "google.com", 
            "github.com", 
            "example.com",
            "stackoverflow.com",
            "rust-lang.org"
        ]
    
    def test_round_robin_mode(self):
        """测试轮询模式（依次使用不同服务器）"""
        print("\n1. 测试轮询模式（依次使用不同服务器）")
        
        # 创建轮询策略解析器
        builder = dns.DnsResolverBuilder()
        builder.query_strategy(QueryStrategy.ROUND_ROBIN)
        builder.add_udp_upstream("Google DNS", "8.8.8.8:53")
        builder.add_udp_upstream("Cloudflare DNS", "1.1.1.1:53")
        builder.add_udp_upstream("阿里DNS", "223.5.5.5:53")
        builder.add_udp_upstream("腾讯DNS", "119.29.29.29:53")
        builder.timeout(5.0)
        
        round_robin_resolver = builder.build()
        
        # 测试域名解析
        for i, domain in enumerate(self.test_domains[:4]):  # 测试4个域名
            print(f"解析域名 {i+1}: {domain}")
            try:
                start_time = time.time()
                result = round_robin_resolver.resolve_a(domain)
                elapsed = (time.time() - start_time) * 1000
                print(f"  结果: {result} (耗时: {elapsed:.2f}ms)")
            except Exception as e:
                elapsed = (time.time() - start_time) * 1000
                print(f"  错误: {e} (耗时: {elapsed:.2f}ms)")
        
        return round_robin_resolver
    
    def test_optimized_round_robin(self):
        """测试优化的轮询模式"""
        print("\n2. 测试优化的轮询模式")
        
        # 创建优化的轮询策略解析器
        builder = dns.DnsResolverBuilder()
        builder.query_strategy(QueryStrategy.ROUND_ROBIN)
        builder.add_udp_upstream("Google DNS", "8.8.8.8:53")
        builder.add_udp_upstream("Cloudflare DNS", "1.1.1.1:53")
        builder.add_udp_upstream("阿里DNS", "223.5.5.5:53")
        builder.add_udp_upstream("腾讯DNS", "119.29.29.29:53")
        
        # 应用轮询优化
        builder.optimize_for_round_robin()  # 自动应用所有优化
        
        # 或者手动设置优化参数
        # builder.round_robin_timeout(1.5)  # 设置较短的超时时间
        # builder.enable_upstream_monitoring(True)  # 启用监控
        
        optimized_resolver = builder.build()
        
        print("优化的轮询解析器已启动，包含性能优化配置")
        
        # 测试优化后的解析性能
        for i, domain in enumerate(self.test_domains):
            print(f"优化解析域名 {i+1}: {domain}")
            try:
                start_time = time.time()
                result = optimized_resolver.resolve_a(domain)
                elapsed = (time.time() - start_time) * 1000
                print(f"  结果: {result} (耗时: {elapsed:.2f}ms)")
            except Exception as e:
                elapsed = (time.time() - start_time) * 1000
                print(f"  错误: {e} (耗时: {elapsed:.2f}ms)")
        
        return optimized_resolver
    
    def test_mixed_transport_round_robin(self):
        """测试混合传输协议的轮询模式"""
        print("\n3. 测试混合传输协议的轮询模式")
        
        # 创建混合传输协议的轮询解析器
        builder = dns.DnsResolverBuilder()
        builder.query_strategy(QueryStrategy.ROUND_ROBIN)
        
        # 添加不同传输协议的上游服务器
        builder.add_udp_upstream("Google UDP", "8.8.8.8:53")
        builder.add_tcp_upstream("Cloudflare TCP", "1.1.1.1:53")
        builder.add_doh_upstream("Google DoH", "https://dns.google/dns-query")
        builder.add_dot_upstream("Cloudflare DoT", "1.1.1.1:853")
        
        builder.enable_edns(True)
        builder.timeout(3.0)
        
        mixed_resolver = builder.build()
        
        print("混合传输协议轮询解析器已启动")
        
        # 测试混合协议解析
        for i, domain in enumerate(self.test_domains[:3]):
            print(f"混合协议解析域名 {i+1}: {domain}")
            try:
                start_time = time.time()
                result = mixed_resolver.resolve_a(domain)
                elapsed = (time.time() - start_time) * 1000
                print(f"  结果: {result} (耗时: {elapsed:.2f}ms)")
            except Exception as e:
                elapsed = (time.time() - start_time) * 1000
                print(f"  错误: {e} (耗时: {elapsed:.2f}ms)")
        
        return mixed_resolver
    
    def compare_strategies(self):
        """比较不同策略的性能"""
        print("\n4. 比较不同策略的性能")
        
        strategies = [
            ("FIFO", QueryStrategy.FIFO),
            ("SMART", QueryStrategy.SMART),
            ("ROUND_ROBIN", QueryStrategy.ROUND_ROBIN)
        ]
        
        test_domain = "example.com"
        results = {}
        
        for strategy_name, strategy in strategies:
            print(f"\n测试 {strategy_name} 策略:")
            
            # 创建解析器
            builder = dns.DnsResolverBuilder()
            builder.query_strategy(strategy)
            builder.add_udp_upstream("Google DNS", "8.8.8.8:53")
            builder.add_udp_upstream("Cloudflare DNS", "1.1.1.1:53")
            builder.add_udp_upstream("阿里DNS", "223.5.5.5:53")
            builder.timeout(3.0)
            
            if strategy == QueryStrategy.ROUND_ROBIN:
                builder.optimize_for_round_robin()
            
            resolver = builder.build()
            
            # 测试多次查询并计算平均时间
            times = []
            for i in range(3):
                try:
                    start_time = time.time()
                    result = resolver.resolve_a(test_domain)
                    elapsed = (time.time() - start_time) * 1000
                    times.append(elapsed)
                    print(f"  查询 {i+1}: {elapsed:.2f}ms")
                except Exception as e:
                    print(f"  查询 {i+1}: 失败 - {e}")
            
            if times:
                avg_time = sum(times) / len(times)
                results[strategy_name] = avg_time
                print(f"  平均耗时: {avg_time:.2f}ms")
            else:
                results[strategy_name] = float('inf')
                print(f"  平均耗时: 失败")
        
        # 显示比较结果
        print("\n=== 性能比较结果 ===")
        sorted_results = sorted(results.items(), key=lambda x: x[1])
        for i, (strategy, time_ms) in enumerate(sorted_results, 1):
            if time_ms == float('inf'):
                print(f"{i}. {strategy}: 失败")
            else:
                print(f"{i}. {strategy}: {time_ms:.2f}ms")
    
    def print_summary(self):
        """打印总结信息"""
        print("\n=== 轮询策略示例完成 ===")
        print("轮询DNS解析器特点:")
        print("- 负载均衡: 依次使用不同的上游服务器")
        print("- 故障转移: 自动跳过失败的服务器")
        print("- 性能优化: 支持专门的轮询优化配置")
        print("- 混合协议: 支持UDP、TCP、DoH、DoT混合使用")
        print("- 监控支持: 可启用上游服务器监控")
        print("\n建议使用场景:")
        print("- 需要负载均衡的高并发环境")
        print("- 对单个服务器依赖度要求较低的场景")
        print("- 需要测试多个DNS服务器性能的场景")
    
    def run_demo(self):
        """运行完整的演示"""
        print("=== 轮询策略DNS解析器示例 ===")
        
        # 1. 测试基本轮询模式
        self.test_round_robin_mode()
        
        # 2. 测试优化的轮询模式
        self.test_optimized_round_robin()
        
        # 3. 测试混合传输协议
        self.test_mixed_transport_round_robin()
        
        # 4. 比较不同策略性能
        self.compare_strategies()
        
        # 5. 打印总结
        self.print_summary()


def main():
    """主函数"""
    demo = RoundRobinDnsDemo()
    demo.run_demo()


if __name__ == "__main__":
    main()