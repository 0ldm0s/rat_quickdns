//! PyO3集成示例
//! 
//! 演示如何使用PyO3将rat_quickdns暴露给Python
//! 注意：此示例需要在Cargo.toml中添加pyo3依赖

use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyList};
use rat_quickdns::{
    EasyDnsResolver, DnsQueryRequest, DnsQueryResponse,
    create_dns_query, encode_dns_query, decode_dns_response
};
use std::collections::HashMap;
use tokio::runtime::Runtime;

/// Python包装的DNS解析器
#[pyclass]
struct PyDnsResolver {
    resolver: EasyDnsResolver,
    runtime: Runtime,
}

#[pymethods]
impl PyDnsResolver {
    /// 创建新的DNS解析器
    #[new]
    fn new() -> PyResult<Self> {
        let resolver = EasyDnsResolver::quick_setup()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create resolver: {}", e)))?;
        
        let runtime = Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create runtime: {}", e)))?;
        
        Ok(Self { resolver, runtime })
    }
    
    /// 解析域名（返回IP地址列表）
    fn resolve(&self, domain: &str) -> PyResult<Vec<String>> {
        self.runtime.block_on(async {
            self.resolver.resolve(domain).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Resolution failed: {}", e)))
    }
    
    /// 解析特定记录类型
    fn resolve_type(&self, domain: &str, record_type: &str) -> PyResult<Vec<String>> {
        self.runtime.block_on(async {
            self.resolver.resolve_type(domain, record_type).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Resolution failed: {}", e)))
    }
    
    /// 批量解析域名
    fn batch_resolve(&self, domains: Vec<String>) -> PyResult<HashMap<String, Vec<String>>> {
        let mut results = HashMap::new();
        
        for domain in domains {
            match self.runtime.block_on(async {
                self.resolver.resolve(&domain).await
            }) {
                Ok(ips) => { results.insert(domain, ips); },
                Err(e) => return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                    format!("Failed to resolve {}: {}", domain, e)
                )),
            }
        }
        
        Ok(results)
    }
    
    /// 处理二进制查询（使用bincode2）
    fn process_binary_query(&self, py: Python, query_data: &PyBytes) -> PyResult<PyObject> {
        let data = query_data.as_bytes();
        
        let response_data = self.runtime.block_on(async {
            self.resolver.process_encoded_query(data).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Query processing failed: {}", e)))?;
        
        Ok(PyBytes::new(py, &response_data).into())
    }
    
    /// 创建查询请求（返回二进制数据）
    fn create_query(&self, py: Python, domain: &str, record_type: &str) -> PyResult<PyObject> {
        let request = create_dns_query(domain, record_type);
        
        let encoded = encode_dns_query(&request)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Encoding failed: {}", e)))?;
        
        Ok(PyBytes::new(py, &encoded).into())
    }
    
    /// 解码响应数据
    fn decode_response(&self, response_data: &PyBytes) -> PyResult<PyDnsResponse> {
        let data = response_data.as_bytes();
        
        let response = decode_dns_response(data)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Decoding failed: {}", e)))?;
        
        Ok(PyDnsResponse::from(response))
    }
}

/// Python包装的DNS响应
#[pyclass]
#[derive(Clone)]
struct PyDnsResponse {
    #[pyo3(get)]
    success: bool,
    #[pyo3(get)]
    query_id: String,
    #[pyo3(get)]
    domain: String,
    #[pyo3(get)]
    record_type: String,
    #[pyo3(get)]
    records: Vec<PyDnsRecord>,
    #[pyo3(get)]
    duration_ms: u64,
    #[pyo3(get)]
    error: Option<String>,
}

impl From<DnsQueryResponse> for PyDnsResponse {
    fn from(response: DnsQueryResponse) -> Self {
        Self {
            success: response.success,
            query_id: response.query_id,
            domain: response.domain,
            record_type: response.record_type,
            records: response.records.into_iter().map(PyDnsRecord::from).collect(),
            duration_ms: response.duration_ms,
            error: response.error,
        }
    }
}

/// Python包装的DNS记录
#[pyclass]
#[derive(Clone)]
struct PyDnsRecord {
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    record_type: String,
    #[pyo3(get)]
    value: String,
    #[pyo3(get)]
    ttl: u32,
}

impl From<rat_quickdns::DnsRecord> for PyDnsRecord {
    fn from(record: rat_quickdns::DnsRecord) -> Self {
        Self {
            name: record.name,
            record_type: record.record_type,
            value: record.value,
            ttl: record.ttl,
        }
    }
}

/// 便捷函数：快速解析域名
#[pyfunction]
fn quick_resolve(domain: &str) -> PyResult<Vec<String>> {
    let resolver = EasyDnsResolver::quick_setup()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create resolver: {}", e)))?;
    
    let runtime = Runtime::new()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create runtime: {}", e)))?;
    
    runtime.block_on(async {
        resolver.resolve(domain).await
    }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Resolution failed: {}", e)))
}

/// 便捷函数：快速解析特定记录类型
#[pyfunction]
fn quick_resolve_type(domain: &str, record_type: &str) -> PyResult<Vec<String>> {
    let resolver = EasyDnsResolver::quick_setup()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create resolver: {}", e)))?;
    
    let runtime = Runtime::new()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create runtime: {}", e)))?;
    
    runtime.block_on(async {
        resolver.resolve_type(domain, record_type).await
    }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Resolution failed: {}", e)))
}

/// Python模块定义
#[pymodule]
fn rat_quickdns_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyDnsResolver>()?;
    m.add_class::<PyDnsResponse>()?;
    m.add_class::<PyDnsRecord>()?;
    m.add_function(wrap_pyfunction!(quick_resolve, m)?)?;
    m.add_function(wrap_pyfunction!(quick_resolve_type, m)?)?;
    Ok(())
}

// Python使用示例（注释形式）
/*
Python使用示例：

```python
import rat_quickdns_py

# 方式1：使用便捷函数
ips = rat_quickdns_py.quick_resolve("example.com")
print(f"example.com IPs: {ips}")

mx_records = rat_quickdns_py.quick_resolve_type("gmail.com", "MX")
print(f"gmail.com MX records: {mx_records}")

# 方式2：使用解析器类
resolver = rat_quickdns_py.PyDnsResolver()

# 单个域名解析
ips = resolver.resolve("github.com")
print(f"github.com IPs: {ips}")

# 批量解析
domains = ["baidu.com", "qq.com", "taobao.com"]
results = resolver.batch_resolve(domains)
for domain, ips in results.items():
    print(f"{domain}: {ips}")

# 二进制查询（高性能）
query_data = resolver.create_query("example.com", "A")
response_data = resolver.process_binary_query(query_data)
response = resolver.decode_response(response_data)

print(f"Query ID: {response.query_id}")
print(f"Success: {response.success}")
print(f"Duration: {response.duration_ms}ms")
for record in response.records:
    print(f"  {record.name} {record.record_type} {record.value} (TTL: {record.ttl})")
```

构建说明：
1. 在Cargo.toml中添加：
   [dependencies]
   pyo3 = { version = "0.20", features = ["extension-module"] }
   
2. 添加crate-type：
   [lib]
   crate-type = ["cdylib"]
   
3. 构建Python扩展：
   maturin develop
   
4. 或使用setuptools-rust进行分发
*/

fn main() {
    println!("这是PyO3集成示例代码");
    println!("请参考代码注释了解如何构建Python扩展");
}