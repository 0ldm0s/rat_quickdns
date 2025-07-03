# ROUND_ROBIN策略性能优化指南

## 概述

本文档详细介绍了`rat_quickdns`中`ROUND_ROBIN`策略的性能优化特性，包括超时优化、健康检查集成、快速失败机制等。

## 🚀 优化特性

### 1. 超时优化

**问题**: 默认5秒超时对于轮询策略过长，影响整体性能。

**解决方案**: 
- 专门的`with_round_robin_timeout()`方法，限制最大2秒
- `optimize_for_round_robin()`自动设置1.5秒超时

```rust
use rat_quickdns::builder::{DnsResolverBuilder, strategy::QueryStrategy};
use std::time::Duration;

// 方法1: 手动设置ROUND_ROBIN超时
let resolver = DnsResolverBuilder::new()
    .query_strategy(QueryStrategy::RoundRobin)
    .with_round_robin_timeout(Duration::from_millis(1500)) // 最大2秒
    .build()
    .await?;

// 方法2: 使用一键优化
let resolver = DnsResolverBuilder::new()
    .query_strategy(QueryStrategy::RoundRobin)
    .optimize_for_round_robin() // 自动应用所有优化
    .build()
    .await?;
```

### 2. 健康检查集成

**问题**: 原始轮询不考虑服务器健康状态，可能选择不可用服务器。

**解决方案**: 
- 优化的`select_round_robin_upstream()`集成健康检查
- 自动过滤不健康的上游服务器
- 备用机制：无健康服务器时选择失败次数最少的

```rust
// 健康感知的轮询选择
let upstream = engine.select_round_robin_upstream().await;

// 高性能场景的快速轮询（跳过健康检查）
let upstream = engine.select_fast_round_robin_upstream().await;
```

### 3. 快速失败机制

**问题**: 传统轮询在服务器故障时仍会等待完整超时。

**解决方案**:
- 集成连续失败计数
- 智能重试逻辑
- 快速切换到下一个服务器

### 4. 并发优化

**问题**: 单线程轮询限制了查询吞吐量。

**解决方案**:
- `optimize_for_round_robin()`自动设置并发数≥4
- 支持手动调整`with_concurrent_queries()`

## 📊 性能对比

| 指标 | 基础配置 | 优化配置 | 改进 |
|------|----------|----------|------|
| 超时时间 | 5秒 | 1.5秒 | -70% |
| 健康检查 | ❌ | ✅ | +100% |
| 并发查询 | 1 | 4+ | +300% |
| 重试次数 | 3 | 1 | -67% |
| 预期QPS | ~20 | ~60+ | +200% |

## 🛠️ 使用方法

### 基础使用

```rust
use rat_quickdns::builder::{DnsResolverBuilder, strategy::QueryStrategy};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let resolver = DnsResolverBuilder::new()
        .query_strategy(QueryStrategy::RoundRobin)
        .add_udp_upstream("阿里DNS", "223.5.5.5")
        .add_udp_upstream("腾讯DNS", "119.29.29.29")
        .add_udp_upstream("Google DNS", "8.8.8.8")
        .optimize_for_round_robin() // 一键优化
        .build()
        .await?;
    
    // 执行查询
    let result = resolver.query(DnsQueryRequest {
        query_id: Some("test-1".to_string()),
        domain: "example.com".to_string(),
        record_type: DnsRecordType::A,
    }).await?;
    
    println!("查询结果: {:?}", result);
    Ok(())
}
```

### 高级配置

```rust
let resolver = DnsResolverBuilder::new()
    .query_strategy(QueryStrategy::RoundRobin)
    // 添加多个上游服务器
    .add_udp_upstream("主DNS", "223.5.5.5")
    .add_udp_upstream("备DNS", "119.29.29.29")
    .add_doh_upstream("DoH服务", "https://dns.alidns.com/dns-query")
    // 精细化配置
    .with_round_robin_timeout(Duration::from_millis(2000)) // 2秒超时
    .with_health_check(true) // 启用健康检查
    .with_retry_count(1) // 减少重试
    .with_concurrent_queries(6) // 6个并发查询
    .with_cache_size(10000) // 缓存优化
    .build()
    .await?;
```

### Python API使用

```python
import asyncio
from rat_quickdns import PyDnsResolverBuilder, PyQueryStrategy, PyDnsRecordType

async def main():
    # 创建优化的ROUND_ROBIN解析器
    builder = PyDnsResolverBuilder()
    resolver = await (
        builder
        .query_strategy(PyQueryStrategy.RoundRobin)
        .add_udp_upstream("阿里DNS", "223.5.5.5")
        .add_udp_upstream("腾讯DNS", "119.29.29.29")
        .optimize_for_round_robin()  # 一键优化
        .build()
    )
    
    # 执行查询
    result = await resolver.query("example.com", PyDnsRecordType.A)
    print(f"查询结果: {result}")

if __name__ == "__main__":
    asyncio.run(main())
```

## 🔧 配置参数详解

### `optimize_for_round_robin()`

一键应用所有ROUND_ROBIN优化，等效于：

```rust
.with_round_robin_timeout(Duration::from_millis(1500))
.with_health_check(true)
.with_retry_count(1)
.with_concurrent_queries(4.max(current_concurrent))
```

### `with_round_robin_timeout(duration)`

- **参数**: `Duration` - 超时时间
- **限制**: 最大2秒，超过会自动截断
- **建议**: 1.5-2秒之间

### 健康检查配置

```rust
.with_health_check(true) // 启用健康检查
.with_health_check_interval(Duration::from_secs(30)) // 检查间隔
.with_health_check_timeout(Duration::from_millis(500)) // 检查超时
```

## 📈 性能监控

### 内置指标

```rust
// 获取上游服务器统计
let stats = resolver.get_upstream_stats().await;
for stat in stats {
    println!("服务器: {}", stat.name);
    println!("  成功率: {:.1}%", stat.success_rate);
    println!("  平均延迟: {:.1}ms", stat.avg_latency);
    println!("  健康状态: {}", stat.is_healthy);
}
```

### 自定义监控

```rust
use std::time::Instant;

let start = Instant::now();
let result = resolver.query(request).await?;
let latency = start.elapsed();

println!("查询耗时: {:.1}ms", latency.as_secs_f64() * 1000.0);
println!("使用服务器: {}", result.upstream_used);
```

## 🎯 最佳实践

### 1. 服务器选择

```rust
// 推荐：混合不同类型的DNS服务器
.add_udp_upstream("本地DNS", "223.5.5.5") // 低延迟
.add_doh_upstream("安全DNS", "https://dns.alidns.com/dns-query") // 高安全性
.add_dot_upstream("隐私DNS", "dns.google:853") // 隐私保护
```

### 2. 超时配置

```rust
// 根据网络环境调整
let timeout = if is_mobile_network {
    Duration::from_millis(2000) // 移动网络：2秒
} else {
    Duration::from_millis(1500) // 固定网络：1.5秒
};

builder.with_round_robin_timeout(timeout)
```

### 3. 并发控制

```rust
// 根据应用场景调整并发数
let concurrent = match app_type {
    AppType::HighThroughput => 8,  // 高吞吐量应用
    AppType::LowLatency => 4,      // 低延迟应用
    AppType::ResourceLimited => 2, // 资源受限环境
};

builder.with_concurrent_queries(concurrent)
```

### 4. 错误处理

```rust
match resolver.query(request).await {
    Ok(result) if result.success => {
        // 处理成功结果
        println!("解析成功: {:?}", result.records);
    },
    Ok(result) => {
        // 处理DNS错误
        eprintln!("DNS错误: {:?}", result.error);
    },
    Err(e) => {
        // 处理系统错误
        eprintln!("系统错误: {}", e);
    }
}
```

## 🔍 故障排查

### 常见问题

1. **查询超时频繁**
   - 检查网络连接
   - 适当增加超时时间
   - 验证上游服务器可用性

2. **成功率低**
   - 启用健康检查
   - 增加上游服务器数量
   - 检查DNS服务器配置

3. **性能不佳**
   - 使用`optimize_for_round_robin()`
   - 增加并发查询数
   - 启用缓存

### 调试模式

```rust
// 启用详细日志
env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

// 或使用环境变量
// RUST_LOG=rat_quickdns=debug cargo run
```

## 📚 示例代码

完整的示例代码请参考：
- [round_robin_optimization.rs](../examples/round_robin_optimization.rs) - Rust示例
- [round_robin_optimization.py](../examples/round_robin_optimization.py) - Python示例

## 🔄 版本兼容性

- ✅ 新增的优化方法向后兼容
- ✅ 现有API保持不变
- ✅ 默认行为未改变（需显式启用优化）

## 📞 技术支持

如有问题或建议，请：
1. 查看[FAQ文档](./FAQ.md)
2. 提交[GitHub Issue](https://github.com/your-repo/rat_quickdns/issues)
3. 参与[讨论区](https://github.com/your-repo/rat_quickdns/discussions)