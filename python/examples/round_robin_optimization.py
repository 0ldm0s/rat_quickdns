#!/usr/bin/env python3
"""
ROUND_ROBIN策略性能优化示例

本示例展示如何使用优化后的ROUND_ROBIN策略进行高性能DNS查询。
包括：
1. 基础ROUND_ROBIN配置
2. 性能优化配置
3. 批量查询测试
4. 性能对比
"""

import time
import statistics
from typing import List, Dict, Any

# Python绑定导入
try:
    import rat_quickdns_py as dns
    from rat_quickdns_py import QueryStrategy
except ImportError:
    print("请确保已正确安装 rat_quickdns_py Python 绑定")
    exit(1)


class RoundRobinOptimizationDemo:
    """ROUND_ROBIN策略优化演示类"""
    
    def __init__(self):
        self.test_domains = [
            "google.com",
            "github.com",
            "stackoverflow.com",
            "microsoft.com",
            "amazon.com",
            "cloudflare.com",
            "baidu.com",
            "taobao.com",
            "qq.com",
            "weibo.com"
        ]
    
    def create_basic_resolver(self) -> 'DnsResolver':
        """创建基础ROUND_ROBIN解析器"""
        builder = dns.DnsResolverBuilder()
        
        # 设置ROUND_ROBIN策略
        builder.query_strategy(QueryStrategy.ROUND_ROBIN)
        
        # 添加多个上游服务器
        builder.add_udp_upstream("阿里DNS", "223.5.5.5")
        builder.add_udp_upstream("腾讯DNS", "119.29.29.29")
        builder.add_udp_upstream("114DNS", "114.114.114.114")
        builder.add_udp_upstream("Google DNS", "8.8.8.8")
        builder.add_udp_upstream("Cloudflare DNS", "1.1.1.1")
        
        # 基础配置
        builder.timeout(5.0)  # 默认5秒超时
        builder.enable_health_checker(True)
        
        return builder.build()
    
    def create_optimized_resolver(self) -> 'DnsResolver':
        """创建优化的ROUND_ROBIN解析器"""
        builder = dns.DnsResolverBuilder()
        
        # 设置ROUND_ROBIN策略
        builder.query_strategy(QueryStrategy.ROUND_ROBIN)
        
        # 添加多个上游服务器
        builder.add_udp_upstream("阿里DNS", "223.5.5.5")
        builder.add_udp_upstream("腾讯DNS", "119.29.29.29")
        builder.add_udp_upstream("114DNS", "114.114.114.114")
        builder.add_udp_upstream("Google DNS", "8.8.8.8")
        builder.add_udp_upstream("Cloudflare DNS", "1.1.1.1")
        
        # 优化配置（使用可用的API）
        builder.timeout(2.0)  # 更短的超时时间
        builder.enable_health_checker(True)
        
        # 注意：以下优化方法可能在当前版本中不可用
        # builder.optimize_for_round_robin()
        # builder.round_robin_timeout(1.5)
        # builder.retries(1)
        # builder.concurrent_queries(4)
        
        return builder.build()
    
    def benchmark_resolver(self, resolver: 'DnsResolver', name: str, iterations: int = 50) -> Dict[str, Any]:
        """对解析器进行性能测试"""
        print(f"\n🚀 开始测试 {name} (共{iterations}次查询)...")
        
        latencies = []
        success_count = 0
        failure_count = 0
        
        start_time = time.time()
        
        for i in range(iterations):
            domain = self.test_domains[i % len(self.test_domains)]
            
            try:
                query_start = time.time()
                result = resolver.resolve_a(domain)
                query_end = time.time()
                
                # 检查结果是否为有效的IP地址列表
                if result and isinstance(result, list) and len(result) > 0:
                    success_count += 1
                    latency_ms = (query_end - query_start) * 1000
                    latencies.append(latency_ms)
                    
                    if i % 10 == 0:
                        print(f"  ✅ {domain}: {result[0]} ({latency_ms:.1f}ms)")
                else:
                    failure_count += 1
                    print(f"  ❌ {domain}: 解析失败或返回空结果")
                    
            except Exception as e:
                failure_count += 1
                print(f"  💥 {domain}: {str(e)}")
        
        total_time = time.time() - start_time
        
        # 计算统计信息
        stats = {
            "name": name,
            "total_queries": iterations,
            "success_count": success_count,
            "failure_count": failure_count,
            "success_rate": success_count / iterations * 100,
            "total_time": total_time,
            "qps": iterations / total_time,
            "avg_latency": statistics.mean(latencies) if latencies else 0,
            "min_latency": min(latencies) if latencies else 0,
            "max_latency": max(latencies) if latencies else 0,
            "p95_latency": statistics.quantiles(latencies, n=20)[18] if len(latencies) > 20 else 0,
        }
        
        return stats
    
    def print_comparison(self, basic_stats: Dict[str, Any], optimized_stats: Dict[str, Any]):
        """打印性能对比结果"""
        print("\n" + "="*80)
        print("📊 ROUND_ROBIN策略性能对比报告")
        print("="*80)
        
        print(f"\n🔍 基础配置 vs 优化配置:")
        print(f"{'指标':<20} {'基础配置':<15} {'优化配置':<15} {'改进':<15}")
        print("-" * 70)
        
        # 成功率对比
        success_improvement = optimized_stats['success_rate'] - basic_stats['success_rate']
        print(f"{'成功率':<20} {basic_stats['success_rate']:<14.1f}% {optimized_stats['success_rate']:<14.1f}% {success_improvement:+.1f}%")
        
        # QPS对比
        qps_improvement = (optimized_stats['qps'] - basic_stats['qps']) / basic_stats['qps'] * 100
        print(f"{'QPS':<20} {basic_stats['qps']:<14.1f} {optimized_stats['qps']:<14.1f} {qps_improvement:+.1f}%")
        
        # 平均延迟对比
        latency_improvement = (basic_stats['avg_latency'] - optimized_stats['avg_latency']) / basic_stats['avg_latency'] * 100
        print(f"{'平均延迟(ms)':<20} {basic_stats['avg_latency']:<14.1f} {optimized_stats['avg_latency']:<14.1f} {latency_improvement:+.1f}%")
        
        # P95延迟对比
        p95_improvement = (basic_stats['p95_latency'] - optimized_stats['p95_latency']) / basic_stats['p95_latency'] * 100 if basic_stats['p95_latency'] > 0 else 0
        print(f"{'P95延迟(ms)':<20} {basic_stats['p95_latency']:<14.1f} {optimized_stats['p95_latency']:<14.1f} {p95_improvement:+.1f}%")
        
        print("\n💡 优化效果总结:")
        if qps_improvement > 0:
            print(f"  ✅ QPS提升 {qps_improvement:.1f}%")
        if latency_improvement > 0:
            print(f"  ✅ 平均延迟降低 {latency_improvement:.1f}%")
        if success_improvement > 0:
            print(f"  ✅ 成功率提升 {success_improvement:.1f}%")
        
        print("\n🎯 优化建议:")
        print("  1. 使用 optimize_for_round_robin() 应用所有优化")
        print("  2. 根据网络环境调整 round_robin_timeout()")
        print("  3. 启用健康检查避免选择不可用服务器")
        print("  4. 增加并发查询数量提高吞吐量")
    
    def run_demo(self):
        """运行完整的演示"""
        print("🔧 ROUND_ROBIN策略性能优化演示")
        print("=" * 50)
        
        # 创建解析器
        print("\n📦 创建解析器实例...")
        basic_resolver = self.create_basic_resolver()
        optimized_resolver = self.create_optimized_resolver()
        
        # 性能测试
        iterations = 100
        basic_stats = self.benchmark_resolver(basic_resolver, "基础ROUND_ROBIN", iterations)
        optimized_stats = self.benchmark_resolver(optimized_resolver, "优化ROUND_ROBIN", iterations)
        
        # 打印对比结果
        self.print_comparison(basic_stats, optimized_stats)
        
        print("\n✨ 演示完成！")


def main():
    """主函数"""
    demo = RoundRobinOptimizationDemo()
    
    try:
        # 运行演示
        demo.run_demo()
    except KeyboardInterrupt:
        print("\n⏹️  演示被用户中断")
    except Exception as e:
        print(f"\n❌ 演示过程中发生错误: {e}")


if __name__ == "__main__":
    main()