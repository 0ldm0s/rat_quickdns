# rat_quickdns

高性能DNS查询库，基于Rust开发，支持多种协议和智能决策。

## 特性

- 🚀 **高性能**: 基于Tokio异步运行时，支持并发查询
- 🌐 **多协议支持**: UDP、TCP、DoH (HTTPS)、DoT (TLS)
- 🎯 **智能负载均衡**: 多种策略自动选择最优服务器
- 🔄 **容错机制**: 自动重试、健康检查、故障转移
- 🗄️ **缓存支持**: 内置DNS缓存，减少重复查询
- 🔌 **跨语言集成**: 支持Python绑定
- 📊 **调用者初始化日志系统**: 使用rat_logger高性能日志库
- 🧠 **智能内存管理**: 集成rat_quick_threshold内存管理

## 快速开始

### 调用者初始化模式

本库使用调用者初始化模式，用户必须先初始化日志系统，然后才能使用DNS查询功能。

```rust
use rat_quickdns::{DnsResolverBuilder, QueryStrategy};
use rat_quickdns::builder::types::{DnsQueryRequest, DnsRecordType};
use rat_logger::{LoggerBuilder, LevelFilter, handler::term::TermConfig};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 调用者初始化日志系统
    LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .add_terminal_with_config(TermConfig::default())
        .init_global_logger()?;

    // 2. 创建并配置DNS解析器
    let resolver = DnsResolverBuilder::new(
        QueryStrategy::Smart,
        true,  // 启用 EDNS
        "global".to_string(),
    )
    .add_udp_upstream("阿里DNS", "223.5.5.5")
    .add_udp_upstream("腾讯DNS", "119.29.29.29")
    .with_timeout(Duration::from_secs(5))
    .with_verbose_logging()  // 启用详细日志
    .build()
    .await?;

    // 3. 执行DNS查询
    let request = DnsQueryRequest {
        domain: "example.com".to_string(),
        record_type: DnsRecordType::A,
        query_id: Some("test-query".to_string()),
        enable_edns: true,
        client_address: None,
        timeout_ms: None,
        disable_cache: false,
        enable_dnssec: false,
    };

    let response = resolver.query(request).await?;
    println!("DNS查询结果: {:?}", response.records);
    Ok(())
}
```

### 严格配置模式

```rust
use rat_quickdns::{StrictDnsConfig, QueryStrategy, UpstreamSpec};
use std::time::Duration;

let config = StrictDnsConfig::builder()
    .strategy(QueryStrategy::Smart)
    .timeout(Duration::from_secs(5))
    .retry_count(3)
    .enable_cache(true)
    .cache_ttl(Duration::from_secs(3600))
    .enable_upstream_monitoring(true)
    .upstream_monitoring_interval(Duration::from_secs(30))
    .port(53)
    .concurrent_queries(10)
    .buffer_size(4096)
    .enable_stats(true)
    .emergency_threshold(0.3)
    .add_upstream(UpstreamSpec::new("8.8.8.8:53".to_string(), "udp".to_string(), 1))
    .build()?;

let resolver = SmartDnsResolver::from_config(config)?;
```

### 日志系统

本库使用rat_logger高性能日志库，支持调用者初始化模式和专用DNS日志格式：

```rust
use rat_quickdns::{logger::init_dns_logger, dns_query, dns_response, dns_error};
use rat_logger::{LoggerBuilder, LevelFilter, handler::term::TermConfig};

// 1. 调用者初始化基础日志系统
LoggerBuilder::new()
    .with_level(LevelFilter::Info)
    .add_terminal_with_config(TermConfig::default())
    .init_global_logger()?;

// 2. 初始化DNS专用日志格式
init_dns_logger(LevelFilter::Info)?;

// 3. 使用专用DNS日志宏
dns_query!("example.com", "A");
dns_response!("example.com", 2, 45);
dns_error!("查询失败: 超时");
```

## Python绑定

```python
import rat_quickdns

# 创建解析器
resolver = rat_quickdns.DnsResolverBuilder.new() \
    .with_strategy("Smart") \
    .with_timeout(5) \
    .add_udp_upstream("阿里DNS", "223.5.5.5") \
    .build()

# 执行查询
response = await resolver.query("example.com", "A")
print(f"查询结果: {response.records}")
```

## 架构设计

### 核心模块

- **传输层**: `src/transport/` - UDP/TCP/DoH/DoT协议实现
- **解析器**: `src/resolver/` - 核心DNS解析逻辑
- **构建器**: `src/builder/` - DnsResolverBuilder统一构建接口
- **配置**: `src/config/` - 严格配置模式（无兜底默认值）
- **Python绑定**: `src/python_api/` - PyO3集成

### 关键特性

- **调用者初始化**: 用户必须明确初始化日志系统
- **零成本抽象**: 所有配置都由用户明确指定
- **类型安全**: 强类型系统确保配置正确性
- **线程安全**: 所有组件都支持多线程并发
- **异步优先**: 基于Tokio的异步运行时

## 构建和测试

```bash
# 构建主库
cargo build

# 构建发布版本
cargo build --release

# 运行所有测试
cargo test

# 运行基准测试
cargo bench

# 构建Python绑定
cargo build --features python-bindings
```

## 示例程序

查看 `examples/` 目录中的完整示例：

- `smart_dns_example.rs` - 智能DNS查询示例
- `mixed_protocol_test.rs` - 混合协议测试
- `dns_logger_example.rs` - 日志系统使用
- `caller_init_dns_example.rs` - 调用者初始化模式
- `dns_resolver_with_logging.rs` - DNS解析器日志配置
- `mx_record_test_udp.rs` - MX记录查询测试

运行示例：

```bash
# 智能DNS解析器示例
cargo run --example smart_dns_example

# 混合协议测试
cargo run --example mixed_protocol_test

# 日志系统示例
cargo run --example dns_logger_example

# 调用者初始化示例
cargo run --example caller_init_dns_example

# DNS解析器日志配置
cargo run --example dns_resolver_with_logging

# MX记录查询测试
cargo run --example mx_record_test_udp
```

## 许可证

本项目采用 LGPL v3 许可证。详见 [LICENSE](LICENSE) 文件。

## 贡献

欢迎提交Issue和Pull Request！

## 路线图

- [ ] 完整的DNS-over-QUIC (DoQ) 支持
- [ ] DNSSEC验证
- [ ] 更详细的性能监控
- [ ] 更多负载均衡策略
- [ ] 插件系统

---

<div align="center">
  <strong>🚀 高性能DNS查询库 - 让 DNS 查询更快、更安全、更可靠！</strong>
</div>