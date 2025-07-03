# rat-quickdns-py 开发指南

本文档介绍如何开发和维护 rat-quickdns-py Python 绑定。

## 项目结构

```
python/
├── src/
│   └── lib.rs              # Python绑定入口
├── examples/
│   ├── basic_usage.py      # 基本使用示例
│   └── performance_test.py # 性能测试示例
├── tests/
│   └── test_resolver.py    # 单元测试
├── Cargo.toml              # Rust项目配置
├── pyproject.toml          # Python项目配置
├── build.py                # 构建脚本
├── README.md               # 项目说明
└── DEVELOPMENT.md          # 开发指南（本文件）
```

## 开发环境设置

### 1. 安装依赖

#### Rust 工具链
```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 验证安装
rustc --version
cargo --version
```

#### Python 依赖
```bash
# 安装 Maturin（用于构建 Python 扩展）
pip install maturin

# 安装开发依赖
pip install pytest pytest-benchmark dnspython
```

### 2. 构建项目

#### 使用构建脚本（推荐）
```bash
# 进入 python 目录
cd python

# 检查环境并构建、安装、测试
python build.py all

# 或者分步执行
python build.py build      # 构建
python build.py install    # 安装
python build.py test       # 测试
```

#### 手动构建
```bash
# 开发模式安装（推荐用于开发）
maturin develop --features python-bindings

# 构建 wheel 包
maturin build --release --features python-bindings

# 安装构建好的包
pip install target/wheels/rat_quickdns_py-*.whl
```

## 代码结构说明

### Rust 侧代码

#### 主模块 (`src/python_api/mod.rs`)
- 定义 Python 模块入口
- 导出主要类和函数
- 提供便捷函数

#### 类型定义 (`src/python_api/types.rs`)
- `PyQueryStrategy`: 查询策略枚举
- `PyResult`: 结果类型
- `PyTransportType`: 传输协议类型

#### 构建器 (`src/python_api/builder.rs`)
- `PyDnsResolverBuilder`: DNS解析器构建器
- 支持链式调用配置

#### 解析器 (`src/python_api/resolver.rs`)
- `PyDnsResolver`: DNS解析器主类
- 提供各种DNS记录类型的解析方法

#### 工具函数 (`src/python_api/utils.rs`)
- IP地址验证
- 域名验证
- 预设配置
- 默认服务器列表

### Python 侧代码

#### 示例代码 (`examples/`)
- `basic_usage.py`: 展示基本功能
- `performance_test.py`: 性能测试和对比

#### 测试代码 (`tests/`)
- `test_resolver.py`: 单元测试
- 覆盖主要功能和边界情况

## 开发工作流

### 1. 添加新功能

1. **在 Rust 侧实现功能**
   ```rust
   // 在相应的模块中添加新方法
   #[pymethods]
   impl PyDnsResolver {
       fn new_method(&self, param: &str) -> PyResult<String> {
           // 实现逻辑
       }
   }
   ```

2. **更新 Python 绑定**
   ```rust
   // 在 mod.rs 中导出新功能（如果需要）
   m.add_function(wrap_pyfunction!(new_function, m)?)?
   ```

3. **添加测试**
   ```python
   # 在 tests/test_resolver.py 中添加测试
   def test_new_method(self):
       result = self.resolver.new_method("test")
       self.assertIsInstance(result, str)
   ```

4. **更新文档和示例**
   - 更新 README.md
   - 在示例中展示新功能

### 2. 性能优化

1. **使用性能测试**
   ```bash
   python examples/performance_test.py
   ```

2. **分析瓶颈**
   - 使用 Rust 的 `cargo flamegraph`
   - 使用 Python 的 `cProfile`

3. **优化策略**
   - 减少 Python-Rust 边界的数据转换
   - 使用批量操作
   - 优化内存分配

### 3. 错误处理

#### Rust 侧错误处理
```rust
// 将 Rust 错误转换为 Python 异常
fn some_function() -> PyResult<String> {
    let result = rust_function().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            format!("Operation failed: {}", e)
        )
    })?;
    Ok(result)
}
```

#### Python 侧错误处理
```python
# 捕获和处理异常
try:
    result = resolver.resolve("example.com")
except RuntimeError as e:
    print(f"DNS resolution failed: {e}")
```

## 测试策略

### 1. 单元测试
```bash
# 运行所有测试
python build.py test

# 运行特定测试
python -m unittest tests.test_resolver.TestResolver.test_resolve
```

### 2. 性能测试
```bash
# 运行性能测试
python examples/performance_test.py
```

### 3. 集成测试
```bash
# 运行示例作为集成测试
python examples/basic_usage.py
```

## 发布流程

### 1. 版本更新
```toml
# 更新 Cargo.toml
[package]
version = "0.2.0"

# 更新 pyproject.toml
[project]
version = "0.2.0"
```

### 2. 构建发布包
```bash
# 构建所有平台的 wheel
maturin build --release --features python-bindings

# 或使用 CI/CD 自动构建
```

### 3. 发布到 PyPI
```bash
# 上传到 PyPI
maturin publish --features python-bindings
```

## 调试技巧

### 1. Rust 侧调试
```rust
// 使用 println! 调试
println!("Debug: value = {:?}", value);

// 使用 log crate
log::debug!("Debug message: {}", value);
```

### 2. Python 侧调试
```python
# 使用 print 调试
print(f"Debug: value = {value}")

# 使用 logging
import logging
logging.basicConfig(level=logging.DEBUG)
logging.debug(f"Debug message: {value}")
```

### 3. 内存调试
```bash
# 使用 valgrind（Linux）
valgrind --tool=memcheck python examples/basic_usage.py

# 使用 AddressSanitizer
RUSTFLAGS="-Z sanitizer=address" cargo build
```

## 常见问题

### 1. 编译错误

**问题**: `pyo3` 版本不兼容
```
error: failed to resolve dependencies
```

**解决**: 更新 `Cargo.toml` 中的 `pyo3` 版本

### 2. 运行时错误

**问题**: 找不到动态库
```
ImportError: dynamic module does not define module export function
```

**解决**: 重新构建并安装
```bash
python build.py clean
python build.py all
```

### 3. 性能问题

**问题**: Python 绑定比纯 Rust 慢

**解决**:
- 减少 Python-Rust 调用频率
- 使用批量操作
- 避免频繁的数据转换

## 贡献指南

1. **Fork 项目**
2. **创建功能分支**
   ```bash
   git checkout -b feature/new-feature
   ```
3. **实现功能并添加测试**
4. **确保所有测试通过**
   ```bash
   python build.py test
   ```
5. **提交 Pull Request**

## 参考资源

- [PyO3 用户指南](https://pyo3.rs/)
- [Maturin 文档](https://maturin.rs/)
- [Rust 异步编程](https://rust-lang.github.io/async-book/)
- [Python C 扩展指南](https://docs.python.org/3/extending/)