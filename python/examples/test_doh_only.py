#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
DoH (DNS over HTTPS) 专项测试

本脚本专门测试DoH协议的DNS解析功能，参考Rust版本的mx_record_test_doh.rs
主要用于排查DoH传输是否正常工作
"""

import time
import gc
import socket
import threading
from urllib.parse import urlparse
from typing import List, Dict, Any, Tuple
from concurrent.futures import ThreadPoolExecutor, as_completed

# Python绑定导入
try:
    import rat_quickdns_py as dns
    from rat_quickdns_py import QueryStrategy
except ImportError:
    print("请确保已正确安装 rat_quickdns_py Python 绑定")
    exit(1)


class DohOnlyTest:
    """DoH专项测试类"""
    
    def __init__(self):
        self.test_domains = [
            "google.com",
            "github.com", 
            "example.com",
            "qq.com",
            "163.com"
        ]
        
        # DoH服务器配置 - 只使用国内服务器
        self.doh_servers = [
            {
                "name": "腾讯DoH",
                "url": "https://doh.pub/dns-query", 
                "region": "国内",
                "description": "腾讯公共DNS DoH服务"
            },
            {
                "name": "阿里DoH",
                "url": "https://dns.alidns.com/dns-query",
                "region": "国内",
                "description": "阿里云公共DNS DoH服务"
            },
            {
                "name": "360DoH",
                "url": "https://doh.360.cn/dns-query",
                "region": "国内",
                "description": "360安全DNS DoH服务"
            },
            {
                "name": "百度DoH",
                "url": "https://doh.dns.baidu.com/dns-query",
                "region": "国内",
                "description": "百度公共DNS DoH服务"
            },
            {
                "name": "DNSPOD DoH",
                "url": "https://doh.pub/dns-query",
                "region": "国内",
                "description": "腾讯DNSPOD DoH服务"
            }
        ]
        
    def resolve_doh_server_ips(self, url: str) -> List[str]:
        """解析DoH服务器的IP地址"""
        try:
            parsed = urlparse(url)
            hostname = parsed.hostname
            if not hostname:
                return []
            addr_info = socket.getaddrinfo(hostname, 443, socket.AF_UNSPEC, socket.SOCK_STREAM)
            ips = list(set([addr[4][0] for addr in addr_info]))
            return ips
        except Exception as e:
            print(f"    ⚠️  解析 {url} 的IP失败: {e}")
            return []
    
    def test_tcp_connection(self, ip: str, port: int = 443, timeout: float = 3.0) -> Tuple[str, float]:
        """测试TCP连接速度"""
        try:
            start_time = time.time()
            sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            sock.settimeout(timeout)
            result = sock.connect_ex((ip, port))
            elapsed = (time.time() - start_time) * 1000
            sock.close()
            return ip, elapsed if result == 0 else float('inf')
        except Exception:
            return ip, float('inf')
    
    def precheck_doh_servers(self) -> List[Dict[str, Any]]:
        """预检测DoH服务器IP并按连接速度排序"""
        print("\n🔍 DoH服务器IP预检测 (加速连接)")
        print("  服务器 |           IP地址 |   连接耗时 | 状态")
        print("  ─────────────────────────────────────────────────────────")
        
        enhanced_servers = []
        
        for server in self.doh_servers:
            server_name = server['name']
            url = server['url']
            ips = self.resolve_doh_server_ips(url)
            
            if not ips:
                print(f"  {server_name:>8} | {'无法解析':>15} |      N/A | ❌")
                continue
            
            best_ip = None
            best_time = float('inf')
            
            with ThreadPoolExecutor(max_workers=min(len(ips), 5)) as executor:
                futures = {executor.submit(self.test_tcp_connection, ip): ip for ip in ips}
                
                for future in as_completed(futures):
                    ip, elapsed = future.result()
                    status = "✅" if elapsed < float('inf') else "❌"
                    display_time = elapsed if elapsed < float('inf') else 0
                    print(f"  {server_name:>8} | {ip:>15} | {display_time:>8.1f}ms | {status}")
                    
                    if elapsed < best_time:
                        best_time = elapsed
                        best_ip = ip
            
            enhanced_server = server.copy()
            if best_ip and best_time < float('inf'):
                enhanced_server['best_ip'] = best_ip
                enhanced_server['best_time'] = best_time
                enhanced_server['precheck_success'] = True
                print(f"  📍 {server_name} 最佳IP: {best_ip} ({best_time:.1f}ms)")
            else:
                enhanced_server['precheck_success'] = False
                print(f"  ⚠️  {server_name} 所有IP连接失败")
            
            enhanced_servers.append(enhanced_server)
        
        enhanced_servers.sort(key=lambda x: (not x.get('precheck_success', False), x.get('best_time', float('inf'))))
        print(f"\n📊 预检测完成，共 {len([s for s in enhanced_servers if s.get('precheck_success')])} 个服务器可用")
        return enhanced_servers
    
    def test_single_doh_server(self, server_config: Dict[str, str]) -> bool:
        """测试单个DoH服务器"""
        print(f"\n🔒 测试DoH服务器: {server_config['name']} ({server_config['region']})")
        print(f"   URL: {server_config['url']}")
        if server_config.get('best_ip'):
            print(f"   最佳IP: {server_config['best_ip']} ({server_config['best_time']:.1f}ms)")
        print(f"   描述: {server_config.get('description', '无描述')}")
        print("  状态 |           域名 |     耗时 | 结果")
        print("  ─────────────────────────────────────────────────────────")
        
        success_count = 0
        total_count = 0
        
        for domain in self.test_domains:
            total_count += 1
            
            try:
                # 创建只使用DoH的解析器
                builder = dns.DnsResolverBuilder()
                builder.query_strategy(QueryStrategy.FIFO)  # 使用FIFO策略
                builder.enable_edns(True)  # 启用EDNS
                builder.region("global")  # 设置全局区域
                
                # 只添加一个DoH上游服务器
                builder.add_doh_upstream(server_config['name'], server_config['url'])
                
                # 设置较短的超时时间，快速失败
                builder.timeout(10.0)  # 10秒超时
                
                # 构建解析器
                resolver = builder.build()
                
                # 执行DNS查询
                start_time = time.time()
                result = resolver.resolve_a(domain)
                elapsed = (time.time() - start_time) * 1000
                
                if result and len(result) > 0:
                    success_count += 1
                    print(f"  ✅ | {domain:>15} | {elapsed:>8.2f}ms | {len(result)} 个IP")
                    # 显示前2个IP地址
                    for i, ip in enumerate(result[:2]):
                        print(f"    📍 IP{i+1}: {ip}")
                    if len(result) > 2:
                        print(f"    📍 ... 还有{len(result)-2}个IP")
                else:
                    print(f"  ⚠️  | {domain:>15} | {elapsed:>8.2f}ms | 无结果")
                
                # 清理解析器
                del resolver
                gc.collect()
                time.sleep(0.1)  # 给Rust端时间清理资源
                
            except Exception as e:
                elapsed = (time.time() - start_time) * 1000 if 'start_time' in locals() else 0
                print(f"  ❌ | {domain:>15} | {elapsed:>8.2f}ms | 错误: {str(e)[:50]}")
        
        success_rate = (success_count / total_count) * 100.0 if total_count > 0 else 0
        print(f"  📊 {server_config['name']} 成功率: {success_rate:.1f}% ({success_count}/{total_count})")
        
        return success_count > 0
    
    def test_mixed_doh_udp(self):
        """测试DoH和UDP混合使用（仅国内服务器）"""
        print("\n🔀 测试DoH和UDP混合使用（仅国内服务器）")
        print("  状态 |           域名 |     耗时 | 协议 | 结果")
        print("  ─────────────────────────────────────────────────────────────────")
        
        try:
            # 创建混合协议解析器
            builder = dns.DnsResolverBuilder()
            builder.query_strategy(QueryStrategy.SMART)  # 使用智能策略
            builder.enable_edns(True)
            builder.region("global")
            
            # 添加国内UDP和DoH上游服务器
            builder.add_udp_upstream("腾讯UDP", "119.29.29.29:53")
            builder.add_doh_upstream("腾讯DoH", "https://doh.pub/dns-query")
            builder.add_udp_upstream("阿里UDP", "223.5.5.5:53")
            builder.add_doh_upstream("阿里DoH", "https://dns.alidns.com/dns-query")
            
            builder.enable_upstream_monitoring(True)  # 启用上游监控
            builder.timeout(8.0)
            
            resolver = builder.build()
            
            for domain in self.test_domains[:3]:  # 只测试前3个域名
                try:
                    start_time = time.time()
                    result = resolver.resolve_a(domain)
                    elapsed = (time.time() - start_time) * 1000
                    
                    if result and len(result) > 0:
                        print(f"  ✅ | {domain:>15} | {elapsed:>8.2f}ms | 混合 | {len(result)} 个IP")
                        # 显示第一个IP
                        print(f"    📍 首个IP: {result[0]}")
                    else:
                        print(f"  ⚠️  | {domain:>15} | {elapsed:>8.2f}ms | 混合 | 无结果")
                        
                except Exception as e:
                    elapsed = (time.time() - start_time) * 1000
                    print(f"  ❌ | {domain:>15} | {elapsed:>8.2f}ms | 混合 | 错误: {str(e)[:30]}")
            
            # 清理解析器
            del resolver
            gc.collect()
            time.sleep(0.1)
            
        except Exception as e:
            print(f"  ❌ 混合协议测试失败: {e}")
    
    def run_test(self):
        """运行完整的DoH测试"""
        print("🚀 DoH (DNS over HTTPS) 专项测试")
        print(f"测试 {len(self.doh_servers)} 个DoH服务器 × {len(self.test_domains)} 个域名")
        print("============================================================")
        
        # 预检测DoH服务器IP并排序
        enhanced_servers = self.precheck_doh_servers()
        
        working_servers = 0
        total_servers = len(enhanced_servers)
        
        # 测试每个DoH服务器（按预检测速度排序）
        for server_config in enhanced_servers:
            if self.test_single_doh_server(server_config):
                working_servers += 1
        
        # 测试混合协议
        self.test_mixed_doh_udp()
        
        # 总结
        print("\n📈 DoH测试总结:")
        print(f"  可用DoH服务器: {working_servers}/{total_servers}")
        print(f"  DoH服务器可用率: {(working_servers/total_servers)*100:.1f}%")
        
        if working_servers == 0:
            print("\n⚠️  所有DoH服务器都无法正常工作，可能的原因:")
            print("   1. 网络连接问题")
            print("   2. DoH功能未正确编译到Python绑定中")
            print("   3. TLS/SSL证书验证失败")
            print("   4. 防火墙阻止HTTPS DNS查询")
            print("   5. DoH传输模块未正确初始化")
        elif working_servers < total_servers:
            print("\n⚠️  部分DoH服务器无法工作，建议:")
            print("   1. 检查网络连接")
            print("   2. 尝试其他DoH服务器")
            print("   3. 检查防火墙设置")
        else:
            print("\n✅ 所有DoH服务器工作正常!")
        
        print("\n💡 DoH协议特点:")
        print("   ✅ 优势: 加密传输、隐私保护、穿越防火墙")
        print("   ⚠️  注意: 首次连接延迟较高、需要TLS握手")
        print("   🚀 优化: IP预检测加速连接建立")


def main():
    """主函数"""
    try:
        test = DohOnlyTest()
        test.run_test()
    except KeyboardInterrupt:
        print("\n程序被用户中断")
    except Exception as e:
        print(f"\n程序执行出错: {e}")
        import traceback
        traceback.print_exc()
    finally:
        print("\n=== 清理资源 ===")
        gc.collect()
        time.sleep(0.5)
        print("程序退出")


if __name__ == "__main__":
    main()