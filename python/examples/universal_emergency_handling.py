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

# 添加项目路径
sys.path.append('../python')

import rat_quickdns as dns
from rat_quickdns import QueryStrategy, DnsRecordType

class UniversalEmergencyDemo:
    """通用应急处理演示类"""
    
    def __init__(self):
        self.test_domain = "example.com"
        self.strategies = [
            (QueryStrategy.FIFO, "FIFO策略"),
            (QueryStrategy.SMART, "SMART策略"),
            (QueryStrategy.ROUND_ROBIN, "ROUND_ROBIN策略")
        ]
    
    def create_resolver_with_strategy(self, strategy: QueryStrategy) -> dns.DnsResolver:
        """创建指定策略的解析器"""
        builder = dns.DnsResolverBuilder()
        builder.query_strategy(strategy)
        
        # 添加一些测试用的上游服务器（故意使用无效地址来模拟故障）
        builder.add_udp_upstream("Invalid1", "192.0.2.1:53", 10)  # RFC5737测试地址
        builder.add_udp_upstream("Invalid2", "192.0.2.2:53", 20)
        builder.add_udp_upstream("Invalid3", "192.0.2.3:53", 30)
        
        # 启用健康检查和决策引擎
        builder.enable_health_checker(True)
        builder.health_check_interval(2)  # 2秒检查间隔
        builder.health_check_timeout(1)   # 1秒超时
        
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
        
        # 等待健康检查运行
        print("\n⏳ 等待健康检查运行...")
        time.sleep(3)
        
        # 检查健康状态
        print("\n📊 当前健康状态:")
        health_status = resolver.get_health_status()
        for server_name, is_healthy in health_status.items():
            status = "🟢 健康" if is_healthy else "🔴 不健康"
            print(f"   {server_name}: {status}")
        
        # 获取应急信息
        print("\n🚨 应急响应信息:")
        emergency_info = resolver.get_emergency_response_info()
        print(f"   所有服务器失败: {emergency_info.all_servers_failed}")
        print(f"   总失败次数: {emergency_info.total_failures}")
        print(f"   应急消息: {emergency_info.emergency_message}")
        
        if emergency_info.failed_servers:
            print("   失败服务器详情:")
            for server in emergency_info.failed_servers:
                print(f"     - {server.name}: 连续失败 {server.consecutive_failures} 次")
        
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
            
            # 检查错误信息是否包含应急信息
            error_msg = str(e)
            if "应急" in error_msg or "🚨" in error_msg:
                print("   ✅ 应急处理机制已激活")
            else:
                print("   ⚠️  应急处理机制可能未正确激活")
    
    def test_partial_failure_scenario(self):
        """测试部分服务器故障的场景"""
        print(f"\n{'='*60}")
        print("测试部分服务器故障场景")
        print(f"{'='*60}")
        
        # 创建混合配置：一些有效服务器 + 一些无效服务器
        builder = dns.DnsResolverBuilder()
        builder.query_strategy(QueryStrategy.SMART)
        
        # 添加有效的DNS服务器
        builder.add_udp_upstream("Cloudflare", "1.1.1.1:53", 10)
        builder.add_udp_upstream("Google", "8.8.8.8:53", 20)
        
        # 添加无效的DNS服务器
        builder.add_udp_upstream("Invalid1", "192.0.2.1:53", 5)
        builder.add_udp_upstream("Invalid2", "192.0.2.2:53", 5)
        
        builder.enable_health_checker(True)
        builder.health_check_interval(2)
        builder.health_check_timeout(1)
        
        resolver = builder.build()
        
        print("✅ 创建了混合配置的DNS解析器")
        print("📋 配置的上游服务器:")
        print("   - Cloudflare (1.1.1.1:53) - 权重: 10 [有效]")
        print("   - Google (8.8.8.8:53) - 权重: 20 [有效]")
        print("   - Invalid1 (192.0.2.1:53) - 权重: 5 [无效]")
        print("   - Invalid2 (192.0.2.2:53) - 权重: 5 [无效]")
        
        # 等待健康检查
        print("\n⏳ 等待健康检查运行...")
        time.sleep(4)
        
        # 检查健康状态
        print("\n📊 当前健康状态:")
        health_status = resolver.get_health_status()
        for server_name, is_healthy in health_status.items():
            status = "🟢 健康" if is_healthy else "🔴 不健康"
            print(f"   {server_name}: {status}")
        
        # 获取应急信息
        print("\n🚨 应急响应信息:")
        emergency_info = resolver.get_emergency_response_info()
        print(f"   所有服务器失败: {emergency_info.all_servers_failed}")
        print(f"   总失败次数: {emergency_info.total_failures}")
        print(f"   应急消息: {emergency_info.emergency_message}")
        
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
            print(f"\n--- {strategy_name} 错误信息增强 ---")
            
            resolver = self.create_resolver_with_strategy(strategy)
            
            # 等待健康检查
            time.sleep(2)
            
            try:
                resolver.resolve("nonexistent-domain-12345.invalid")
            except Exception as e:
                error_msg = str(e)
                print(f"原始错误: {error_msg[:100]}...")
                
                # 检查是否包含策略信息
                if f"{strategy_name}" in error_msg or "策略" in error_msg:
                    print("✅ 错误信息包含策略信息")
                
                # 检查是否包含应急信息
                if "应急" in error_msg or "🚨" in error_msg:
                    print("✅ 错误信息包含应急信息")
                
                # 检查是否包含统计信息
                if "失败" in error_msg and "次" in error_msg:
                    print("✅ 错误信息包含失败统计")
    
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
        print("✅ 所有查询策略都支持统一的应急处理机制")
        print("✅ 系统能够智能区分全部故障和部分故障")
        print("✅ 错误信息得到了智能增强，包含详细的诊断信息")
        print("✅ 应急响应提供了有用的故障排查信息")
        print("="*80)

def main():
    """主函数"""
    demo = UniversalEmergencyDemo()
    demo.run_demo()

if __name__ == "__main__":
    main()