#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
基本使用示例

展示了rat-quickdns-py的基本功能和用法。
"""

import time
import rat_quickdns_py as dns
from rat_quickdns_py import QueryStrategy


def basic_resolve_example():
    """基本域名解析示例"""
    print("\n=== 基本域名解析示例 ===")
    
    # 创建解析器（使用构建器模式替代便捷函数）
    print("\n使用构建器模式解析单个域名:")
    builder = dns.DnsResolverBuilder()
    builder.add_udp_upstream("Cloudflare", "1.1.1.1:53")
    builder.add_udp_upstream("Google", "8.8.8.8:53")
    builder.query_strategy(QueryStrategy.SMART)
    builder.timeout(5.0)
    resolver = builder.build()
    
    start_time = time.time()
    ips = resolver.resolve("example.com")
    elapsed = (time.time() - start_time) * 1000
    if ips:
        print(f"example.com -> {ips} (耗时: {elapsed:.2f}ms)")
    else:
        print(f"example.com -> 解析失败 (耗时: {elapsed:.2f}ms)")
    
    # 批量解析多个域名
    print("\n批量解析多个域名:")
    domains = ["google.com", "github.com", "cloudflare.com", "invalid-domain-example.xyz"]
    start_time = time.time()
    
    results = []
    for domain in domains:
        try:
            ips = resolver.resolve(domain)
            if ips:
                results.append(ips)
            else:
                results.append(None)
        except Exception as e:
            print(f"  解析 {domain} 时出错: {e}")
            results.append(None)
    
    elapsed = (time.time() - start_time) * 1000
    print(f"批量解析耗时: {elapsed:.2f}ms")
    for i, result in enumerate(results):
        if result and isinstance(result, list) and len(result) > 0:
            print(f"  ✓ {domains[i]} -> {result}")
        else:
            print(f"  ✗ {domains[i]} -> 解析失败或返回空结果")


def custom_resolver_example():
    """自定义解析器示例"""
    print("\n=== 自定义解析器示例 ===")
    
    # 创建解析器构建器
    builder = dns.DnsResolverBuilder()
    
    # 配置解析器
    builder.query_strategy(QueryStrategy.SMART)  # 使用智能决策策略
    builder.add_udp_upstream("Cloudflare", "1.1.1.1:53")  # 添加Cloudflare DNS (UDP)
    builder.add_udp_upstream("Google", "8.8.8.8:53")      # 添加Google DNS (UDP)
    builder.timeout(3.0)                                     # 设置3秒超时
    builder.enable_edns(True)                               # 启用EDNS
    builder.enable_health_checker(True)                     # 启用健康检查
    
    # 构建解析器
    resolver = builder.build()
    
    # 解析不同类型的DNS记录
    domain = "example.com"
    print(f"\n解析{domain}的不同记录类型:")
    
    # A记录 (IPv4)
    start_time = time.time()
    ipv4_addrs = resolver.resolve_a(domain)
    elapsed = (time.time() - start_time) * 1000
    print(f"  A记录: {ipv4_addrs} (耗时: {elapsed:.2f}ms)")
    
    # AAAA记录 (IPv6)
    start_time = time.time()
    ipv6_addrs = resolver.resolve_aaaa(domain)
    elapsed = (time.time() - start_time) * 1000
    print(f"  AAAA记录: {ipv6_addrs} (耗时: {elapsed:.2f}ms)")
    
    # 尝试解析MX记录
    try:
        start_time = time.time()
        mx_records = resolver.resolve_mx(domain)
        elapsed = (time.time() - start_time) * 1000
        print(f"  MX记录: {mx_records} (耗时: {elapsed:.2f}ms)")
    except Exception as e:
        print(f"  MX记录: 解析失败 - {e}")
    
    # 尝试解析TXT记录
    try:
        start_time = time.time()
        txt_records = resolver.resolve_txt(domain)
        elapsed = (time.time() - start_time) * 1000
        print(f"  TXT记录: {txt_records} (耗时: {elapsed:.2f}ms)")
    except Exception as e:
        print(f"  TXT记录: 解析失败 - {e}")


def query_strategy_comparison():
    """查询策略对比示例"""
    print("\n=== 查询策略对比 ===")
    
    # 测试域名
    domain = "google.com"
    print(f"\n解析域名: {domain}")
    
    # 创建不同策略的解析器
    strategies = [
        ("FIFO (先进先出)", QueryStrategy.FIFO),
        ("SMART (智能决策)", QueryStrategy.SMART),
        ("ROUND_ROBIN (轮询)", QueryStrategy.ROUND_ROBIN),
    ]
    
    # 添加多个上游服务器，包括一个故意延迟的服务器
    for name, strategy in strategies:
        builder = dns.DnsResolverBuilder()
        builder.query_strategy(strategy)
        builder.add_udp_upstream("Cloudflare", "1.1.1.1:53")
        builder.add_udp_upstream("Google", "8.8.8.8:53")
        builder.add_udp_upstream("Quad9", "9.9.9.9:53")
        builder.timeout(5.0)
        
        resolver = builder.build()
        
        # 测量解析时间
        start_time = time.time()
        try:
            ips = resolver.resolve(domain)
            elapsed = (time.time() - start_time) * 1000
            print(f"  {name}: 成功 - {ips} (耗时: {elapsed:.2f}ms)")
        except Exception as e:
            elapsed = (time.time() - start_time) * 1000
            print(f"  {name}: 失败 - {e} (耗时: {elapsed:.2f}ms)")


def preset_builders_example():
    """预设构建器示例"""
    print("\n=== 预设构建器示例 ===")
    
    presets = ["fast", "secure", "balanced"]
    domain = "cloudflare.com"
    
    for preset in presets:
        print(f"\n使用 '{preset}' 预设:")
        builder = dns.create_preset_builder(preset)
        resolver = builder.build()
        
        start_time = time.time()
        try:
            ips = resolver.resolve(domain)
            elapsed = (time.time() - start_time) * 1000
            print(f"  解析 {domain} -> {ips} (耗时: {elapsed:.2f}ms)")
        except Exception as e:
            elapsed = (time.time() - start_time) * 1000
            print(f"  解析 {domain} 失败: {e} (耗时: {elapsed:.2f}ms)")


def utility_functions_example():
    """工具函数示例"""
    print("\n=== 工具函数示例 ===")
    
    # IP地址验证
    print("\nIP地址验证:")
    ip_examples = ["192.168.1.1", "2001:db8::1", "invalid-ip", "256.256.256.256"]
    for ip in ip_examples:
        is_valid = dns.is_valid_ip(ip)
        is_ipv4 = dns.is_valid_ipv4(ip)
        is_ipv6 = dns.is_valid_ipv6(ip)
        print(f"  '{ip}': 有效IP={is_valid}, IPv4={is_ipv4}, IPv6={is_ipv6}")
    
    # 域名验证
    print("\n域名验证:")
    domain_examples = ["example.com", "sub.domain.co.uk", "invalid..domain", "domain-with-@.com"]
    for domain in domain_examples:
        is_valid = dns.is_valid_domain(domain)
        print(f"  '{domain}': 有效={is_valid}")
    
    # Socket地址解析
    print("\nSocket地址解析:")
    addr_examples = ["8.8.8.8:53", "[2001:db8::1]:853", "domain:80", "192.168.1.1"]
    for addr in addr_examples:
        is_valid = dns.is_valid_socket_addr(addr)
        parsed = dns.parse_socket_addr(addr)
        print(f"  '{addr}': 有效={is_valid}, 解析结果={parsed}")
    
    # 获取默认服务器列表
    print("\n默认DNS服务器:")
    servers = dns.get_default_dns_servers()
    for name, addr in servers[:3]:  # 只显示前3个
        print(f"  {name}: {addr}")
    print(f"  ... 共{len(servers)}个服务器")
    
    # 获取默认DoH服务器列表
    print("\n默认DoH服务器:")
    doh_servers = dns.get_default_doh_servers()
    for name, url in doh_servers:
        print(f"  {name}: {url}")


def main():
    """主函数"""
    print(f"rat-quickdns-py 版本: {dns.__version__}")

    # 运行示例
    basic_resolve_example()
    custom_resolver_example()
    query_strategy_comparison()
    preset_builders_example()
    utility_functions_example()


if __name__ == "__main__":
    main()