# CLAUDE.md

本文件为Claude Code提供在此代码库中工作的指导。

## 项目概述

rat_quickdns是一个高性能DNS解析库，支持UDP、TCP、DoH(HTTPS)、DoT(TLS)多种协议，具有智能负载均衡、缓存、健康检查等功能。

## 核心架构

- **传输层**: `src/transport/` - UDP/TCP/DoH/DoT协议实现
- **解析器**: `src/resolver/` - 核心DNS解析逻辑
- **构建器**: `src/builder/` - DnsResolverBuilder统一构建接口
- **配置**: `src/config/` - 严格配置模式（无兜底默认值）
- **Python绑定**: `src/python_api/` - PyO3集成

## 构建和测试命令

### 基础构建
```bash
# 构建主库
cargo build

# 构建发布版本
cargo build --release

# 构建所有特性（包括Python绑定）
cargo build --all-features
```

### 测试
```bash
# 运行所有测试
cargo test

# 运行特定测试模块
cargo test --test dns_response_tests

# 运行基准测试
cargo bench

# 带详细输出
cargo test -- --nocapture
```

### Python绑定开发
```bash
# 进入python目录
cd python

# 使用构建脚本
python build.py all      # 构建、安装、测试
python build.py build    # 仅构建
python build.py install  # 安装到Python环境
python build.py test     # 运行Python测试

# 直接使用maturin
maturin develop          # 开发模式安装
maturin build            # 构建wheel包
```

### 示例运行
```bash
# 运行智能DNS示例
cargo run --example smart_dns_example

# 运行混合协议测试
cargo run --example mixed_protocol_test

# 运行DNS日志示例
cargo run --example dns_logger_example
```

## 开发注意事项

1. **配置严格性**: 使用`StrictDnsConfig`而非默认配置，所有参数必须显式设置
2. **日志系统**: 使用`zerg_creep`日志库，已重新导出到crate根部
3. **内存管理**: 集成`rat_quick_threshold`内存管理
4. **错误处理**: 使用`thiserror`定义明确的错误类型

## 代码风格

- 遵循Rust标准格式化
- 使用`cargo fmt`进行代码格式化
- 使用`cargo clippy`进行代码检查
- 所有公共API必须有文档注释

## 特性标志

- `default`: 基础功能
- `python-bindings`: Python绑定支持
- `orni_dns`: OrniDNS兼容性支持

## 依赖关系

- **核心运行时**: tokio
- **HTTP客户端**: hyper, reqwest
- **TLS支持**: rustls, native-tls
- **序列化**: serde, bincode
- **缓存**: lru, dashmap
- **日志**: zerg_creep (本地路径依赖)
- **内存管理**: rat_quick_threshold (本地路径依赖)