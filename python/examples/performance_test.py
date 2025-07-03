#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
性能测试示例

测试rat-quickdns-py的性能，并与标准库的dns.resolver进行对比。
"""

import time
import random
import statistics
import concurrent.futures
from typing import List, Tuple, Dict, Any

import rat_quickdns_py as quickdns
from rat_quickdns_py import QueryStrategy

# 尝试导入dnspython，如果不可用则跳过对比测试
try:
    import dns.resolver
    DNSPYTHON_AVAILABLE = True
except ImportError:
    DNSPYTHON_AVAILABLE = False
    print("警告: dnspython未安装，将跳过对比测试")
    print("可以通过运行 'pip install dnspython' 安装")


# 测试域名列表
TEST_DOMAINS = [
    "google.com",
    "github.com",
    "cloudflare.com",
    "microsoft.com",
    "amazon.com",
    "apple.com",
    "netflix.com",
    "facebook.com",
    "twitter.com",
    "instagram.com",
    "wikipedia.org",
    "reddit.com",
    "linkedin.com",
    "yahoo.com",
    "twitch.tv",
    "spotify.com",
    "adobe.com",
    "cnn.com",
    "bbc.com",
    "nytimes.com",
]


def format_time(ms: float) -> str:
    """格式化时间显示"""
    if ms < 1:
        return f"{ms * 1000:.2f}μs"
    elif ms < 1000:
        return f"{ms:.2f}ms"
    else:
        return f"{ms / 1000:.2f}s"


def test_quickdns_single(domains: List[str], iterations: int = 1) -> Dict[str, Any]:
    """测试rat-quickdns-py单个域名解析性能"""
    print("\n测试 rat-quickdns-py 单个域名解析性能...")
    
    # 创建解析器
    builder = quickdns.DnsResolverBuilder()
    builder.query_strategy(QueryStrategy.FIFO)  # 使用最快优先策略
    builder.add_udp_upstream("Cloudflare", "1.1.1.1:53", 10)
    builder.add_udp_upstream("Google", "8.8.8.8:53", 20)
    builder.timeout(5.0)
    resolver = builder.build()
    
    results = []
    success_count = 0
    
    for _ in range(iterations):
        for domain in domains:
            start_time = time.time()
            try:
                ips = resolver.resolve(domain)
                elapsed_ms = (time.time() - start_time) * 1000
                results.append(elapsed_ms)
                success_count += 1
            except Exception as e:
                elapsed_ms = (time.time() - start_time) * 1000
                print(f"  解析失败 {domain}: {e} (耗时: {elapsed_ms:.2f}ms)")
    
    # 计算统计数据
    if results:
        avg_time = statistics.mean(results)
        min_time = min(results)
        max_time = max(results)
        median_time = statistics.median(results)
        p95_time = sorted(results)[int(len(results) * 0.95)]
        success_rate = (success_count / (len(domains) * iterations)) * 100
    else:
        avg_time = min_time = max_time = median_time = p95_time = 0
        success_rate = 0
    
    stats = {
        "avg_time": avg_time,
        "min_time": min_time,
        "max_time": max_time,
        "median_time": median_time,
        "p95_time": p95_time,
        "success_rate": success_rate,
        "total_queries": len(domains) * iterations,
        "successful_queries": success_count,
    }
    
    print(f"  平均解析时间: {format_time(avg_time)}")
    print(f"  最小解析时间: {format_time(min_time)}")
    print(f"  最大解析时间: {format_time(max_time)}")
    print(f"  中位数解析时间: {format_time(median_time)}")
    print(f"  95百分位解析时间: {format_time(p95_time)}")
    print(f"  成功率: {success_rate:.2f}%")
    
    return stats


def test_quickdns_batch(domains: List[str], iterations: int = 1) -> Dict[str, Any]:
    """测试rat-quickdns-py批量域名解析性能"""
    print("\n测试 rat-quickdns-py 批量域名解析性能...")
    
    # 创建解析器
    builder = quickdns.DnsResolverBuilder()
    builder.query_strategy(QueryStrategy.FIFO)  # 使用最快优先策略
    builder.add_udp_upstream("Cloudflare", "1.1.1.1:53", 10)
    builder.add_udp_upstream("Google", "8.8.8.8:53", 20)
    builder.timeout(5.0)
    resolver = builder.build()
    
    batch_times = []
    success_count = 0
    
    for _ in range(iterations):
        start_time = time.time()
        results = resolver.batch_resolve(domains)
        elapsed_ms = (time.time() - start_time) * 1000
        batch_times.append(elapsed_ms)
        
        # 统计成功数量
        for result in results:
            if result.is_ok():
                success_count += 1
    
    # 计算统计数据
    if batch_times:
        avg_time = statistics.mean(batch_times)
        min_time = min(batch_times)
        max_time = max(batch_times)
        success_rate = (success_count / (len(domains) * iterations)) * 100
        avg_per_domain = avg_time / len(domains)
    else:
        avg_time = min_time = max_time = avg_per_domain = 0
        success_rate = 0
    
    stats = {
        "avg_batch_time": avg_time,
        "min_batch_time": min_time,
        "max_batch_time": max_time,
        "avg_per_domain": avg_per_domain,
        "success_rate": success_rate,
        "total_queries": len(domains) * iterations,
        "successful_queries": success_count,
    }
    
    print(f"  平均批量解析时间: {format_time(avg_time)}")
    print(f"  最小批量解析时间: {format_time(min_time)}")
    print(f"  最大批量解析时间: {format_time(max_time)}")
    print(f"  平均每个域名时间: {format_time(avg_per_domain)}")
    print(f"  成功率: {success_rate:.2f}%")
    
    return stats


def test_dnspython(domains: List[str], iterations: int = 1) -> Dict[str, Any]:
    """测试dnspython性能"""
    if not DNSPYTHON_AVAILABLE:
        return {"error": "dnspython未安装"}
    
    print("\n测试 dnspython 性能...")
    
    # 创建解析器
    resolver = dns.resolver.Resolver()
    resolver.nameservers = ["1.1.1.1", "8.8.8.8"]
    resolver.timeout = 5.0
    
    results = []
    success_count = 0
    
    for _ in range(iterations):
        for domain in domains:
            start_time = time.time()
            try:
                answers = resolver.resolve(domain, "A")
                elapsed_ms = (time.time() - start_time) * 1000
                results.append(elapsed_ms)
                success_count += 1
            except Exception as e:
                elapsed_ms = (time.time() - start_time) * 1000
                print(f"  解析失败 {domain}: {e} (耗时: {elapsed_ms:.2f}ms)")
    
    # 计算统计数据
    if results:
        avg_time = statistics.mean(results)
        min_time = min(results)
        max_time = max(results)
        median_time = statistics.median(results)
        p95_time = sorted(results)[int(len(results) * 0.95)]
        success_rate = (success_count / (len(domains) * iterations)) * 100
    else:
        avg_time = min_time = max_time = median_time = p95_time = 0
        success_rate = 0
    
    stats = {
        "avg_time": avg_time,
        "min_time": min_time,
        "max_time": max_time,
        "median_time": median_time,
        "p95_time": p95_time,
        "success_rate": success_rate,
        "total_queries": len(domains) * iterations,
        "successful_queries": success_count,
    }
    
    print(f"  平均解析时间: {format_time(avg_time)}")
    print(f"  最小解析时间: {format_time(min_time)}")
    print(f"  最大解析时间: {format_time(max_time)}")
    print(f"  中位数解析时间: {format_time(median_time)}")
    print(f"  95百分位解析时间: {format_time(p95_time)}")
    print(f"  成功率: {success_rate:.2f}%")
    
    return stats


def test_concurrent_performance(domains: List[str], workers: int, iterations: int = 1) -> Dict[str, Any]:
    """测试并发性能"""
    print(f"\n测试 rat-quickdns-py 并发性能 (工作线程: {workers})...")
    
    # 创建解析器
    builder = quickdns.DnsResolverBuilder()
    builder.query_strategy(QueryStrategy.FIFO)
    builder.add_udp_upstream("Cloudflare", "1.1.1.1:53", 10)
    builder.add_udp_upstream("Google", "8.8.8.8:53", 20)
    builder.timeout(5.0)
    resolver = builder.build()
    
    # 准备任务列表
    tasks = []
    for _ in range(iterations):
        tasks.extend(domains)
    
    # 随机打乱任务顺序
    random.shuffle(tasks)
    
    results = []
    success_count = 0
    
    start_time = time.time()
    
    # 使用线程池并发执行
    with concurrent.futures.ThreadPoolExecutor(max_workers=workers) as executor:
        future_to_domain = {executor.submit(resolver.resolve, domain): domain for domain in tasks}
        
        for future in concurrent.futures.as_completed(future_to_domain):
            domain = future_to_domain[future]
            try:
                ips = future.result()
                success_count += 1
            except Exception as e:
                pass
    
    total_time = (time.time() - start_time) * 1000
    queries_per_second = len(tasks) / (total_time / 1000)
    success_rate = (success_count / len(tasks)) * 100
    
    stats = {
        "total_time": total_time,
        "queries_per_second": queries_per_second,
        "success_rate": success_rate,
        "total_queries": len(tasks),
        "successful_queries": success_count,
        "workers": workers,
    }
    
    print(f"  总时间: {format_time(total_time)}")
    print(f"  每秒查询数: {queries_per_second:.2f} qps")
    print(f"  成功率: {success_rate:.2f}%")
    
    return stats


def test_strategy_performance(domains: List[str]) -> Dict[str, List[Dict[str, Any]]]:
    """测试不同查询策略的性能"""
    print("\n测试不同查询策略的性能...")
    
    strategies = [
        ("FIFO (最快优先)", QueryStrategy.FIFO),
        ("PARALLEL (并行)", QueryStrategy.PARALLEL),
        ("SEQUENTIAL (顺序)", QueryStrategy.SEQUENTIAL),
        ("SMART (智能决策)", QueryStrategy.SMART),
    ]
    
    results = []
    
    for name, strategy in strategies:
        print(f"\n测试 {name} 策略...")
        
        builder = quickdns.DnsResolverBuilder()
        builder.query_strategy(strategy)
        builder.add_udp_upstream("Cloudflare", "1.1.1.1:53", 10)
        builder.add_udp_upstream("Google", "8.8.8.8:53", 20)
        builder.add_udp_upstream("Quad9", "9.9.9.9:53", 30)
        builder.timeout(5.0)
        resolver = builder.build()
        
        times = []
        success_count = 0
        
        for domain in domains:
            start_time = time.time()
            try:
                ips = resolver.resolve(domain)
                elapsed_ms = (time.time() - start_time) * 1000
                times.append(elapsed_ms)
                success_count += 1
            except Exception as e:
                elapsed_ms = (time.time() - start_time) * 1000
                print(f"  解析失败 {domain}: {e} (耗时: {elapsed_ms:.2f}ms)")
        
        # 计算统计数据
        if times:
            avg_time = statistics.mean(times)
            min_time = min(times)
            max_time = max(times)
            median_time = statistics.median(times)
            success_rate = (success_count / len(domains)) * 100
        else:
            avg_time = min_time = max_time = median_time = 0
            success_rate = 0
        
        stats = {
            "strategy": name,
            "avg_time": avg_time,
            "min_time": min_time,
            "max_time": max_time,
            "median_time": median_time,
            "success_rate": success_rate,
        }
        
        print(f"  平均解析时间: {format_time(avg_time)}")
        print(f"  最小解析时间: {format_time(min_time)}")
        print(f"  最大解析时间: {format_time(max_time)}")
        print(f"  中位数解析时间: {format_time(median_time)}")
        print(f"  成功率: {success_rate:.2f}%")
        
        results.append(stats)
    
    return {"strategy_comparison": results}


def print_comparison(quickdns_stats: Dict[str, Any], dnspython_stats: Dict[str, Any]) -> None:
    """打印性能对比结果"""
    if "error" in dnspython_stats:
        print("\n无法进行对比: dnspython未安装")
        return
    
    print("\n=== rat-quickdns-py vs dnspython 性能对比 ===")
    
    # 计算性能提升百分比
    if dnspython_stats["avg_time"] > 0:
        speedup = ((dnspython_stats["avg_time"] - quickdns_stats["avg_time"]) / dnspython_stats["avg_time"]) * 100
    else:
        speedup = 0
    
    print(f"平均解析时间:")
    print(f"  rat-quickdns-py: {format_time(quickdns_stats['avg_time'])}")
    print(f"  dnspython: {format_time(dnspython_stats['avg_time'])}")
    print(f"  性能提升: {speedup:.2f}%")
    
    print(f"\n中位数解析时间:")
    print(f"  rat-quickdns-py: {format_time(quickdns_stats['median_time'])}")
    print(f"  dnspython: {format_time(dnspython_stats['median_time'])}")
    
    print(f"\n95百分位解析时间:")
    print(f"  rat-quickdns-py: {format_time(quickdns_stats['p95_time'])}")
    print(f"  dnspython: {format_time(dnspython_stats['p95_time'])}")
    
    print(f"\n成功率:")
    print(f"  rat-quickdns-py: {quickdns_stats['success_rate']:.2f}%")
    print(f"  dnspython: {dnspython_stats['success_rate']:.2f}%")


def main() -> None:
    """主函数"""
    print(f"rat-quickdns-py 性能测试")
    print(f"版本: {quickdns.__version__}")
    print(f"测试域名数量: {len(TEST_DOMAINS)}")
    
    # 单个域名解析性能测试
    quickdns_stats = test_quickdns_single(TEST_DOMAINS)
    
    # 批量域名解析性能测试
    batch_stats = test_quickdns_batch(TEST_DOMAINS)
    
    # dnspython性能测试（如果可用）
    if DNSPYTHON_AVAILABLE:
        dnspython_stats = test_dnspython(TEST_DOMAINS)
        print_comparison(quickdns_stats, dnspython_stats)
    
    # 并发性能测试
    workers_list = [2, 4, 8, 16]
    for workers in workers_list:
        concurrent_stats = test_concurrent_performance(TEST_DOMAINS, workers)
    
    # 查询策略性能对比
    strategy_stats = test_strategy_performance(TEST_DOMAINS)


if __name__ == "__main__":
    main()