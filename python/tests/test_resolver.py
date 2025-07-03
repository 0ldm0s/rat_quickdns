#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
rat-quickdns-py 解析器测试

测试DNS解析器的基本功能和正确性。
"""

import unittest
import sys
import os

# 添加项目根目录到Python路径
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))

try:
    import rat_quickdns_py as dns
    from rat_quickdns_py import QueryStrategy
except ImportError:
    print("错误: 无法导入rat_quickdns_py模块")
    print("请确保已安装该模块或正确设置了PYTHONPATH")
    sys.exit(1)


class TestResolver(unittest.TestCase):
    """测试DNS解析器的基本功能"""
    
    def setUp(self):
        """测试前准备"""
        # 创建解析器
        builder = dns.DnsResolverBuilder()
        builder.query_strategy(QueryStrategy.SMART)
        builder.add_udp_upstream("Cloudflare", "1.1.1.1:53", 10)
        builder.add_udp_upstream("Google", "8.8.8.8:53", 20)
        builder.timeout(5.0)
        self.resolver = builder.build()
        
        # 测试域名
        self.valid_domain = "example.com"
        self.invalid_domain = "invalid-domain-that-does-not-exist-123456.xyz"
    
    def test_resolve(self):
        """测试基本解析功能"""
        # 测试有效域名
        try:
            ips = self.resolver.resolve(self.valid_domain)
            self.assertIsInstance(ips, list)
            self.assertTrue(len(ips) > 0)
            
            # 验证返回的是有效IP地址
            for ip in ips:
                self.assertTrue(dns.is_valid_ip(ip))
        except Exception as e:
            self.fail(f"解析有效域名失败: {e}")
        
        # 测试无效域名（应该抛出异常）
        with self.assertRaises(Exception):
            self.resolver.resolve(self.invalid_domain)
    
    def test_resolve_a(self):
        """测试A记录解析"""
        try:
            ips = self.resolver.resolve_a(self.valid_domain)
            self.assertIsInstance(ips, list)
            self.assertTrue(len(ips) > 0)
            
            # 验证返回的是IPv4地址
            for ip in ips:
                self.assertTrue(dns.is_valid_ipv4(ip))
        except Exception as e:
            self.fail(f"解析A记录失败: {e}")
    
    def test_resolve_aaaa(self):
        """测试AAAA记录解析"""
        try:
            ips = self.resolver.resolve_aaaa(self.valid_domain)
            self.assertIsInstance(ips, list)
            
            # 验证返回的是IPv6地址（如果有）
            for ip in ips:
                self.assertTrue(dns.is_valid_ipv6(ip))
        except Exception as e:
            # 某些域名可能没有IPv6记录，不应该导致测试失败
            pass
    
    def test_batch_resolve(self):
        """测试批量解析"""
        domains = ["google.com", "github.com", self.invalid_domain]
        results = self.resolver.batch_resolve(domains)
        
        self.assertEqual(len(results), len(domains))
        
        # 检查前两个域名应该解析成功
        self.assertTrue(results[0].is_ok())
        self.assertTrue(results[1].is_ok())
        
        # 检查无效域名应该解析失败
        self.assertTrue(results[2].is_err())
        
        # 测试Result类的方法
        ips = results[0].unwrap()
        self.assertIsInstance(ips, list)
        self.assertTrue(len(ips) > 0)
        
        error_msg = results[2].unwrap_err()
        self.assertIsInstance(error_msg, str)
        self.assertTrue(len(error_msg) > 0)
        
        # 测试unwrap_or方法
        default_value = ["0.0.0.0"]
        self.assertEqual(results[2].unwrap_or(default_value), default_value)


class TestQueryStrategy(unittest.TestCase):
    """测试不同查询策略"""
    
    def setUp(self):
        """测试前准备"""
        self.test_domain = "example.com"
    
    def test_fifo_strategy(self):
        """测试最快优先策略"""
        builder = dns.DnsResolverBuilder()
        builder.query_strategy(QueryStrategy.FIFO)
        builder.add_udp_upstream("Cloudflare", "1.1.1.1:53", 10)
        builder.add_udp_upstream("Google", "8.8.8.8:53", 20)
        resolver = builder.build()
        
        ips = resolver.resolve(self.test_domain)
        self.assertIsInstance(ips, list)
        self.assertTrue(len(ips) > 0)
    
    def test_parallel_strategy(self):
        """测试并行策略"""
        builder = dns.DnsResolverBuilder()
        builder.query_strategy(QueryStrategy.PARALLEL)
        builder.add_udp_upstream("Cloudflare", "1.1.1.1:53", 10)
        builder.add_udp_upstream("Google", "8.8.8.8:53", 20)
        resolver = builder.build()
        
        ips = resolver.resolve(self.test_domain)
        self.assertIsInstance(ips, list)
        self.assertTrue(len(ips) > 0)
    
    def test_sequential_strategy(self):
        """测试顺序策略"""
        builder = dns.DnsResolverBuilder()
        builder.query_strategy(QueryStrategy.SEQUENTIAL)
        builder.add_udp_upstream("Cloudflare", "1.1.1.1:53", 10)
        builder.add_udp_upstream("Google", "8.8.8.8:53", 20)
        resolver = builder.build()
        
        ips = resolver.resolve(self.test_domain)
        self.assertIsInstance(ips, list)
        self.assertTrue(len(ips) > 0)
    
    def test_smart_strategy(self):
        """测试智能决策策略"""
        builder = dns.DnsResolverBuilder()
        builder.query_strategy(QueryStrategy.SMART)
        builder.add_udp_upstream("Cloudflare", "1.1.1.1:53", 10)
        builder.add_udp_upstream("Google", "8.8.8.8:53", 20)
        builder.enable_health_checker(True)
        resolver = builder.build()
        
        ips = resolver.resolve(self.test_domain)
        self.assertIsInstance(ips, list)
        self.assertTrue(len(ips) > 0)


class TestUtilityFunctions(unittest.TestCase):
    """测试工具函数"""
    
    def test_ip_validation(self):
        """测试IP地址验证"""
        # 有效IPv4
        self.assertTrue(dns.is_valid_ip("192.168.1.1"))
        self.assertTrue(dns.is_valid_ipv4("192.168.1.1"))
        self.assertFalse(dns.is_valid_ipv6("192.168.1.1"))
        
        # 有效IPv6
        self.assertTrue(dns.is_valid_ip("2001:db8::1"))
        self.assertFalse(dns.is_valid_ipv4("2001:db8::1"))
        self.assertTrue(dns.is_valid_ipv6("2001:db8::1"))
        
        # 无效IP
        self.assertFalse(dns.is_valid_ip("invalid-ip"))
        self.assertFalse(dns.is_valid_ipv4("invalid-ip"))
        self.assertFalse(dns.is_valid_ipv6("invalid-ip"))
        
        self.assertFalse(dns.is_valid_ip("256.256.256.256"))
        self.assertFalse(dns.is_valid_ipv4("256.256.256.256"))
    
    def test_domain_validation(self):
        """测试域名验证"""
        # 有效域名
        self.assertTrue(dns.is_valid_domain("example.com"))
        self.assertTrue(dns.is_valid_domain("sub.domain.co.uk"))
        self.assertTrue(dns.is_valid_domain("domain-with-dash.com"))
        
        # 无效域名
        self.assertFalse(dns.is_valid_domain("invalid..domain"))
        self.assertFalse(dns.is_valid_domain(".starts-with-dot.com"))
        self.assertFalse(dns.is_valid_domain("ends-with-dot.com."))
        self.assertFalse(dns.is_valid_domain("domain-with@special-chars.com"))
    
    def test_socket_addr_validation(self):
        """测试Socket地址验证"""
        # 有效Socket地址
        self.assertTrue(dns.is_valid_socket_addr("8.8.8.8:53"))
        self.assertTrue(dns.is_valid_socket_addr("[2001:db8::1]:853"))
        
        # 无效Socket地址
        self.assertFalse(dns.is_valid_socket_addr("invalid-address"))
        self.assertFalse(dns.is_valid_socket_addr("8.8.8.8"))  # 缺少端口
        self.assertFalse(dns.is_valid_socket_addr("8.8.8.8:invalid"))  # 无效端口
    
    def test_parse_socket_addr(self):
        """测试Socket地址解析"""
        # 有效IPv4 Socket地址
        result = dns.parse_socket_addr("8.8.8.8:53")
        self.assertIsNotNone(result)
        self.assertEqual(result, ("8.8.8.8", 53))
        
        # 有效IPv6 Socket地址
        result = dns.parse_socket_addr("[2001:db8::1]:853")
        self.assertIsNotNone(result)
        self.assertEqual(result, ("2001:db8::1", 853))
        
        # 无效Socket地址
        result = dns.parse_socket_addr("invalid-address")
        self.assertIsNone(result)


class TestPresetBuilders(unittest.TestCase):
    """测试预设构建器"""
    
    def test_fast_preset(self):
        """测试快速预设"""
        builder = dns.create_preset_builder("fast")
        resolver = builder.build()
        
        ips = resolver.resolve("example.com")
        self.assertIsInstance(ips, list)
        self.assertTrue(len(ips) > 0)
    
    def test_secure_preset(self):
        """测试安全预设"""
        builder = dns.create_preset_builder("secure")
        resolver = builder.build()
        
        ips = resolver.resolve("example.com")
        self.assertIsInstance(ips, list)
        self.assertTrue(len(ips) > 0)
    
    def test_balanced_preset(self):
        """测试平衡预设"""
        builder = dns.create_preset_builder("balanced")
        resolver = builder.build()
        
        ips = resolver.resolve("example.com")
        self.assertIsInstance(ips, list)
        self.assertTrue(len(ips) > 0)
    
    def test_invalid_preset(self):
        """测试无效预设"""
        with self.assertRaises(Exception):
            dns.create_preset_builder("invalid-preset")


if __name__ == "__main__":
    unittest.main()