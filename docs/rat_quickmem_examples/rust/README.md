# rat_quickmem Rust 示例

本目录包含 rat_quickmem Rust 库的使用示例。

## 文件说明

- `basic_usage.rs` - 基础使用示例，展示核心功能
- `performance_test.rs` - 性能测试示例，基准测试和优化
- `Cargo.toml` - 项目配置文件

## 运行示例

### 基础使用示例

```bash
# 运行基础示例
cargo run --bin basic_usage

# 或者以发布模式运行（更快）
cargo run --release --bin basic_usage
```

这个示例展示了：
- 基本数据类型的编码/解码
- 复杂数据结构（HashMap、嵌套向量）
- 自定义配置使用
- 数据完整性验证

### 性能测试示例

```bash
# 运行性能测试
cargo run --bin performance_test

# 发布模式运行（推荐用于性能测试）
cargo run --release --bin performance_test
```

这个示例包含：
- 编码/解码性能基准
- 内存使用效率分析
- 批量操作性能测试
- 吞吐量计算

## 示例输出

### 基础使用示例输出

```
=== rat_quickmem Rust 基础示例 ===

--- 基础数据类型示例 ---
字符串: Hello, rat_quickmem! -> 23 bytes -> Hello, rat_quickmem!
数字: 42 -> 8 bytes -> 42
向量: [1, 2, 3, 4, 5] -> 25 bytes -> [1, 2, 3, 4, 5]

--- 复杂数据结构示例 ---
HashMap: {"name": "Alice", "age": "30", "city": "Beijing"} -> 45 bytes
解码结果: {"name": "Alice", "age": "30", "city": "Beijing"}

--- 自定义配置示例 ---
自定义配置: QuickMemConfig { max_data_size: 1048576, max_batch_count: 100, enable_compression: true }
大数据测试: 10000 bytes -> 8234 bytes (压缩率: 17.66%)
数据完整性验证: ✓

所有示例执行完成！
```

### 性能测试示例输出

```
=== rat_quickmem 性能测试 ===

--- 编码性能测试 ---
数据大小: 890000 bytes
迭代次数: 1000
平均编码时间: 245.2µs
编码吞吐量: 3456.78 MB/s

--- 解码性能测试 ---
编码数据大小: 734521 bytes
迭代次数: 1000
平均解码时间: 189.7µs
解码吞吐量: 3689.12 MB/s

--- 内存使用效率测试 ---
数据量: 100 项
原始大小: 8900 bytes
编码大小: 7234 bytes
压缩比: 0.813
节省空间: 18.7%

性能测试完成！
```

## 自定义示例

你可以基于这些示例创建自己的应用：

```rust
use rat_quickmem::{encode, decode, QuickMemConfig};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建自定义数据
    let mut data = HashMap::new();
    data.insert("user_id", "12345");
    data.insert("session", "abc-def-ghi");
    data.insert("timestamp", "2024-01-01T00:00:00Z");
    
    // 编码数据
    let encoded = encode(&data)?;
    println!("编码后大小: {} bytes", encoded.len());
    
    // 解码数据
    let decoded: HashMap<&str, &str> = decode(&encoded)?;
    println!("解码结果: {:?}", decoded);
    
    Ok(())
}
```

## 性能优化建议

### 编译优化

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
```

### 运行时优化

1. **批量处理**: 对于大量小数据，使用批量编码
2. **内存复用**: 重用编码缓冲区
3. **配置调优**: 根据数据特征调整配置参数

```rust
// 批量处理示例
let mut encoder = QuickEncoder::new();
for data in batch_data {
    encoder.encode_to_buffer(&data)?;
}
let encoded_batch = encoder.finalize()?;
```

## 故障排除

### 编译错误

如果遇到编译错误，请检查：

1. Rust 版本是否 >= 1.70
2. 依赖版本是否兼容
3. 特性标志是否正确

### 运行时错误

常见错误及解决方案：

- `DataTooLarge`: 减少数据大小或增加 `max_data_size`
- `BatchTooLarge`: 减少批量大小或增加 `max_batch_count`
- `SerializationError`: 检查数据类型是否支持序列化

### 性能问题

如果性能不理想：

1. 使用 `--release` 模式编译
2. 检查数据结构是否适合序列化
3. 考虑使用批量操作
4. 调整配置参数

## 更多信息

- [rat_quickmem 主文档](../../README.md)
- [API 文档](https://docs.rs/rat_quickmem)
- [安全指南](../../SECURITY.md)
- [性能指南](../../docs/PERFORMANCE.md)