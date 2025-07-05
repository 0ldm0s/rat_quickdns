//! Python API解析器模块
//! 
//! 提供了DnsResolver的Python绑定，用于执行DNS查询操作。

use pyo3::prelude::*;
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::builder::SmartDnsResolver;
use crate::builder::types::{DnsQueryRequest, DnsRecordType};
use crate::builder::strategy::QueryStrategy;
use super::types::{PyQueryStrategy, PyDnsResult, PyEmergencyResponseInfo};

/// Python版本的DNS解析器
/// 
/// 提供DNS查询功能，支持单个域名解析、批量解析等操作。
/// 解析器是线程安全的，可以在多个线程中共享使用。
/// 
/// Example:
///     >>> resolver = DnsResolverBuilder().build()
///     >>> ips = resolver.resolve("example.com")
///     >>> print(ips)
///     ['93.184.216.34']
#[pyclass(name = "DnsResolver")]
pub struct PyDnsResolver {
    inner: Arc<SmartDnsResolver>,
    runtime: Arc<Runtime>,
}

#[pymethods]
impl PyDnsResolver {
    /// 解析单个域名
    /// 
    /// Args:
    ///     domain (str): 要解析的域名
    /// 
    /// Returns:
    ///     List[str]: 解析得到的IP地址列表
    /// 
    /// Raises:
    ///     RuntimeError: 如果解析失败
    /// 
    /// Example:
    ///     >>> ips = resolver.resolve("google.com")
    ///     >>> print(ips)
    ///     ['142.250.191.14']
    pub fn resolve(&self, py: Python, domain: &str) -> pyo3::PyResult<Vec<String>> {
        let resolver = self.inner.clone();
        let domain = domain.to_string();
        
        py.allow_threads(|| {
            self.runtime.block_on(async move {
                let request = DnsQueryRequest::new(domain.clone(), DnsRecordType::A);
                let result = resolver.query(request).await;
                match result {
                    Ok(response) => Ok(response.ip_addresses().into_iter().map(|ip| ip.to_string()).collect()),
                    Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                        format!("DNS resolution failed for '{}': {}", domain, e)
                    )),
                }
            })
        })
    }
    
    /// 批量解析多个域名
    /// 
    /// Args:
    ///     domains (List[str]): 要解析的域名列表
    /// 
    /// Returns:
    ///     List[PyDnsResult]: 解析结果列表，每个元素对应一个域名的解析结果
    /// 
    /// Example:
    ///     >>> domains = ["google.com", "github.com", "invalid-domain.xyz"]
    ///     >>> results = resolver.batch_resolve(domains)
    ///     >>> for i, result in enumerate(results):
    ///     ...     if result.is_ok():
    ///     ...         print(f"{domains[i]}: {result.unwrap()}")
    ///     ...     else:
    ///     ...         print(f"{domains[i]}: Error - {result.unwrap_err()}")
    pub fn batch_resolve(&self, py: Python, domains: Vec<String>) -> pyo3::PyResult<Vec<PyDnsResult>> {
        let resolver = self.inner.clone();
        
        py.allow_threads(|| {
            self.runtime.block_on(async move {
                let mut results = Vec::new();
                for domain in domains {
                    let request = DnsQueryRequest::new(domain.clone(), DnsRecordType::A);
                    let result = resolver.query(request).await;
                    results.push((domain, result));
                }
                Ok(results.into_iter().map(|(domain, result)| {
                    match result {
                        Ok(response) => PyDnsResult::ok(response.ip_addresses().into_iter().map(|ip| ip.to_string()).collect()),
                        Err(e) => PyDnsResult::err(e.to_string()),
                    }
                }).collect())
            })
        })
    }
    
    /// 解析A记录（IPv4地址）
    /// 
    /// Args:
    ///     domain (str): 要解析的域名
    /// 
    /// Returns:
    ///     List[str]: IPv4地址列表
    /// 
    /// Raises:
    ///     RuntimeError: 如果解析失败
    /// 
    /// Example:
    ///     >>> ipv4_addrs = resolver.resolve_a("example.com")
    ///     >>> print(ipv4_addrs)
    ///     ['93.184.216.34']
    fn resolve_a(&self, py: Python, domain: &str) -> pyo3::PyResult<Vec<String>> {
        let resolver = self.inner.clone();
        let domain = domain.to_string();
        
        py.allow_threads(|| {
            self.runtime.block_on(async move {
                let request = DnsQueryRequest::new(domain.clone(), DnsRecordType::A);
                let result = resolver.query(request).await;
                match result {
                    Ok(response) => Ok(response.ip_addresses().into_iter().map(|ip| ip.to_string()).collect()),
                    Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                        format!("A record resolution failed for '{}': {}", domain, e)
                    )),
                }
            })
        })
    }
    
    /// 解析AAAA记录（IPv6地址）
    /// 
    /// Args:
    ///     domain (str): 要解析的域名
    /// 
    /// Returns:
    ///     List[str]: IPv6地址列表
    /// 
    /// Raises:
    ///     RuntimeError: 如果解析失败
    /// 
    /// Example:
    ///     >>> ipv6_addrs = resolver.resolve_aaaa("google.com")
    ///     >>> print(ipv6_addrs)
    ///     ['2404:6800:4008:c06::71']
    fn resolve_aaaa(&self, py: Python, domain: &str) -> pyo3::PyResult<Vec<String>> {
        let resolver = self.inner.clone();
        let domain = domain.to_string();
        
        py.allow_threads(|| {
            self.runtime.block_on(async move {
                let request = DnsQueryRequest::new(domain.clone(), DnsRecordType::AAAA);
                let result = resolver.query(request).await;
                match result {
                    Ok(response) => Ok(response.ip_addresses().into_iter().map(|ip| ip.to_string()).collect()),
                    Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                        format!("AAAA record resolution failed for '{}': {}", domain, e)
                    )),
                }
            })
        })
    }
    
    /// 解析CNAME记录
    /// 
    /// Args:
    ///     domain (str): 要解析的域名
    /// 
    /// Returns:
    ///     List[str]: CNAME记录列表
    /// 
    /// Raises:
    ///     RuntimeError: 如果解析失败
    /// 
    /// Example:
    ///     >>> cnames = resolver.resolve_cname("www.github.com")
    ///     >>> print(cnames)
    ///     ['github.com']
    fn resolve_cname(&self, py: Python, domain: &str) -> pyo3::PyResult<Vec<String>> {
        let resolver = self.inner.clone();
        let domain = domain.to_string();
        
        py.allow_threads(|| {
            self.runtime.block_on(async move {
                let request = DnsQueryRequest::new(domain.clone(), DnsRecordType::CNAME);
                let result = resolver.query(request).await;
                match result {
                    Ok(response) => Ok(response.domains()),
                    Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                        format!("CNAME record resolution failed for '{}': {}", domain, e)
                    )),
                }
            })
        })
    }
    
    /// 解析MX记录
    /// 
    /// Args:
    ///     domain (str): 要解析的域名
    /// 
    /// Returns:
    ///     List[str]: MX记录列表
    /// 
    /// Raises:
    ///     RuntimeError: 如果解析失败
    /// 
    /// Example:
    ///     >>> mx_records = resolver.resolve_mx("gmail.com")
    ///     >>> print(mx_records)
    ///     ['5 gmail-smtp-in.l.google.com', '10 alt1.gmail-smtp-in.l.google.com']
    fn resolve_mx(&self, py: Python, domain: &str) -> pyo3::PyResult<Vec<String>> {
        let resolver = self.inner.clone();
        let domain = domain.to_string();
        
        py.allow_threads(|| {
            self.runtime.block_on(async move {
                let request = DnsQueryRequest::new(domain.clone(), DnsRecordType::MX);
                let result = resolver.query(request).await;
                match result {
                    Ok(response) => Ok(response.mx_records().into_iter().map(|(priority, exchange)| {
                        format!("{} {}", priority, exchange)
                    }).collect()),
                    Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                        format!("MX record resolution failed for '{}': {}", domain, e)
                    )),
                }
            })
        })
    }
    
    /// 解析TXT记录
    /// 
    /// Args:
    ///     domain (str): 要解析的域名
    /// 
    /// Returns:
    ///     List[str]: TXT记录列表
    /// 
    /// Raises:
    ///     RuntimeError: 如果解析失败
    /// 
    /// Example:
    ///     >>> txt_records = resolver.resolve_txt("google.com")
    ///     >>> print(txt_records)
    ///     ['v=spf1 include:_spf.google.com ~all']
    fn resolve_txt(&self, py: Python, domain: &str) -> pyo3::PyResult<Vec<String>> {
        let resolver = self.inner.clone();
        let domain = domain.to_string();
        
        py.allow_threads(|| {
            self.runtime.block_on(async move {
                let request = DnsQueryRequest::new(domain.clone(), DnsRecordType::TXT);
                let result = resolver.query(request).await;
                match result {
                    Ok(response) => Ok(response.texts()),
                    Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                        format!("TXT record resolution failed for '{}': {}", domain, e)
                    )),
                }
            })
        })
    }
    
    /// 获取解析器统计信息
    /// 
    /// Returns:
    ///     dict: 包含查询统计、缓存命中率等信息的字典
    /// 
    /// Example:
    ///     >>> stats = resolver.get_stats()
    ///     >>> print(f"Total queries: {stats['total_queries']}")
    ///     >>> print(f"Cache hit rate: {stats['cache_hit_rate']:.2%}")
    fn get_stats(&self, py: Python) -> pyo3::PyResult<PyObject> {
        let resolver = self.inner.clone();
        let dict = pyo3::types::PyDict::new(py);
        
        let stats = py.allow_threads(|| {
            self.runtime.block_on(async move {
                resolver.get_stats().await
            })
        });
        
        dict.set_item("total_queries", stats.total_queries)?;
        dict.set_item("successful_queries", stats.successful_queries)?;
        dict.set_item("failed_queries", stats.failed_queries)?;
        dict.set_item("total_upstreams", stats.total_upstreams)?;
        dict.set_item("available_upstreams", stats.available_upstreams)?;
        dict.set_item("strategy", format!("{:?}", stats.strategy))?;
        dict.set_item("edns_enabled", stats.edns_enabled)?;
        
        let success_rate = stats.success_rate();
        dict.set_item("success_rate", success_rate)?;
        
        let avg_latency_ms = stats.avg_latency().as_millis() as f64;
        dict.set_item("avg_latency_ms", avg_latency_ms)?;
        
        if let Some(fastest) = &stats.fastest_upstream {
            dict.set_item("fastest_upstream", fastest)?;
        }
        
        if let Some(slowest) = &stats.slowest_upstream {
            dict.set_item("slowest_upstream", slowest)?;
        }
        
        Ok(dict.into())
    }
    
    /// 使用指定策略解析域名
    /// 
    /// Args:
    ///     domain (str): 要解析的域名
    ///     strategy (PyQueryStrategy): 查询策略
    /// 
    /// Returns:
    ///     PyDnsResult: 解析结果
    /// 
    /// Example:
    ///     >>> from easy_dns import QueryStrategy
    ///     >>> result = resolver.resolve_with_strategy("example.com", QueryStrategy.SMART)
    ///     >>> if result.is_ok():
    ///     ...     print(result.unwrap())
    fn resolve_with_strategy(&self, py: Python, domain: &str, strategy: PyQueryStrategy) -> pyo3::PyResult<PyDnsResult> {
        let resolver = self.inner.clone();
        let domain = domain.to_string();
        let strategy = strategy.to_rust();
        
        py.allow_threads(|| {
            self.runtime.block_on(async move {
                let request = DnsQueryRequest::new(domain.clone(), DnsRecordType::A);
                let result = resolver.query(request).await;
                Ok(match result {
                    Ok(response) => PyDnsResult::ok(response.ip_addresses().into_iter().map(|ip| ip.to_string()).collect()),
                    Err(e) => PyDnsResult::err(e.to_string()),
                })
            })
        })
    }
    
    /// 获取应急响应信息
     /// 
     /// Returns:
     ///     PyEmergencyResponseInfo: 应急响应信息对象，包含失败服务器信息和应急状态
     /// 
     /// Example:
     ///     >>> emergency_info = resolver.get_emergency_response_info()
     ///     >>> if emergency_info.all_servers_failed:
     ///     >>>     print(f"所有服务器都失败了: {emergency_info.emergency_message}")
     ///     >>>     for detail in emergency_info.get_failed_server_details():
     ///     >>>         print(f"  - {detail}")
     fn get_emergency_response_info(&self, py: Python) -> pyo3::PyResult<PyEmergencyResponseInfo> {
         let resolver = self.inner.clone();
         
         py.allow_threads(|| {
             self.runtime.block_on(async move {
                 if let Some(engine) = resolver.get_decision_engine() {
                     let info = engine.get_emergency_response_info().await;
                     Ok(PyEmergencyResponseInfo::from(&info))
                 } else {
                     // 没有决策引擎时返回默认信息
                     Ok(PyEmergencyResponseInfo {
                         all_servers_failed: false,
                         failed_servers: vec![],
                         last_working_server: None,
                         total_failures: 0,
                         emergency_message: "无决策引擎，使用基础解析器".to_string(),
                     })
                 }
             })
         })
     }
    
    /// 字符串表示
    fn __str__(&self) -> String {
        "DnsResolver".to_string()
    }
    
    /// 调试表示
    fn __repr__(&self) -> String {
        "DnsResolver()".to_string()
    }
}

impl PyDnsResolver {
    /// 创建新的Python DNS解析器实例
    pub fn new(resolver: SmartDnsResolver) -> Self {
        let runtime = Runtime::new().expect("Failed to create Tokio runtime");
        
        Self {
            inner: Arc::new(resolver),
            runtime: Arc::new(runtime),
        }
    }
    
    /// 获取内部解析器的引用
    pub fn inner(&self) -> &SmartDnsResolver {
        &self.inner
    }
}