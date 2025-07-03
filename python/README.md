# rat-quickdns-py

Python bindings for rat_quickdns - A high-performance DNS client library written in Rust.

## Features

- **高性能**: 基于Rust的异步DNS解析，支持多种传输协议
- **智能决策**: 自动选择最优DNS服务器，支持健康检查
- **多协议支持**: UDP、TCP、DoH (DNS over HTTPS)、DoT (DNS over TLS)
- **负载均衡**: FIFO、并行、顺序、智能决策等多种查询策略
- **EDNS支持**: 支持客户端子网、UDP负载大小等扩展功能
- **批量查询**: 高效的并发批量域名解析

## Installation

```bash
pip install rat-quickdns-py
```

## Quick Start

```python
import asyncio
from rat_quickdns_py import DnsResolver, QueryStrategy

async def main():
    # 创建DNS解析器
    resolver = DnsResolver.builder() \
        .query_strategy(QueryStrategy.SMART) \
        .enable_edns(True) \
        .add_udp_upstream("Google DNS", "8.8.8.8:53", 100) \
        .add_udp_upstream("Cloudflare DNS", "1.1.1.1:53", 100) \
        .build()
    
    # 解析单个域名
    ips = await resolver.resolve("example.com")
    print(f"example.com resolves to: {ips}")
    
    # 批量解析
    domains = ["google.com", "github.com", "microsoft.com"]
    results = await resolver.batch_query(domains)
    
    for domain, result in zip(domains, results):
        if result.is_ok():
            print(f"{domain}: {result.unwrap()}")
        else:
            print(f"{domain}: Error - {result.unwrap_err()}")

# 运行异步函数
asyncio.run(main())
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