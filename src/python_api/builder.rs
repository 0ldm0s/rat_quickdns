//! Python API构建器模块
//! 
//! 提供了DnsResolverBuilder的Python绑定，用于配置和构建DNS解析器。

use pyo3::prelude::*;
use tokio::runtime::Runtime;

use crate::builder::DnsResolverBuilder as RustDnsResolverBuilder;
use crate::builder::strategy::QueryStrategy as RustQueryStrategy;
use crate::upstream_handler::{UpstreamSpec, UpstreamManager};
use super::resolver::PyDnsResolver;
use super::types::PyQueryStrategy;

/// Python版本的DNS解析器构建器
/// 
/// 用于配置和构建DNS解析器实例。支持链式调用，可以配置查询策略、
/// 上游DNS服务器、超时时间、EDNS功能等。
/// 
/// Example:
///     >>> builder = DnsResolverBuilder()
///     >>> builder.query_strategy(QueryStrategy.SMART)
///     >>> builder.add_udp_upstream("Cloudflare", "1.1.1.1:53", 100)
///     >>> builder.add_udp_upstream("Google", "8.8.8.8:53", 90)
///     >>> builder.timeout(5.0)
///     >>> resolver = builder.build()
#[pyclass(name = "DnsResolverBuilder")]
pub struct PyDnsResolverBuilder {
    inner: RustDnsResolverBuilder,
}

#[pymethods]
impl PyDnsResolverBuilder {
    /// 创建新的构建器实例
    /// 
    /// Returns:
    ///     DnsResolverBuilder: 新的构建器实例
    #[new]
    pub fn new() -> Self {
        // 为 Python API 提供合理的默认值
        let default_strategy = RustQueryStrategy::Smart;
        let default_edns = true;
        let default_region = "CN".to_string();
        
        Self {
            inner: RustDnsResolverBuilder::new(
                default_strategy,
                default_edns,
                default_region,
            ),
        }
    }
    
    /// 设置查询策略
    /// 
    /// Args:
    ///     strategy (QueryStrategy): 查询策略
    /// 
    /// Returns:
    ///     DnsResolverBuilder: 返回自身以支持链式调用
    /// 
    /// Example:
    ///     >>> builder.query_strategy(QueryStrategy.SMART)
    pub fn query_strategy(&mut self, strategy: &PyQueryStrategy) -> PyResult<()> {
        self.inner = self.inner.clone().query_strategy(strategy.to_rust());
        Ok(())
    }
    
    /// 添加UDP上游DNS服务器
    /// 
    /// Args:
    ///     name (str): 服务器名称（用于标识）
    ///     server (str): 服务器地址，支持"IP"、"IP:端口"格式
    /// 
    /// Returns:
    ///     DnsResolverBuilder: 返回自身以支持链式调用
    /// 
    /// Example:
    ///     >>> builder.add_udp_upstream("Google", "8.8.8.8")
    ///     >>> builder.add_udp_upstream("Cloudflare", "1.1.1.1")
    pub fn add_udp_upstream(&mut self, name: String, server: String) -> PyResult<()> {
        self.inner = self.inner.clone().add_udp_upstream(name, server);
        Ok(())
    }
    
    /// 添加TCP上游DNS服务器
    /// 
    /// Args:
    ///     name (str): 服务器名称（用于标识）
    ///     server (str): 服务器地址，支持"IP"、"IP:端口"格式
    /// 
    /// Returns:
    ///     DnsResolverBuilder: 返回自身以支持链式调用
    /// 
    /// Example:
    ///     >>> builder.add_tcp_upstream("Cloudflare", "1.1.1.1")
    ///     >>> builder.add_tcp_upstream("Quad9", "9.9.9.9")
    pub fn add_tcp_upstream(&mut self, name: String, server: String) -> PyResult<()> {
        self.inner = self.inner.clone().add_tcp_upstream(name, server);
        Ok(())
    }
    
    /// 添加DoH上游服务器
    /// 
    /// Args:
    ///     name (str): 服务器名称
    ///     url (str): DoH服务器URL (例如: "https://dns.google/dns-query")
    /// 
    /// Returns:
    ///     Self: 返回自身以支持链式调用
    /// 
    /// Raises:
    ///     ValueError: 当URL格式无效时
    /// 
    /// Example:
    ///     >>> builder.add_doh_upstream("Google DoH", "https://dns.google/dns-query")
    ///     >>> builder.add_doh_upstream("Cloudflare DoH", "https://cloudflare-dns.com/dns-query")
    pub fn add_doh_upstream(&mut self, name: String, url: String) -> PyResult<()> {
        // 验证URL格式
        if !url.starts_with("https://") {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "DoH URL must use HTTPS".to_string()
            ));
        }
        
        self.inner = self.inner.clone().add_doh_upstream(name, url);
        Ok(())
    }
    
    /// 添加DoT (DNS over TLS) 上游服务器
    /// 
    /// Args:
    ///     name (str): 服务器名称（用于标识）
    ///     server (str): 服务器地址，支持"IP"、"IP:端口"、"域名"格式
    /// 
    /// Returns:
    ///     DnsResolverBuilder: 返回自身以支持链式调用
    /// 
    /// Example:
    ///     >>> builder.add_dot_upstream("Quad9", "9.9.9.9")
    ///     >>> builder.add_dot_upstream("Cloudflare DoT", "1.1.1.1")
    pub fn add_dot_upstream(&mut self, name: String, server: String) -> PyResult<()> {
        self.inner = self.inner.clone().add_dot_upstream(name, server);
        Ok(())
    }
    
    /// 设置查询超时时间
    /// 
    /// Args:
    ///     timeout_secs (float): 超时时间（秒）
    /// 
    /// Returns:
    ///     DnsResolverBuilder: 返回自身以支持链式调用
    /// 
    /// Example:
    ///     >>> builder.timeout(5.0)  # 5秒超时
    pub fn timeout(&mut self, timeout_secs: f64) -> PyResult<()> {
        let duration = std::time::Duration::from_secs_f64(timeout_secs);
        self.inner = self.inner.clone().with_timeout(duration);
        Ok(())
    }
    
    /// 设置区域
    /// 
    /// Args:
    ///     region (str): 区域标识符
    /// 
    /// Returns:
    ///     DnsResolverBuilder: 返回自身以支持链式调用
    /// 
    /// Example:
    ///     >>> builder.region("CN")  # 中国区域
    pub fn region(&mut self, region: String) -> PyResult<()> {
        self.inner = self.inner.clone().region(region);
        Ok(())
    }
    
    /// 添加公共DNS服务器
    /// 
    /// Returns:
    ///     DnsResolverBuilder: 返回自身以支持链式调用
    /// 
    /// Example:
    ///     >>> builder.with_public_dns()
    pub fn with_public_dns(&mut self) -> PyResult<()> {
        self.inner = self.inner.clone().with_public_dns().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to add public DNS: {}", e))
        })?;
        Ok(())
    }
    
    /// 启用EDNS功能
    /// 
    /// Args:
    ///     enable (bool): 是否启用EDNS
    /// 
    /// Returns:
    ///     DnsResolverBuilder: 返回自身以支持链式调用
    /// 
    /// Example:
    ///     >>> builder.enable_edns(True)
    pub fn enable_edns(&mut self, enable: bool) -> PyResult<()> {
        self.inner = self.inner.clone().enable_edns(enable);
        Ok(())
    }
    
    /// 启用上游监控
    /// 
    /// Args:
    ///     enable (bool): 是否启用上游监控
    /// 
    /// Returns:
    ///     DnsResolverBuilder: 返回自身以支持链式调用
    /// 
    /// Example:
    ///     >>> builder.enable_upstream_monitoring(True)
    pub fn enable_upstream_monitoring(&mut self, enable: bool) -> PyResult<()> {
        self.inner = self.inner.clone().with_upstream_monitoring(enable);
        Ok(())
    }
    
    /// 启用健康检查器（上游监控的别名）
    /// 
    /// Args:
    ///     enable (bool): 是否启用健康检查
    /// 
    /// Returns:
    ///     DnsResolverBuilder: 返回自身以支持链式调用
    /// 
    /// Example:
    ///     >>> builder.enable_health_checker(True)
    pub fn enable_health_checker(&mut self, enable: bool) -> PyResult<()> {
        // 健康检查器实际上就是上游监控功能
        self.enable_upstream_monitoring(enable)
    }
    
    /// 为ROUND_ROBIN策略设置优化的超时时间
    /// 
    /// 轮询策略使用更短的超时时间以避免客户端等待过久
    /// 
    /// Args:
    ///     timeout_secs (float): 超时时间（秒），最大不超过2秒
    /// 
    /// Returns:
    ///     DnsResolverBuilder: 返回自身以支持链式调用
    /// 
    /// Example:
    ///     >>> builder.query_strategy(QueryStrategy.ROUND_ROBIN)
    ///     >>> builder.round_robin_timeout(1.5)  # 1.5秒超时
    pub fn round_robin_timeout(&mut self, timeout_secs: f64) -> PyResult<()> {
        let duration = std::time::Duration::from_secs_f64(timeout_secs);
        self.inner = self.inner.clone().with_round_robin_timeout(duration);
        Ok(())
    }
    
    /// 为ROUND_ROBIN策略应用性能优化配置
    /// 
    /// 包括：降低超时时间、启用健康检查、增加并发数、减少重试次数
    /// 
    /// Returns:
    ///     DnsResolverBuilder: 返回自身以支持链式调用
    /// 
    /// Example:
    ///     >>> builder.query_strategy(QueryStrategy.ROUND_ROBIN)
    ///     >>> builder.optimize_for_round_robin()  # 应用所有优化
    pub fn optimize_for_round_robin(&mut self) -> PyResult<()> {
        self.inner = self.inner.clone().optimize_for_round_robin();
        Ok(())
    }
    
    /// 启用调试级别日志
    /// 
    /// 启用调试级别日志，显示所有调试信息，包括DNS解析过程的详细信息
    /// 
    /// Returns:
    ///     DnsResolverBuilder: 返回自身以支持链式调用
    /// 
    /// Example:
    ///     >>> builder.with_debug_logger_init()
    pub fn with_debug_logger_init(&mut self) -> PyResult<()> {
        self.inner = self.inner.clone().with_debug_logger_init();
        Ok(())
    }
    
    /// 启用静默日志模式
    /// 
    /// 禁用所有日志输出，适用于生产环境或需要安静运行的场景
    /// 
    /// Returns:
    ///     DnsResolverBuilder: 返回自身以支持链式调用
    /// 
    /// Example:
    ///     >>> builder.with_silent_logger_init()
    pub fn with_silent_logger_init(&mut self) -> PyResult<()> {
        self.inner = self.inner.clone().with_silent_logger_init();
        Ok(())
    }
    
    /// 启用自动日志初始化
    /// 
    /// 根据环境自动选择合适的日志级别
    /// 
    /// Returns:
    ///     DnsResolverBuilder: 返回自身以支持链式调用
    /// 
    /// Example:
    ///     >>> builder.with_auto_logger_init()
    pub fn with_auto_logger_init(&mut self) -> PyResult<()> {
        self.inner = self.inner.clone().with_auto_logger_init();
        Ok(())
    }

    
    // 注意：上游监控间隔在当前版本中不支持配置
    // 上游监控功能通过 enable_upstream_monitoring 启用后会使用默认间隔
    
    /// 构建DNS解析器实例
    /// 
    /// Returns:
    ///     DnsResolver: 配置好的DNS解析器实例
    /// 
    /// Raises:
    ///     RuntimeError: 如果构建失败
    /// 
    /// Example:
    ///     >>> resolver = builder.build()
    pub fn build(&self, py: Python) -> pyo3::PyResult<PyDnsResolver> {
        py.allow_threads(|| {
            let rt = Runtime::new().map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                    format!("Failed to create runtime: {}", e)
                )
            })?;
            
            rt.block_on(async {
                let resolver = self.inner.clone().build().await.map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                        format!("Failed to build resolver: {}", e)
                    )
                })?;
                
                Ok(PyDnsResolver::new(resolver))
            })
        })
    }
    
    /// 字符串表示
    fn __str__(&self) -> String {
        "DnsResolverBuilder".to_string()
    }
    
    /// 调试表示
    fn __repr__(&self) -> String {
        "DnsResolverBuilder()".to_string()
    }
}

impl PyDnsResolverBuilder {
    /// 设置重试次数
    pub fn retries(&mut self, count: usize) -> PyResult<()> {
        self.inner = self.inner.clone().with_retry_count(count);
        Ok(())
    }
    
    /// 启用/禁用缓存
    pub fn cache(&mut self, enable: bool) -> PyResult<()> {
        self.inner = self.inner.clone().with_cache(enable);
        Ok(())
    }
    
    /// 设置DNS服务器端口
    pub fn port(&mut self, port: u16) -> PyResult<()> {
        self.inner = self.inner.clone().with_port(port);
        Ok(())
    }
    
    /// 设置并发查询数量
    pub fn concurrent_queries(&mut self, count: usize) -> PyResult<()> {
        self.inner = self.inner.clone().with_concurrent_queries(count);
        Ok(())
    }
    
    /// 启用/禁用递归查询
    pub fn recursion_desired(&mut self, enable: bool) -> PyResult<()> {
        self.inner = self.inner.clone().with_recursion(enable);
        Ok(())
    }
    
    /// 设置查询缓冲区大小
    pub fn buffer_size(&mut self, size: usize) -> PyResult<()> {
        self.inner = self.inner.clone().with_buffer_size(size);
        Ok(())
    }
    
    /// 获取内部构建器的引用
    pub fn inner(&self) -> &RustDnsResolverBuilder {
        &self.inner
    }
}