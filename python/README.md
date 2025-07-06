# rat-quickdns-py

Python bindings for rat_quickdns - A high-performance DNS client library written in Rust.

## Features

- **高性能**: 基于Rust的异步DNS解析，支持多种传输协议
- **智能决策**: 自动选择最优DNS服务器，支持健康检查
- **多协议支持**: UDP、TCP、DoH (DNS over HTTPS)、DoT (DNS over TLS)
- **连接优化**: IP预检测技术，自动选择最快连接路径，显著降低DoH/DoT首次连接延迟
- **负载均衡**: FIFO、并行、顺序、智能决策等多种查询策略
- **EDNS支持**: 支持客户端子网、UDP负载大小等扩展功能
- **批量查询**: 高效的并发批量域名解析

## Installation

```bash
pip install rat-quickdns-py
```

## Quick Start

```python
import rat_quickdns_py as dns
from rat_quickdns_py import QueryStrategy

# 基础UDP解析示例
builder = dns.DnsResolverBuilder()
builder.query_strategy(QueryStrategy.SMART)
builder.enable_edns(True)
builder.add_udp_upstream("Google DNS", "8.8.8.8:53")
builder.add_udp_upstream("Cloudflare DNS", "1.1.1.1:53")
resolver = builder.build()

# 解析单个域名
ips = resolver.resolve_a("example.com")
print(f"example.com resolves to: {ips}")

# DoH预检测优化示例（推荐用于生产环境）
builder = dns.DnsResolverBuilder()
builder.query_strategy(QueryStrategy.SMART)
builder.enable_edns(True)
builder.add_doh_upstream("腾讯DoH", "https://doh.pub/dns-query")
builder.add_doh_upstream("阿里DoH", "https://dns.alidns.com/dns-query")
builder.enable_upstream_monitoring(True)  # 启用上游监控
builder.timeout(8.0)
resolver = builder.build()

# 批量解析
domains = ["google.com", "github.com", "microsoft.com"]
for domain in domains:
    try:
        result = resolver.resolve_a(domain)
        print(f"{domain}: {result}")
    except Exception as e:
        print(f"{domain}: Error - {e}")
```

## API Reference

### DnsResolver

主要的DNS解析器类，提供异步DNS查询功能。

#### Methods

- `builder()` -> `DnsResolverBuilder`: 创建构建器
- `resolve(domain: str)` -> `List[str]`: 解析单个域名
- `batch_query(domains: List[str])` -> `List[Result[List[str], str]]`: 批量解析域名
- `start_health_check()`: 启动健康检查（智能模式）

### DnsResolverBuilder

DNS解析器构建器，用于配置解析器参数。

#### Methods

- `query_strategy(strategy: QueryStrategy)` -> `Self`: 设置查询策略
- `enable_edns(enable: bool)` -> `Self`: 启用/禁用EDNS
- `add_udp_upstream(name: str, address: str, weight: int)` -> `Self`: 添加UDP上游服务器
- `add_tcp_upstream(name: str, address: str, weight: int)` -> `Self`: 添加TCP上游服务器
- `add_doh_upstream(name: str, url: str, weight: int)` -> `Self`: 添加DoH上游服务器
- `add_dot_upstream(name: str, address: str, weight: int)` -> `Self`: 添加DoT上游服务器
- `build()` -> `DnsResolver`: 构建解析器实例

### QueryStrategy

查询策略枚举：

- `FIFO`: 最快优先（支持早期取消）
- `PARALLEL`: 并行查询所有服务器
- `SEQUENTIAL`: 顺序查询服务器
- `SMART`: 智能决策（推荐）

## Examples

### 专项测试工具

- **`examples/test_doh_only.py`**: DoH专项测试，包含IP预检测功能
  - 自动解析DoH服务器的所有IP地址
  - 并发测试TCP连接速度，选择最佳IP
  - 按连接性能排序服务器，优化查询顺序
  - 支持国内主流DoH服务器（腾讯、阿里、360、百度等）

- **`examples/smart_dns_example.py`**: 智能DNS解析演示
  - 展示多种查询策略的使用
  - 混合协议配置示例
  - 批量查询和性能统计

### 性能优化特性

- **IP预检测**: DoH/DoT首次连接延迟降低30-50%
- **智能路由**: 基于TCP连接测试的服务器选择
- **故障快速恢复**: 3秒超时机制，支持IPv4/IPv6双栈
- **并发检测**: ThreadPoolExecutor实现的高效IP测试

更多示例请参考 `examples/` 目录。

## Development

### Building from source

```bash
# 安装maturin
pip install maturin

# 开发模式构建
maturin develop

# 发布模式构建
maturin build --release
```

### Testing

```bash
pytest tests/
```

## License

MIT License