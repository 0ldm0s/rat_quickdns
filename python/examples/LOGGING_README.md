# Python绑定日志初始化指南

## 概述

rat_quickdns Python绑定现在支持调用者初始化的日志系统，用户完全控制日志系统的初始化，不初始化则禁用日志功能。

## 核心特性

### 1. 调用者完全控制
- 用户必须主动初始化日志系统
- 不初始化则日志功能完全禁用
- 防止重复初始化

### 2. 多种初始化方式
- `init_logging()`: 基本配置（Info级别）
- `init_logging_with_level(level)`: 指定日志级别
- `init_logging_advanced()`: 完全自定义配置
- `init_dns_logging()`: DNS专用基本配置
- `init_dns_logging_advanced()`: DNS专用高级配置

### 3. 日志状态检查
- `is_logging_initialized()`: 检查日志系统是否已初始化

### 4. 丰富的日志函数
- 通用日志：`log_info()`, `log_warn()`, `log_error()`, `log_debug()`, `log_trace()`
- DNS专用日志：`dns_info()`, `dns_warn()`, `dns_error()`, `dns_debug()`, `dns_trace()`

## 使用示例

### 基本使用

```python
import rat_quickdns_py as dns

# 检查日志状态
print(f"日志初始化状态: {dns.is_logging_initialized()}")

# 初始化基本日志系统
dns.init_logging()

# 使用日志函数
dns.log_info("这是一条信息日志")
dns.log_error("这是一条错误日志")

# DNS专用日志
dns.dns_info("DNS查询开始")
```

### 高级配置

```python
# 高级日志配置
dns.init_logging_advanced(
    level="debug",                    # 调试级别
    enable_color=True,               # 启用颜色
    timestamp_format="%H:%M:%S",     # 自定义时间格式
    custom_format_template="[LEVEL] {message}"  # 自定义格式
)

# DNS专用高级配置
dns.init_dns_logging_advanced(
    level="debug",
    enable_dns_format=True,
    enable_color=True,
    timestamp_format="%H:%M:%S"
)
```

### 结合DNS查询

```python
import rat_quickdns_py as dns
from rat_quickdns_py import QueryStrategy

# 1. 初始化日志系统（可选）
dns.init_logging_with_level("info")

# 2. 创建DNS解析器
builder = dns.DnsResolverBuilder()
builder.query_strategy(QueryStrategy.SMART)
builder.add_udp_upstream("阿里DNS", "223.5.5.5")
builder.timeout(5.0)
resolver = builder.build()

# 3. 使用日志函数记录操作
dns.log_info("开始DNS查询")
dns.dns_info(f"查询域名: example.com")

# 4. 执行查询
ips = resolver.resolve("example.com")
dns.dns_info(f"查询成功: {ips}")
```

## 日志级别

支持的日志级别：
- `"trace"`: 最详细级别
- `"debug"`: 调试信息
- `"info"`: 一般信息（默认）
- `"warn"`: 警告信息
- `"error"`: 错误信息
- `"off"`: 关闭日志

## 注意事项

1. **一次性初始化**: 日志系统一旦初始化，无法重新配置或取消初始化
2. **非阻塞操作**: 未初始化时调用日志函数不会有任何效果
3. **性能考虑**: 生产环境建议使用 `"info"` 或 `"warn"` 级别
4. **DNS专用格式**: DNS专用日志提供更专业的DNS查询格式

## 完整示例

参考 `caller_init_log_example.py` 文件，它演示了所有日志功能的完整使用方式。

运行示例：
```bash
python examples/caller_init_log_example.py
```

## 兼容性

- 不影响现有代码的兼容性
- 日志系统完全可选
- 所有原有的DNS查询功能保持不变