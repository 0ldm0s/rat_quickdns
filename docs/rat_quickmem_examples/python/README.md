# rat_quickmem Python 示例

本目录包含 rat_quickmem Python 绑定的使用示例。

## 文件说明

- `python_example.py` - 基础使用示例，展示编码/解码功能
- `security_test.py` - 安全特性测试，验证输入限制和错误处理

## 运行前准备

### 1. 构建 Python 绑定

```bash
# 进入 python 绑定目录
cd ../../python

# 使用 maturin 构建并安装
maturin develop

# 或者构建 wheel
maturin build --release
```

### 2. 安装依赖

```bash
pip install maturin
```

## 运行示例

### 基础使用示例

```bash
python python_example.py
```

这个示例展示了：
- 基本数据类型的编码/解码
- 复杂数据结构的处理
- 性能测试
- 错误处理

### 安全测试示例

```bash
python security_test.py
```

这个示例验证了：
- 数据大小限制
- 批量处理限制
- 恶意输入防护
- 内存安全保护

## 示例输出

运行示例后，你将看到类似以下的输出：

```
=== rat_quickmem Python 绑定示例 ===

--- 基础数据类型测试 ---
字符串编码/解码: Hello, World! -> 编码成功 -> Hello, World!
数字编码/解码: 42 -> 编码成功 -> 42
列表编码/解码: [1, 2, 3, 4, 5] -> 编码成功 -> [1, 2, 3, 4, 5]

--- 性能测试 ---
编码 1000 次，平均时间: 0.05ms
解码 1000 次，平均时间: 0.03ms

所有测试通过！
```

## 故障排除

### 导入错误

如果遇到 `ImportError: No module named 'rat_quickmem_py'`，请确保：

1. 已正确构建 Python 绑定
2. 在正确的 Python 环境中运行
3. 绑定库在 Python 路径中

### 构建错误

如果构建失败，请检查：

1. Rust 工具链是否正确安装
2. Python 开发头文件是否安装
3. maturin 版本是否兼容

### 运行时错误

如果运行时出现错误，请检查：

1. 输入数据是否符合限制
2. 内存是否充足
3. 权限是否正确

## 扩展示例

你可以基于这些示例创建自己的应用：

```python
import rat_quickmem_py as rmem

# 自定义数据处理
def process_data(data):
    try:
        # 编码数据
        encoded = rmem.encode(data)
        print(f"编码后大小: {len(encoded)} bytes")
        
        # 解码数据
        decoded = rmem.decode(encoded)
        return decoded
    except Exception as e:
        print(f"处理失败: {e}")
        return None

# 使用示例
result = process_data({"name": "Alice", "age": 30})
print(f"处理结果: {result}")
```

## 更多信息

- [rat_quickmem 主文档](../../README.md)
- [Python 绑定文档](../../python/README.md)
- [安全指南](../../SECURITY.md)