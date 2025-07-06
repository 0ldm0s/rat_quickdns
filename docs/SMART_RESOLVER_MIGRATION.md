# SmartDnsResolver 统一入口迁移指南

## 概述

`SmartDnsResolver` 已经重构为统一的DNS解析器入口，通过 `DnsResolverBuilder` 构建器模式创建。所有示例和测试用例都应该使用这个统一入口。

## 新的API设计

### 1. 构建器模式

```rust
use rat_quickdns::builder::{DnsResolverBuilder, QueryStrategy};
use rat_quickdns::builder::types::{DnsQueryRequest, DnsRecordType};
use rat_quickmem::QuickMemConfig;

// 创建 QuickMem 配置
let quickmem_config = QuickMemConfig::new(
    1024 * 1024,  // 1MB 内存池
    4096,         // 4KB 块大小
    100,          // 最大块数
);

// 创建解析器
let resolver = DnsResolverBuilder::new(
    QueryStrategy::Smart,  // 查询策略
    true,                  // 启用 EDNS
    "global".to_string(),  // 区域
    quickmem_config,       // QuickMem 配置
)
.with_timeout(Duration::from_secs(5))
.with_retry_count(2)
.add_udp_upstream("阿里DNS", "223.5.5.5")
.add_doh_upstream("Cloudflare DoH", "https://cloudflare-dns.com/dns-query")
.build()
.await?;
```

### 2. 查询API

```rust
// 创建查询请求
let request = DnsQueryRequest {
    domain: "www.example.com".to_string(),
    record_type: DnsRecordType::A,
    query_id: Some("test-1".to_string()),
};

// 执行查询
let response = resolver.query(request).await?;
```

## 支持的查询策略

### 1. Smart 策略（推荐）
- 自动选择最优上游服务器
- 支持智能决策引擎
- 自动健康检查和故障转移

```rust
let smart_resolver = DnsResolverBuilder::new(
    QueryStrategy::Smart,
    true,
    "global".to_string(),
    quickmem_config,
)
.with_upstream_monitoring(true)
.add_udp_upstream("阿里DNS", "223.5.5.5")
.add_udp_upstream("腾讯DNS", "119.29.29.29")
.build()
.await?;
```

### 2. RoundRobin 策略
- 轮询负载均衡
- 支持优化配置

```rust
let round_robin_resolver = DnsResolverBuilder::new(
    QueryStrategy::RoundRobin,
    true,
    "global".to_string(),
    quickmem_config,
)
.optimize_for_round_robin()  // 应用轮询优化
.add_udp_upstream("Google DNS", "8.8.8.8")
.add_udp_upstream("Cloudflare DNS", "1.1.1.1")
.build()
.await?;
```

### 3. FIFO 策略
- 按顺序尝试上游服务器
- 适合有明确优先级的场景

```rust
let fifo_resolver = DnsResolverBuilder::new(
    QueryStrategy::Fifo,
    true,
    "global".to_string(),
    quickmem_config,
)
.add_udp_upstream("首选DNS", "114.114.114.114")
.add_udp_upstream("备用DNS", "8.8.8.8")
.build()
.await?;
```

## 便捷方法

### 1. 公共DNS配置

```rust
let public_dns_resolver = DnsResolverBuilder::new(
    QueryStrategy::Smart,
    true,
    "global".to_string(),
    QuickMemConfig::default(),
)
.with_public_dns()?  // 自动添加多个公共DNS服务器
.build()
.await?;
```

### 2. 日志配置

```rust
// 详细日志
let verbose_resolver = DnsResolverBuilder::new(...)
    .with_verbose_logging()
    .build().await?;

// 静默日志
let quiet_resolver = DnsResolverBuilder::new(...)
    .with_quiet_logging()
    .build().await?;

// 自定义日志
let custom_resolver = DnsResolverBuilder::new(...)
    .with_log_level(LevelFilter::Info)
    .with_dns_log_format(true)
    .build().await?;
```

## 迁移步骤

### 1. 更新导入

```rust
// 旧的导入
use rat_quickdns::{DnsResolverBuilder, types::*};

// 新的导入
use rat_quickdns::builder::{DnsResolverBuilder, QueryStrategy};
use rat_quickdns::builder::types::{DnsQueryRequest, DnsRecordType};
use rat_quickmem::QuickMemConfig;
```

### 2. 更新构建器调用

```rust
// 旧的方式
let resolver = DnsResolverBuilder::new()
    .query_strategy(QueryStrategy::Smart)
    .enable_edns(true)
    .add_udp_upstream("DNS", "8.8.8.8")
    .build().await?;

// 新的方式
let resolver = DnsResolverBuilder::new(
    QueryStrategy::Smart,
    true,  // 启用 EDNS
    "global".to_string(),
    QuickMemConfig::default(),
)
.add_udp_upstream("DNS", "8.8.8.8")
.build().await?;
```

### 3. 更新查询调用

```rust
// 旧的方式
let response = resolver.query("example.com", RecordType::A, QClass::IN).await?;

// 新的方式
let request = DnsQueryRequest {
    domain: "example.com".to_string(),
    record_type: DnsRecordType::A,
    query_id: Some("test".to_string()),
};
let response = resolver.query(request).await?;
```

## 优势

1. **统一入口**: 所有DNS解析功能通过 `SmartDnsResolver` 统一提供
2. **类型安全**: 强类型的查询请求和响应
3. **灵活配置**: 支持多种查询策略和配置选项
4. **性能优化**: 内置智能决策和负载均衡
5. **易于扩展**: 模块化设计便于添加新功能

## 示例文件状态

- ✅ `unified_smart_resolver_example.rs` - 新创建的统一示例
- ✅ `smart_dns_example.rs` - 已更新
- ✅ `mx_record_test_dot.rs` - 已更新
- ✅ `comprehensive_dns_query.rs` - 已更新
- 🔄 其他示例文件正在更新中...

## 注意事项

1. 所有新的测试用例都应该使用 `SmartDnsResolver`
2. 旧的API将逐步废弃，建议尽快迁移
3. `QuickMemConfig` 是必需的参数，可以使用 `QuickMemConfig::default()` 作为默认配置
4. 查询策略现在在构造函数中指定，不再通过链式调用设置