# 通用应急处理机制

## 概述

本文档描述了 `rat_quickdns` 中实现的通用应急处理机制。该机制适用于所有查询策略（FIFO、SMART、ROUND_ROBIN），为DNS解析器提供了统一的故障处理和恢复能力。

## 设计原则

### 1. 策略无关性
应急处理机制独立于具体的查询策略，所有策略都能受益于统一的故障检测和处理逻辑。

### 2. 分层处理
- **预检查**: 在执行查询前检查应急状态
- **查询执行**: 各策略按自身逻辑执行查询
- **错误增强**: 查询失败后增强错误信息

### 3. 智能诊断
提供详细的故障信息，包括失败服务器列表、失败统计、最后工作服务器等。

## 核心组件

### 1. 应急状态检测

```rust
/// 通用应急状态检查
async fn check_emergency_status(&self) -> Option<String> {
    if let Some(engine) = &self.decision_engine {
        if engine.all_upstreams_failed().await {
            let emergency_info = engine.get_emergency_response_info().await;
            return Some(format!(
                "🚨 应急模式激活: {} (策略: {:?})",
                emergency_info.emergency_message,
                self.query_strategy
            ));
        }
    }
    None
}
```

### 2. 错误信息增强

```rust
/// 增强错误信息，添加应急响应详情
async fn enhance_error_with_emergency_info(&self, original_error: DnsError) -> String {
    if let Some(engine) = &self.decision_engine {
        let emergency_info = engine.get_emergency_response_info().await;
        
        if emergency_info.all_servers_failed {
            format!(
                "查询失败 (策略: {:?}): {}\n🚨 应急信息: {}\n📊 失败统计: {}次\n📋 失败服务器: [{}]",
                self.query_strategy,
                original_error,
                emergency_info.emergency_message,
                emergency_info.total_failures,
                emergency_info.failed_servers.iter()
                    .map(|s| format!("{} ({}次)", s.name, s.consecutive_failures))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        } else {
            // 处理部分故障情况...
        }
    }
}
```

### 3. 应急响应信息结构

```rust
/// 应急响应信息
#[derive(Debug, Clone)]
pub struct EmergencyResponseInfo {
    /// 是否所有服务器都失败
    pub all_servers_failed: bool,
    /// 失败的服务器列表
    pub failed_servers: Vec<FailedServerInfo>,
    /// 最后工作的服务器
    pub last_working_server: Option<String>,
    /// 总失败次数
    pub total_failures: u32,
    /// 应急消息
    pub emergency_message: String,
}

/// 失败服务器信息
#[derive(Debug, Clone)]
pub struct FailedServerInfo {
    /// 服务器名称
    pub name: String,
    /// 服务器地址
    pub server: String,
    /// 连续失败次数
    pub consecutive_failures: u32,
    /// 最后失败时间
    pub last_failure_time: Option<Instant>,
    /// 失败原因
    pub failure_reason: String,
}
```

## 查询流程集成

### 统一查询入口

```rust
pub async fn query(&self, request: &DnsQueryRequest) -> Result<(crate::Response, String)> {
    // 1. 预检查应急状态
    if let Some(emergency_msg) = self.check_emergency_status().await {
        return Err(DnsError::Server(emergency_msg));
    }
    
    // 2. 执行策略特定的查询逻辑
    let result = self.execute_query_strategy(request).await;
    
    // 3. 处理查询结果
    match result {
        Ok((response, server)) => {
            // 更新性能指标
            if let Some(engine) = &self.decision_engine {
                engine.update_metrics(&server, Duration::from_millis(0), true, true).await;
            }
            Ok((response, server))
        },
        Err(e) => {
            // 增强错误信息
            let enhanced_error = self.enhance_error_with_emergency_info(e).await;
            Err(DnsError::Server(enhanced_error))
        }
    }
}
```

### 策略特定实现

各查询策略保持其核心逻辑不变，但移除了策略特定的应急处理代码：

```rust
// FIFO策略 - 简化后
async fn query_fifo(&self, request: &DnsQueryRequest) -> Result<(crate::Response, String)> {
    let record_type = self.convert_record_type(request.record_type);
    
    if let Some(engine) = &self.decision_engine {
        if let Some(spec) = engine.select_fifo_upstream().await {
            // 执行查询并更新指标
            // ...
        } else {
            Err(DnsError::NoUpstreamAvailable)
        }
    } else {
        // 回退到基础解析器
        // ...
    }
}
```

## 应急策略

### 1. 全部服务器故障
- **检测**: 所有上游服务器健康检查失败
- **响应**: 返回详细的应急信息，包括故障统计和恢复建议
- **消息**: "DNS解析服务暂时不可用：所有X个上游服务器均无响应"

### 2. 部分服务器故障
- **检测**: 部分上游服务器健康检查失败
- **响应**: 继续使用健康服务器，记录故障信息
- **消息**: "DNS解析服务部分可用：X/Y个上游服务器正常工作"

### 3. 应急服务器选择
当没有健康服务器时，各策略会选择"最不坏"的服务器：
- **FIFO**: 选择第一个配置的服务器
- **SMART**: 选择连续失败次数最少的服务器
- **ROUND_ROBIN**: 选择连续失败次数最少的服务器

## Python API 集成

### 应急信息获取

```python
# 获取应急响应信息
emergency_info = resolver.get_emergency_response_info()

print(f"所有服务器失败: {emergency_info.all_servers_failed}")
print(f"总失败次数: {emergency_info.total_failures}")
print(f"应急消息: {emergency_info.emergency_message}")

# 失败服务器详情
for server in emergency_info.failed_servers:
    print(f"{server.name}: 连续失败 {server.consecutive_failures} 次")
```

### Python 绑定结构

```python
class PyEmergencyResponseInfo:
    all_servers_failed: bool
    failed_servers: List[PyFailedServerInfo]
    last_working_server: Optional[str]
    total_failures: int
    emergency_message: str

class PyFailedServerInfo:
    name: str
    server: str
    consecutive_failures: int
    failure_reason: str
    last_failure_time: Optional[float]
```

## 使用示例

### Rust 示例

```rust
use rat_quickdns::builder::{DnsResolverBuilder, QueryStrategy};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建解析器（任何策略都支持应急处理）
    let mut builder = DnsResolverBuilder::new();
    builder.query_strategy(QueryStrategy::Smart);
    builder.add_udp_upstream("Primary", "1.1.1.1:53", 10);
    builder.add_udp_upstream("Secondary", "8.8.8.8:53", 20);
    builder.enable_health_checker(true);
    
    let resolver = builder.build();
    
    // 查询会自动应用应急处理
    match resolver.query(&request).await {
        Ok((response, server)) => {
            println!("查询成功，使用服务器: {}", server);
        },
        Err(e) => {
            // 错误信息已经包含应急诊断信息
            println!("查询失败: {}", e);
            
            // 可以获取详细的应急信息
            if let Some(engine) = resolver.get_decision_engine() {
                let emergency_info = engine.get_emergency_response_info().await;
                // 处理应急信息...
            }
        }
    }
    
    Ok(())
}
```

### Python 示例

```python
import rat_quickdns as dns
from rat_quickdns import QueryStrategy

# 创建解析器
builder = dns.DnsResolverBuilder()
builder.query_strategy(QueryStrategy.ROUND_ROBIN)
builder.add_udp_upstream("Primary", "1.1.1.1:53", 10)
builder.add_udp_upstream("Secondary", "8.8.8.8:53", 20)
builder.enable_health_checker(True)

resolver = builder.build()

try:
    # 查询会自动应用应急处理
    ips = resolver.resolve("example.com")
    print(f"查询成功: {ips}")
except Exception as e:
    # 错误信息包含应急诊断信息
    print(f"查询失败: {e}")
    
    # 获取详细应急信息
    emergency_info = resolver.get_emergency_response_info()
    if emergency_info.all_servers_failed:
        print("所有服务器都不可用")
        for server in emergency_info.failed_servers:
            print(f"  {server.name}: 失败 {server.consecutive_failures} 次")
```

## 监控和诊断

### 健康状态监控

```python
# 获取所有服务器健康状态
health_status = resolver.get_health_status()
for server_name, is_healthy in health_status.items():
    status = "健康" if is_healthy else "不健康"
    print(f"{server_name}: {status}")
```

### 性能指标获取

```python
# 获取性能统计
stats = resolver.get_stats()
print(f"总查询次数: {stats.total_queries}")
print(f"成功查询次数: {stats.successful_queries}")
print(f"平均响应时间: {stats.average_response_time}ms")
```

## 配置建议

### 1. 健康检查配置
- **检查间隔**: 建议 2-5 秒
- **超时时间**: 建议 1-2 秒
- **失败阈值**: 建议连续失败 3-5 次后标记为不健康

### 2. 应急策略配置
- **重试次数**: 建议最多尝试 3 个不同服务器
- **重试间隔**: 建议 50-100ms 的短暂延迟
- **回退策略**: 启用应急服务器选择

### 3. 监控配置
- **定期检查**: 定期获取健康状态和应急信息
- **告警阈值**: 当失败率超过 50% 时触发告警
- **日志记录**: 记录所有应急事件用于分析

## 故障排查

### 常见问题

1. **所有服务器都标记为不健康**
   - 检查网络连接
   - 验证DNS服务器地址和端口
   - 检查防火墙设置

2. **应急信息不准确**
   - 确保健康检查已启用
   - 检查健康检查间隔设置
   - 验证决策引擎是否正确初始化

3. **错误信息不包含应急信息**
   - 确保使用了正确的查询方法
   - 检查决策引擎是否已配置
   - 验证应急处理是否已启用

### 调试技巧

1. **启用详细日志**
   ```rust
   env_logger::init();
   ```

2. **手动检查应急状态**
   ```rust
   let emergency_info = engine.get_emergency_response_info().await;
   println!("应急状态: {:?}", emergency_info);
   ```

3. **监控健康检查**
   ```python
   import time
   while True:
       health = resolver.get_health_status()
       print(f"健康状态: {health}")
       time.sleep(5)
   ```

## 总结

通用应急处理机制为 `rat_quickdns` 提供了强大的故障处理能力：

- ✅ **策略无关**: 所有查询策略都受益于统一的应急处理
- ✅ **智能诊断**: 提供详细的故障信息和恢复建议
- ✅ **自动恢复**: 支持自动故障检测和服务器选择
- ✅ **易于监控**: 提供丰富的API用于状态监控和诊断
- ✅ **Python集成**: 完整的Python API支持

这种设计确保了DNS解析服务的高可用性和可靠性，同时为运维人员提供了强大的故障排查工具。