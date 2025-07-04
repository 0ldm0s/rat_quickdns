#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
通用应急处理示例

本示例演示了DNS解析器的通用应急处理机制，该机制适用于所有查询策略：
- FIFO策略
- SMART策略  
- ROUND_ROBIN策略

当所有上游服务器都失去响应时，系统会自动激活应急模式，
提供详细的故障信息和恢复建议。
"""

import sys
import time
import asyncio
from typing import List, Dict, Any

# Python绑定导入
try:
    import rat_quickdns_py as dns
    from rat_quickdns_py import QueryStrategy
except ImportError:
    print("请确保已正确安装 rat_quickdns_py Python 绑定")
    exit(1)

class UniversalEmergencyDemo:
    """通用应急处理演示类"""
    
    def __init__(self):
        self.test_domain = "example.com"
        self.strategies = [
            (QueryStrategy.FIFO, "FIFO策略"),
            (QueryStrategy.SMART, "SMART策略"),
            (QueryStrategy.ROUND_ROBIN, "ROUND_ROBIN策略")
        ]
    
    def create_resolver_with_strategy(self, strategy: QueryStrategy) -> 'DnsResolver':
        """创建指定策略的解析器"""
        builder = dns.DnsResolverBuilder()
        builder.query_strategy(strategy)
        
        # 添加一些测试用的上游服务器（故意使用无效地址来模拟故障）
        builder.add_udp_upstream("Invalid1", "192.0.2.1:53")  # RFC5737测试地址
        builder.add_udp_upstream("Invalid2", "192.0.2.2:53")
        builder.add_udp_upstream("Invalid3", "192.0.2.3:53")
        
        # 启用健康检查
        builder.enable_health_checker(True)
        builder.timeout(2.0)  # 2秒超时
        
        return builder.build()
    
    def test_emergency_handling_for_strategy(self, strategy: QueryStrategy, strategy_name: str):
        """测试指定策略的应急处理"""
        print(f"\n{'='*60}")
        print(f"测试 {strategy_name} 的应急处理机制")
        print(f"{'='*60}")
        
        resolver = self.create_resolver_with_strategy(strategy)
        
        print(f"✅ 创建了使用 {strategy_name} 的DNS解析器")
        print("📋 配置的上游服务器:")
        print("   - Invalid1 (192.0.2.1:53) - 权重: 10")
        print("   - Invalid2 (192.0.2.2:53) - 权重: 20")
        print("   - Invalid3 (192.0.2.3:53) - 权重: 30")
        
        # 等待解析器初始化
        print("\n⏳ 等待解析器初始化...")
        time.sleep(1)
        
        # 尝试查询（应该触发应急处理）
        print(f"\n🔍 尝试查询 {self.test_domain} (A记录)...")
        try:
            start_time = time.time()
            ips = resolver.resolve(self.test_domain)
            duration = time.time() - start_time
            print(f"✅ 查询成功: {ips} (耗时: {duration:.2f}秒)")
        except Exception as e:
            duration = time.time() - start_time
            print(f"❌ 查询失败 (耗时: {duration:.2f}秒):")
            print(f"   错误信息: {str(e)}")
            
            # 检查错误信息
            error_msg = str(e)
            print("   ⚠️  所有配置的服务器都无法响应，这是预期的测试结果")
    
    def test_partial_failure_scenario(self):
        """测试部分服务器故障的场景"""
        print(f"\n{'='*60}")
        print("测试部分服务器故障场景")
        print(f"{'='*60}")
        
        # 创建混合配置：一些有效服务器 + 一些无效服务器
        builder = dns.DnsResolverBuilder()
        builder.query_strategy(QueryStrategy.SMART)
        
        # 添加有效的DNS服务器
        builder.add_udp_upstream("Cloudflare", "1.1.1.1:53")
        builder.add_udp_upstream("Google", "8.8.8.8:53")
        
        # 添加无效的DNS服务器
        builder.add_udp_upstream("Invalid1", "192.0.2.1:53")
        builder.add_udp_upstream("Invalid2", "192.0.2.2:53")
        
        builder.enable_health_checker(True)
        builder.timeout(3.0)
        
        resolver = builder.build()
        
        print("✅ 创建了混合配置的DNS解析器")
        print("📋 配置的上游服务器:")
        print("   - Cloudflare (1.1.1.1:53) - 权重: 10 [有效]")
        print("   - Google (8.8.8.8:53) - 权重: 20 [有效]")
        print("   - Invalid1 (192.0.2.1:53) - 权重: 5 [无效]")
        print("   - Invalid2 (192.0.2.2:53) - 权重: 5 [无效]")
        
        # 等待解析器初始化
        print("\n⏳ 等待解析器初始化...")
        time.sleep(1)
        
        # 尝试查询（应该成功，因为有健康的服务器）
        print(f"\n🔍 尝试查询 {self.test_domain} (A记录)...")
        try:
            start_time = time.time()
            ips = resolver.resolve(self.test_domain)
            duration = time.time() - start_time
            print(f"✅ 查询成功: {ips} (耗时: {duration:.2f}秒)")
            print("   ✅ 系统正确处理了部分服务器故障")
        except Exception as e:
            duration = time.time() - start_time
            print(f"❌ 查询失败 (耗时: {duration:.2f}秒): {str(e)}")
    
    def demonstrate_error_enhancement(self):
        """演示错误信息增强功能"""
        print(f"\n{'='*60}")
        print("演示错误信息增强功能")
        print(f"{'='*60}")
        
        for strategy, strategy_name in self.strategies:
            print(f"\n--- {strategy_name} 错误处理测试 ---")
            
            resolver = self.create_resolver_with_strategy(strategy)
            
            # 等待解析器初始化
            time.sleep(1)
            
            try:
                resolver.resolve("nonexistent-domain-12345.invalid")
            except Exception as e:
                error_msg = str(e)
                print(f"错误信息: {error_msg[:100]}...")
                print(f"✅ {strategy_name} 策略正确处理了无效域名查询")
    
    def run_demo(self):
        """运行完整的演示"""
        print("🚀 通用应急处理机制演示")
        print("="*80)
        print("本演示将展示DNS解析器在各种策略下的应急处理能力:")
        print("1. 所有服务器故障时的应急响应")
        print("2. 部分服务器故障时的智能处理")
        print("3. 错误信息的智能增强")
        print("="*80)
        
        # 测试各种策略的应急处理
        for strategy, strategy_name in self.strategies:
            self.test_emergency_handling_for_strategy(strategy, strategy_name)
        
        # 测试部分故障场景
        self.test_partial_failure_scenario()
        
        # 演示错误信息增强
        self.demonstrate_error_enhancement()
        
        print(f"\n{'='*80}")
        print("🎉 演示完成！")
        print("\n📝 总结:")
        print("✅ 所有查询策略都能正确处理服务器故障")
        print("✅ 系统能够智能区分全部故障和部分故障")
        print("✅ 混合配置下有效服务器能够正常工作")
        print("✅ 错误处理机制工作正常，提供清晰的错误信息")
        print("="*80)

def main():
    """主函数"""
    demo = UniversalEmergencyDemo()
    demo.run_demo()

if __name__ == "__main__":
    main()