# rat_quickdns

高性能、开箱即用的DNS解析库，集成了`rat_quickmem`内存管理和`bincode2`序列化。

## 特性

- 🚀 **高性能**: 基于Tokio异步运行时，支持并发查询
- 🔧 **开箱即用**: 提供构造器模式和便捷宏，快速上手
- 📦 **序列化友好**: 集成bincode2，支持高效的二进制序列化
- 🧠 **智能内存管理**: 集成rat_quickmem，优化内存使用
- 🌐 **多协议支持**: UDP、TCP、DoH (HTTPS)、DoT (TLS)
- ⚡ **连接优化**: IP预检测技术，自动选择最快连接路径，显著降低DoH/DoT首次连接延迟
- 🎯 **智能负载均衡**: 混合上游策略，自动选择最优DNS服务器
- 🔄 **容错机制**: 自动重试、健康检查、故障转移
- 🗄️ **缓存支持**: 内置DNS缓存，减少重复查询
- 🔌 **跨语言集成**: 支持Tauri和PyO3集成

## 快速开始

### 基础用法

```rust
use rat_quickdns::quick_dns;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用默认配置创建解析器
    let resolver = quick_dns!()?;
    
    // 解析域名
    let ips = resolver.resolve("example.com").await?;
    println!("IPs: {:?}", ips);
    
    Ok(())
}
```

### 构造器模式

```rust
use rat_quickdns::DnsResolverBuilder;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let resolver = DnsResolverBuilder::new()
        .add_udp_server("223.5.5.5", 53)      // 阿里DNS
        .add_udp_server("119.29.29.29", 53)   // 腾讯DNS
        .add_doh_server("https://dns.alidns.com/dns-query")
        .with_timeout(Duration::from_secs(3))
        .with_retry_count(2)
        .with_cache(true)
        .build()?;
    
    // 解析不同类型的记录
    let a_records = resolver.resolve("github.com").await?;
    let mx_records = resolver.resolve_type("gmail.com", "MX").await?;
    
    println!("A records: {:?}", a_records);
    println!("MX records: {:?}", mx_records);
    
    Ok(())
}
```

### 便捷宏用法

```rust
use rat_quickdns::quick_dns;

// 默认配置
let resolver = quick_dns!()?;

// 自定义超时
let resolver = quick_dns!(timeout = 5)?;

// 使用公共DNS
let resolver = quick_dns!(public)?;

// 自定义服务器
let resolver = quick_dns!(servers = ["8.8.8.8", "8.8.4.4"])?;
```

### 序列化功能

```rust
use rat_quickdns::{
    EasyDnsResolver, create_dns_query, 
    encode_dns_query, decode_dns_response
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let resolver = EasyDnsResolver::default()?;
    
    // 创建查询请求
    let request = create_dns_query("example.com", "A");
    
    // 编码为二进制
    let encoded_request = encode_dns_query(&request)?;
    
    // 处理编码请求
    let encoded_response = resolver.process_encoded_query(&encoded_request).await?;
    
    // 解码响应
    let response = decode_dns_response(&encoded_response)?;
    
    println!("Query ID: {}", response.query_id);
    println!("Success: {}", response.success);
    println!("Records: {:?}", response.records);
    
    Ok(())
}
```

### 批量查询

```rust
use rat_quickdns::{EasyDnsResolver, create_dns_query};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let resolver = EasyDnsResolver::quick_setup()?;
    
    // 创建批量查询
    let requests = vec![
        create_dns_query("baidu.com", "A"),
        create_dns_query("qq.com", "A"),
        create_dns_query("github.com", "A"),
    ];
    
    // 执行批量查询
    let responses = resolver.process_batch_queries(requests).await?;
    
    for response in responses {
        println!("{}: {} records", response.domain, response.records.len());
    }
    
    Ok(())
}
```

## 跨语言集成

### Tauri集成

在Tauri应用中使用DNS解析功能：

```rust
// src-tauri/src/dns.rs
use rat_quickdns::EasyDnsResolver;

#[tauri::command]
pub async fn resolve_domain(domain: String) -> Result<Vec<String>, String> {
    let resolver = EasyDnsResolver::quick_setup()
        .map_err(|e| e.to_string())?;
    
    resolver.resolve(&domain).await
        .map_err(|e| e.to_string())
}
```

前端调用：

```typescript
import { invoke } from "@tauri-apps/api/tauri";

const ips = await invoke<string[]>("resolve_domain", {
    domain: "example.com"
});
console.log("IPs:", ips);
```

### PyO3集成

将DNS解析功能暴露给Python：

```rust
use pyo3::prelude::*;
use rat_quickdns::EasyDnsResolver;

#[pyfunction]
fn quick_resolve(domain: &str) -> PyResult<Vec<String>> {
    let resolver = EasyDnsResolver::quick_setup()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
    
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        resolver.resolve(domain).await
    }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

#[pymodule]
fn rat_quickdns_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(quick_resolve, m)?)?
    Ok(())
}
```

Python使用：

```python
import rat_quickdns_py

ips = rat_quickdns_py.quick_resolve("example.com")
print(f"IPs: {ips}")
```

## 配置选项

### DNS服务器配置

```rust
let resolver = DnsResolverBuilder::new()
    // UDP服务器
    .add_udp_server("223.5.5.5", 53)      // 阿里DNS
    .add_udp_server("119.29.29.29", 53)   // 腾讯DNS
    .add_udp_server("114.114.114.114", 53) // 114DNS
    
    // DoH服务器
    .add_doh_server("https://dns.alidns.com/dns-query")
    .add_doh_server("https://doh.pub/dns-query")
    
    // DoT服务器
    .add_dot_server("223.5.5.5", 853)
    .add_dot_server("1.1.1.1", 853)
    
    .build()?;
```

### 性能调优

```rust
let resolver = DnsResolverBuilder::new()
    .with_timeout(Duration::from_secs(3))     // 查询超时
    .with_retry_count(2)                      // 重试次数
    .with_cache(true)                         // 启用缓存
    .with_cache_size(1000)                    // 缓存大小
    .with_concurrent_queries(10)              // 并发查询数
    .build()?;
```

## 示例

查看`examples/`目录获取更多示例：

- [`smart_dns_example.rs`](examples/smart_dns_example.rs) - 智能DNS解析器示例（✅ 测试成功）
- [`tauri_integration_example.rs`](examples/tauri_integration_example.rs) - Tauri集成示例

### Python测试工具

- **`python/examples/test_doh_only.py`**: DoH专项测试，包含IP预检测功能
  - 自动解析DoH服务器的所有IP地址
  - 并发测试TCP连接速度，选择最佳IP
  - 按连接性能排序服务器，优化查询顺序
  - 支持国内主流DoH服务器（腾讯、阿里、360、百度等）

- **`python/examples/smart_dns_example.py`**: 智能DNS解析演示
  - 展示多种查询策略的使用
  - 混合协议配置示例
  - 批量查询和性能统计

### 性能优化特性

- **IP预检测**: DoH/DoT首次连接延迟降低30-50%
- **智能路由**: 基于TCP连接测试的服务器选择
- **故障快速恢复**: 3秒超时机制，支持IPv4/IPv6双栈
- **并发检测**: ThreadPoolExecutor实现的高效IP测试

运行示例：

```bash
# 智能DNS解析器示例（支持FIFO和智能决策模式）
cargo run --example smart_dns_example

# Tauri集成示例（仅代码演示）
cargo run --example tauri_integration_example

# Python DoH测试（需要先构建Python绑定）
cd python && python examples/test_doh_only.py
```

## ✨ 特性

### 🚀 多协议支持
- **UDP DNS**: 传统的 UDP DNS 查询
- **TCP DNS**: 基于 TCP 的 DNS 查询
- **DoT (DNS over TLS)**: 加密的 DNS 查询
- **DoH (DNS over HTTPS)**: 基于 HTTPS 的 DNS 查询

### ⚖️ 负载均衡
- **轮询 (Round Robin)**: 依次使用每个服务器
- **随机 (Random)**: 随机选择服务器
- **加权 (Weighted)**: 根据权重分配请求
- **最少连接 (Least Connections)**: 选择连接数最少的服务器
- **一致性哈希 (Consistent Hash)**: 基于查询内容的一致性路由
- **最快响应 (Fastest)**: 选择响应最快的服务器

### 🧠 智能缓存
- **TTL 缓存**: 基于 DNS 记录 TTL 的缓存
- **LRU 策略**: 最近最少使用的缓存淘汰
- **LFU 策略**: 最少使用频率的缓存淘汰
- **自定义缓存策略**: 支持自定义缓存行为

### 🛡️ 查询过滤
- **域名黑白名单**: 支持精确匹配和通配符
- **正则表达式过滤**: 灵活的模式匹配
- **记录类型过滤**: 按 DNS 记录类型过滤
- **IP 地址过滤**: 支持 IP 范围和 CIDR
- **自定义过滤规则**: 可扩展的过滤框架

### 🏥 健康检查
- **自动故障检测**: 实时监控服务器健康状态
- **自动恢复**: 故障服务器恢复后自动重新启用
- **自适应检查间隔**: 根据服务器状态调整检查频率
- **故障转移**: 自动切换到健康的服务器

### 📊 性能监控
- **详细指标**: 查询延迟、成功率、错误统计
- **Prometheus 集成**: 支持 Prometheus 指标导出
- **分布式追踪**: 支持 OpenTelemetry 追踪
- **性能分析**: 内置性能分析工具

### 🌍 客户端IP转发 (EDNS Client Subnet)
- **地理位置优化**: 根据客户端真实IP获得最优DNS解析结果
- **CDN支持**: 为CDN服务提供精确的地理位置信息
- **IPv4/IPv6支持**: 同时支持IPv4和IPv6客户端子网
- **自定义子网掩码**: 灵活配置子网精度
- **默认客户端IP**: 支持设置全局默认客户端IP

### 🔒 安全特性
- **DNSSEC 验证**: 支持 DNS 安全扩展验证
- **速率限制**: 防止 DNS 查询滥用
- **审计日志**: 详细的查询和操作日志
- **TLS 证书验证**: 严格的证书验证

## 🚀 快速开始

### 安装

将以下内容添加到您的 `Cargo.toml`：

```toml
[dependencies]
rat_quickdns = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

### 基本用法

```rust
use rat_quickdns::{
    DnsClient, DnsConfig, DnsServerConfig, TransportType,
    QueryType, QueryClass
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建默认配置
    let config = DnsConfig::default();
    
    // 创建 DNS 客户端
    let client = DnsClient::new(config).await?;
    
    // 执行 DNS 查询
    let response = client.query(
        "example.com",
        QueryType::A,
        QueryClass::IN
    ).await?;
    
    println!("DNS Response: {:?}", response);
    Ok(())
}
```

### 自定义配置

```rust
use rat_quickdns::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = DnsConfigBuilder::new()
        .add_server(DnsServerConfig {
            address: "8.8.8.8:53".parse()?,
            transport: TransportType::Udp,
            weight: 1,
            priority: 1,
            timeout: Duration::from_secs(5),
            enabled: true,
            label: Some("Google DNS".to_string()),
            tls_config: None,
        })
        .strategy(QueryStrategy::Parallel)
        .global_timeout(Duration::from_secs(10))
        .enable_cache(true)
        .cache_size(2000)
        .build()?;
    
    let client = DnsClient::new(config).await?;
    
    // 查询 A 记录
    let ipv4_addrs = resolve_ipv4("example.com").await?;
    println!("IPv4 addresses: {:?}", ipv4_addrs);
    
    // 查询 AAAA 记录
    let ipv6_addrs = resolve_ipv6("example.com").await?;
    println!("IPv6 addresses: {:?}", ipv6_addrs);
    
    // 查询 MX 记录
    let mx_records = resolve_mx("example.com").await?;
    println!("MX records: {:?}", mx_records);
    
    Ok(())
}
```

## 🌍 客户端IP转发 (EDNS Client Subnet)

客户端IP转发功能允许DNS服务器根据客户端的真实IP地址返回最优的解析结果，这对CDN和地理位置相关的服务特别有用。

### 使用客户端IP查询

```rust
use rat_quickdns::{Resolver, RecordType};
use std::net::{IpAddr, Ipv4Addr};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut resolver = Resolver::new(Default::default());
    
    // 使用特定的客户端IP进行查询
    let client_ip = IpAddr::V4(Ipv4Addr::new(203, 208, 60, 1)); // 中国IP
    let response = resolver.query_with_client_ip(
        "www.example.com", 
        RecordType::A, 
        client_ip
    ).await?;
    
    println!("查询结果(客户端IP: {}): {:?}", client_ip, response.answers);
    Ok(())
}
```

### 使用自定义客户端子网

```rust
use rat_quickdns::{ClientSubnet, QClass};
use std::net::Ipv4Addr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut resolver = Resolver::new(Default::default());
    
    // 创建自定义客户端子网
    let subnet = ClientSubnet::from_ipv4(
        Ipv4Addr::new(8, 8, 8, 0), // 网络地址
        24 // 子网掩码长度
    );
    
    let response = resolver.query_with_client_subnet(
        "www.example.com",
        RecordType::A,
        QClass::IN,
        Some(subnet)
    ).await?;
    
    println!("查询结果: {:?}", response.answers);
    Ok(())
}
```

### 设置默认客户端IP

```rust
use std::net::{IpAddr, Ipv4Addr};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut resolver = Resolver::new(Default::default());
    
    // 设置默认客户端IP，后续所有查询都会使用此IP
    let default_ip = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
    resolver.set_default_client_ip(Some(default_ip));
    
    // 现在所有查询都会自动包含客户端IP信息
    let response = resolver.query("www.example.com", RecordType::A).await?;
    
    println!("查询结果(默认客户端IP: {}): {:?}", default_ip, response.answers);
    Ok(())
}
```

### IPv6客户端子网支持

```rust
use rat_quickdns::ClientSubnet;
use std::net::Ipv6Addr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut resolver = Resolver::new(Default::default());
    
    // 创建IPv6客户端子网
    let ipv6_subnet = ClientSubnet::from_ipv6(
        Ipv6Addr::new(0x2001, 0xda8, 0x8000, 0, 0, 0, 0, 0), // 中国IPv6网段
        48 // IPv6子网掩码长度
    );
    
    let response = resolver.query_with_client_subnet(
        "www.example.com",
        RecordType::AAAA,
        QClass::IN,
        Some(ipv6_subnet)
    ).await?;
    
    println!("IPv6查询结果: {:?}", response.answers);
    Ok(())
}
```

## 🔐 安全 DNS (DoT/DoH)

### DNS over TLS (DoT)

```rust
use rat_quickdns::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dot_server = DnsConfig::create_dot_server("cloudflare-dns.com", 853)?;
    
    let mut config = DnsConfig::default();
    config.servers = vec![dot_server];
    
    let client = DnsClient::new(config).await?;
    let response = client.query("example.com", QueryType::A, QueryClass::IN).await?;
    
    println!("Secure DNS response: {:?}", response);
    Ok(())
}
```

### DNS over HTTPS (DoH)

```rust
use rat_quickdns::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let doh_server = DnsConfig::create_doh_server("/dns-query", "cloudflare-dns.com")?;
    
    let mut config = DnsConfig::default();
    config.servers = vec![doh_server];
    
    let client = DnsClient::new(config).await?;
    let response = client.query("example.com", QueryType::A, QueryClass::IN).await?;
    
    println!("DoH response: {:?}", response);
    Ok(())
}
```

### 便利函数

```rust
use rat_quickdns::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用安全客户端
    let client = create_secure_client().await?;
    
    // 或者使用简单客户端
    let client = create_simple_client().await?;
    
    // 直接解析不同类型的记录
    let ipv4_addrs = resolve_ipv4("example.com").await?;
    let ipv6_addrs = resolve_ipv6("example.com").await?;
    let mx_records = resolve_mx("example.com").await?;
    let txt_records = resolve_txt("example.com").await?;
    
    Ok(())
}
```

## 🛡️ 查询过滤

```rust
use rat_quickdns::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = DnsConfig::default();
    
    // 启用过滤器
    config.filter.enabled = true;
    
    // 添加黑名单域名
    config.filter.blacklist_domains = vec![
        "malware.com".to_string(),
        "phishing.net".to_string(),
        "*.ads.com".to_string(),  // 通配符支持
    ];
    
    // 添加白名单域名
    config.filter.whitelist_domains = vec![
        "trusted.com".to_string(),
        "*.safe.org".to_string(),
    ];
    
    let client = DnsClient::new(config).await?;
    
    // 被过滤的域名查询将返回错误
    let result = client.query("malware.com", QueryType::A, QueryClass::IN).await;
    assert!(result.is_err());
    
    Ok(())
}
```

## 📊 性能监控和日志

### 启用日志系统

```rust
use rat_quickdns::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化 DNS 日志系统
    #[cfg(feature = "logging")]
    {
        use rat_quickdns::logger::*;
        
        // 使用默认配置初始化日志
        init_dns_logger();
        
        // 或者使用自定义配置
        let log_config = DnsLogConfig::terminal()
            .with_level(DnsLogLevel::Debug)
            .with_colors(true)
            .with_emoji(true)
            .enable_query_log(true)
            .enable_performance_log(true);
        
        init_dns_logger_with_config(log_config);
    }
    
    let mut config = DnsConfig::default();
    #[cfg(feature = "logging")]
    {
        config.log.enable_performance_log = true;
        config.log.enable_query_log = true;
    }
    
    let client = DnsClient::new(config).await?;
    
    // 执行一些查询
    for domain in ["example.com", "google.com", "github.com"] {
        let _ = client.query(domain, QueryType::A, QueryClass::IN).await;
    }
    
    // 获取统计信息
    let stats = client.get_stats().await;
    println!("Total queries: {}", stats.total_queries);
    println!("Success rate: {:.2}%", stats.success_rate() * 100.0);
    println!("Average latency: {:?}", stats.average_latency());
    
    Ok(())
}
```

### 日志输出到文件

```rust
use rat_quickdns::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "logging")]
    {
        use rat_quickdns::logger::*;
        use std::path::PathBuf;
        
        // 配置日志输出到文件
        let log_config = DnsLogConfig::file(PathBuf::from("./logs"))
            .with_level(DnsLogLevel::Info)
            .with_max_file_size(10 * 1024 * 1024)  // 10MB
            .with_max_compressed_files(5)
            .enable_query_log(true)
            .enable_performance_log(true);
        
        init_dns_logger_with_config(log_config);
    }
    
    let client = DnsClient::new(DnsConfig::default()).await?;
    let response = client.query("example.com", QueryType::A, QueryClass::IN).await?;
    
    println!("DNS Response: {:?}", response);
    Ok(())
}
```

### 日志输出到网络

```rust
use rat_quickdns::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "logging")]
    {
        use rat_quickdns::logger::*;
        
        // 配置日志输出到远程服务器
        let log_config = DnsLogConfig::udp(
            "192.168.1.100".to_string(),
            9999,
            "my_auth_token".to_string(),
            "rat_quickdns_app".to_string()
        )
        .with_level(DnsLogLevel::Debug)
        .enable_query_log(true)
        .enable_performance_log(true);
        
        init_dns_logger_with_config(log_config);
    }
    
    let client = DnsClient::new(DnsConfig::default()).await?;
    let response = client.query("example.com", QueryType::A, QueryClass::IN).await?;
    
    println!("DNS Response: {:?}", response);
    Ok(())
}
```

## 🏗️ 架构

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   DNS Client    │───▶│   Resolver      │───▶│   Transport     │
│                 │    │                 │    │                 │
│ - Query API     │    │ - Load Balancer │    │ - UDP/TCP       │
│ - Configuration │    │ - Health Check  │    │ - DoT/DoH       │
│ - Statistics    │    │ - Query Filter  │    │ - Connection    │
└─────────────────┘    │ - Cache         │    │   Management    │
                       │ - Strategy      │    └─────────────────┘
                       └─────────────────┘
                               │
                       ┌─────────────────┐
                       │   Protocol      │
                       │                 │
                       │ - Message       │
                       │   Parsing       │
                       │ - Encoding/     │
                       │   Decoding      │
                       │ - Validation    │
                       └─────────────────┘
```

## 🔧 功能特性

### 可选功能

在 `Cargo.toml` 中启用所需的功能：

```toml
[dependencies]
rat_quickdns = { version = "0.1.0", features = ["full"] }

# 或者选择特定功能
rat_quickdns = { version = "0.1.0", features = [
    "tokio",        # 异步运行时支持
    "serde",        # 序列化支持
    "cache",        # 缓存功能
    "filter",       # 查询过滤
    "health-check", # 健康检查
    "dot",          # DNS over TLS
    "doh",          # DNS over HTTPS
    "dnssec",       # DNSSEC 验证
    "metrics",      # Prometheus 指标
    "logging",      # 日志系统支持
] }
```

### 可用功能列表

- `tokio`: 异步运行时支持（默认启用）
- `serde`: 序列化和配置文件支持（默认启用）
- `cache`: DNS 缓存功能（默认启用）
- `filter`: 查询过滤功能（默认启用）
- `health-check`: 服务器健康检查（默认启用）
- `udp`: UDP 传输协议支持
- `tcp`: TCP 传输协议支持
- `dot`: DNS over TLS 支持
- `doh`: DNS over HTTPS 支持
- `dnssec`: DNSSEC 验证支持
- `rate-limiting`: 速率限制功能
- `metrics`: Prometheus 指标导出
- `logging`: 日志系统支持
- `full`: 启用所有功能

## 📚 示例

查看 `examples/` 目录中的更多示例：

- [`smart_dns_example.rs`](examples/smart_dns_example.rs) - 智能DNS解析器示例（✅ 测试成功）
  - 演示FIFO和智能决策模式的使用
  - 支持多种DNS记录类型查询（A、AAAA、MX、TXT）
  - 包含健康检查和统计信息功能
  - 支持多上游服务器配置和负载均衡

- [`tauri_integration_example.rs`](examples/tauri_integration_example.rs) - Tauri集成示例
  - 演示如何在Tauri应用中集成rat_quickdns
  - 提供前端和后端的完整代码示例
  - 支持域名解析、批量查询和二进制数据处理

运行示例：

```bash
# 智能DNS解析器示例（推荐）
cargo run --example smart_dns_example

# Tauri集成示例（仅代码演示）
cargo run --example tauri_integration_example
```

## 🧪 测试

```bash
# 运行所有测试
cargo test --all-features

# 运行特定模块测试
cargo test --all-features cache::

# 运行集成测试
cargo test --all-features --test integration

# 运行基准测试
cargo bench --all-features
```

## 📈 性能

### 基准测试结果

```
DNS Query Performance:
├── UDP Query (1000 requests)     │ 1.2ms avg │ 850 req/s
├── TCP Query (1000 requests)     │ 2.1ms avg │ 476 req/s
├── DoT Query (1000 requests)     │ 3.8ms avg │ 263 req/s
├── DoH Query (1000 requests)     │ 4.2ms avg │ 238 req/s
└── Cached Query (1000 requests)  │ 0.1ms avg │ 10000 req/s

Load Balancing Performance:
├── Round Robin                   │ 1.3ms avg │ 769 req/s
├── Random                        │ 1.2ms avg │ 833 req/s
├── Weighted                      │ 1.4ms avg │ 714 req/s
├── Least Connections            │ 1.5ms avg │ 667 req/s
└── Consistent Hash              │ 1.6ms avg │ 625 req/s

Cache Performance:
├── LRU Cache Hit                │ 0.05ms avg │ 20000 req/s
├── LFU Cache Hit                │ 0.06ms avg │ 16667 req/s
└── Cache Miss + Store           │ 1.3ms avg │ 769 req/s
```

运行基准测试：

```bash
cargo bench --all-features
```

## 🤝 贡献

我们欢迎各种形式的贡献！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解详细信息。

### 开发环境设置

```bash
# 克隆仓库
git clone https://github.com/your-org/rat_quickdns.git
cd rat_quickdns

# 安装依赖
cargo build --all-features

# 运行测试
cargo test --all-features

# 运行 linting
cargo clippy --all-features

# 格式化代码
cargo fmt
```

## 📄 许可证

本项目采用 MIT 或 Apache-2.0 双重许可证。详情请参见：

- [MIT License](LICENSE-MIT)
- [Apache License 2.0](LICENSE-APACHE)

## 🙏 致谢

- [Trust-DNS](https://github.com/bluejekyll/trust-dns) - DNS 协议实现参考
- [Hickory DNS](https://github.com/hickory-dns/hickory-dns) - 现代 DNS 库设计
- [Tokio](https://tokio.rs/) - 异步运行时
- [Hyper](https://hyper.rs/) - HTTP 客户端
- [Rustls](https://github.com/rustls/rustls) - TLS 实现

---

<div align="center">
  <strong>🚀 让 DNS 查询更快、更安全、更可靠！</strong>
</div>