//! Python API类型定义
//! 
//! 定义了Python绑定中使用的各种类型和枚举。

use pyo3::prelude::*;
use crate::builder::strategy::QueryStrategy as RustQueryStrategy;
use crate::builder::engine::{FailedServerInfo, EmergencyResponseInfo};
use std::time::Instant;

/// Python绑定的查询策略枚举
/// 
/// 支持的查询策略：
/// - FIFO: 先进先出，按添加顺序查询上游服务器
/// - SMART: 智能决策，基于历史性能选择最优服务器
/// - ROUND_ROBIN: 轮询策略，依次使用不同的上游服务器
#[pyclass(name = "QueryStrategy")]
#[derive(Debug, Clone, Copy)]
pub enum PyQueryStrategy {
    /// 先进先出策略
    FIFO,
    /// 智能决策策略
    SMART,
    /// 轮询策略
    ROUND_ROBIN,
}

impl PyQueryStrategy {
    /// 转换为Rust内部的QueryStrategy
    pub fn to_rust(self) -> RustQueryStrategy {
        match self {
            PyQueryStrategy::FIFO => RustQueryStrategy::Fifo,
            PyQueryStrategy::SMART => RustQueryStrategy::Smart,
            PyQueryStrategy::ROUND_ROBIN => RustQueryStrategy::RoundRobin,
        }
    }
    
    /// 从Rust内部的QueryStrategy转换
    pub fn from_rust(strategy: RustQueryStrategy) -> Self {
        match strategy {
            RustQueryStrategy::Fifo => PyQueryStrategy::FIFO,
            RustQueryStrategy::Smart => PyQueryStrategy::SMART,
            RustQueryStrategy::RoundRobin => PyQueryStrategy::ROUND_ROBIN,
        }
    }
}

/// Python版本的结果类型
/// 
/// 类似于Rust的Result<T, E>，用于表示可能成功或失败的操作结果。
#[pyclass(name = "Result")]
#[derive(Clone, Debug)]
pub struct PyDnsResult {
    inner: Result<Vec<String>, String>,
}

#[pymethods]
impl PyDnsResult {
    /// 检查结果是否成功
    /// 
    /// Returns:
    ///     bool: 如果结果成功返回True，否则返回False
    fn is_ok(&self) -> bool {
        self.inner.is_ok()
    }
    
    /// 检查结果是否失败
    /// 
    /// Returns:
    ///     bool: 如果结果失败返回True，否则返回False
    fn is_err(&self) -> bool {
        self.inner.is_err()
    }
    
    /// 获取成功结果的值
    /// 
    /// Returns:
    ///     List[str]: 成功时返回IP地址列表
    /// 
    /// Raises:
    ///     RuntimeError: 如果结果是错误状态
    fn unwrap(&self) -> pyo3::PyResult<Vec<String>> {
        match &self.inner {
            Ok(value) => Ok(value.clone()),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                format!("Called unwrap on an Err value: {}", e)
            )),
        }
    }
    
    /// 获取错误结果的值
    /// 
    /// Returns:
    ///     str: 失败时返回错误信息
    /// 
    /// Raises:
    ///     RuntimeError: 如果结果是成功状态
    fn unwrap_err(&self) -> pyo3::PyResult<String> {
        match &self.inner {
            Ok(_) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "Called unwrap_err on an Ok value"
            )),
            Err(e) => Ok(e.clone()),
        }
    }
    
    /// 获取成功结果的值，如果失败则返回默认值
    /// 
    /// Args:
    ///     default (List[str]): 失败时返回的默认值
    /// 
    /// Returns:
    ///     List[str]: 成功时返回实际值，失败时返回默认值
    fn unwrap_or(&self, default: Vec<String>) -> Vec<String> {
        match &self.inner {
            Ok(value) => value.clone(),
            Err(_) => default,
        }
    }
    
    /// 字符串表示
    fn __str__(&self) -> String {
        match &self.inner {
            Ok(value) => format!("Ok({:?})", value),
            Err(e) => format!("Err(\"{}\")", e),
        }
    }
    
    /// 调试表示
    fn __repr__(&self) -> String {
        format!("Result({})", self.__str__())
    }
}

impl PyDnsResult {
    /// 创建成功结果
    pub fn ok(value: Vec<String>) -> Self {
        Self {
            inner: Ok(value),
        }
    }
    
    /// 创建失败结果
    pub fn err(error: String) -> Self {
        Self {
            inner: Err(error),
        }
    }
    
    /// 从Rust Result转换
    pub fn from_rust_result(result: Result<Vec<std::net::IpAddr>, crate::error::DnsError>) -> Self {
        match result {
            Ok(ips) => Self::ok(ips.into_iter().map(|ip| ip.to_string()).collect()),
            Err(e) => Self::err(e.to_string()),
        }
    }
}

/// Python绑定的传输协议类型
/// 
/// 支持的传输协议：
/// - UDP: 标准UDP协议
/// - TCP: 标准TCP协议  
/// - DOH: DNS over HTTPS
/// - DOT: DNS over TLS
#[pyclass(name = "TransportType")]
#[derive(Debug, Clone, Copy)]
pub enum PyTransportType {
    /// UDP传输协议
    UDP,
    /// TCP传输协议
    TCP,
    /// DNS over HTTPS
    DOH,
    /// DNS over TLS
    DOT,
}

/// Python绑定的DNS记录类型
#[pyclass(name = "DnsRecordType")]
#[derive(Debug, Clone, Copy)]
pub enum PyDnsRecordType {
    /// A记录 - IPv4地址
    A,
    /// AAAA记录 - IPv6地址
    AAAA,
    /// CNAME记录 - 别名
    CNAME,
    /// MX记录 - 邮件交换
    MX,
    /// TXT记录 - 文本记录
    TXT,
    /// NS记录 - 名称服务器
    NS,
    /// PTR记录 - 指针记录
    PTR,
    /// SOA记录 - 授权开始
    SOA,
    /// SRV记录 - 服务记录
    SRV,
}

/// Python绑定的失败服务器信息
#[pyclass]
#[derive(Debug, Clone)]
pub struct PyFailedServerInfo {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub server: String,
    #[pyo3(get)]
    pub consecutive_failures: u32,
    #[pyo3(get)]
    pub failure_reason: String,
    #[pyo3(get)]
    pub last_failure_seconds_ago: Option<f64>,
}

#[pymethods]
impl PyFailedServerInfo {
    fn __str__(&self) -> String {
        format!(
            "FailedServer(name='{}', server='{}', failures={}, reason='{}')",
            self.name, self.server, self.consecutive_failures, self.failure_reason
        )
    }
    
    fn __repr__(&self) -> String {
        self.__str__()
    }
}

impl From<&FailedServerInfo> for PyFailedServerInfo {
    fn from(info: &FailedServerInfo) -> Self {
        let last_failure_seconds_ago = info.last_failure_time
            .map(|time| Instant::now().duration_since(time).as_secs_f64());
            
        PyFailedServerInfo {
            name: info.name.clone(),
            server: info.server.clone(),
            consecutive_failures: info.consecutive_failures,
            failure_reason: info.failure_reason.clone(),
            last_failure_seconds_ago,
        }
    }
}

/// Python绑定的应急响应信息
#[pyclass]
#[derive(Debug, Clone)]
pub struct PyEmergencyResponseInfo {
    #[pyo3(get)]
    pub all_servers_failed: bool,
    #[pyo3(get)]
    pub failed_servers: Vec<PyFailedServerInfo>,
    #[pyo3(get)]
    pub last_working_server: Option<String>,
    #[pyo3(get)]
    pub total_failures: u32,
    #[pyo3(get)]
    pub emergency_message: String,
}

#[pymethods]
impl PyEmergencyResponseInfo {
    fn __str__(&self) -> String {
        format!(
            "EmergencyResponse(all_failed={}, servers={}, total_failures={}, message='{}')",
            self.all_servers_failed,
            self.failed_servers.len(),
            self.total_failures,
            self.emergency_message
        )
    }
    
    fn __repr__(&self) -> String {
        self.__str__()
    }
    
    /// 获取失败服务器的详细信息
    fn get_failed_server_details(&self) -> Vec<String> {
        self.failed_servers.iter()
            .map(|server| format!(
                "{}: {} failures, last failure: {}",
                server.name,
                server.consecutive_failures,
                server.last_failure_seconds_ago
                    .map(|s| format!("{:.1}s ago", s))
                    .unwrap_or_else(|| "unknown".to_string())
            ))
            .collect()
    }
}

impl From<&EmergencyResponseInfo> for PyEmergencyResponseInfo {
    fn from(info: &EmergencyResponseInfo) -> Self {
        PyEmergencyResponseInfo {
            all_servers_failed: info.all_servers_failed,
            failed_servers: info.failed_servers.iter().map(PyFailedServerInfo::from).collect(),
            last_working_server: info.last_working_server.clone(),
            total_failures: info.total_failures,
            emergency_message: info.emergency_message.clone(),
        }
    }
}

#[pymethods]
impl PyDnsRecordType {
    /// 转换为字符串
    pub fn to_string(&self) -> String {
        match self {
            PyDnsRecordType::A => "A".to_string(),
            PyDnsRecordType::AAAA => "AAAA".to_string(),
            PyDnsRecordType::CNAME => "CNAME".to_string(),
            PyDnsRecordType::MX => "MX".to_string(),
            PyDnsRecordType::TXT => "TXT".to_string(),
            PyDnsRecordType::NS => "NS".to_string(),
            PyDnsRecordType::PTR => "PTR".to_string(),
            PyDnsRecordType::SOA => "SOA".to_string(),
            PyDnsRecordType::SRV => "SRV".to_string(),
        }
    }
    
    /// 从字符串创建
    #[staticmethod]
    pub fn from_string(s: &str) -> pyo3::PyResult<PyDnsRecordType> {
        match s.to_uppercase().as_str() {
            "A" => Ok(PyDnsRecordType::A),
            "AAAA" => Ok(PyDnsRecordType::AAAA),
            "CNAME" => Ok(PyDnsRecordType::CNAME),
            "MX" => Ok(PyDnsRecordType::MX),
            "TXT" => Ok(PyDnsRecordType::TXT),
            "NS" => Ok(PyDnsRecordType::NS),
            "PTR" => Ok(PyDnsRecordType::PTR),
            "SOA" => Ok(PyDnsRecordType::SOA),
            "SRV" => Ok(PyDnsRecordType::SRV),
            _ => Err(pyo3::exceptions::PyValueError::new_err(
                format!("Unsupported DNS record type: {}", s)
            )),
        }
    }
    

   }

impl PyDnsRecordType {
    /// 从Rust类型创建（内部使用）
    pub fn from_rust(record_type: crate::builder::types::DnsRecordType) -> Self {
        match record_type {
            crate::builder::types::DnsRecordType::A => PyDnsRecordType::A,
            crate::builder::types::DnsRecordType::AAAA => PyDnsRecordType::AAAA,
            crate::builder::types::DnsRecordType::CNAME => PyDnsRecordType::CNAME,
            crate::builder::types::DnsRecordType::MX => PyDnsRecordType::MX,
            crate::builder::types::DnsRecordType::TXT => PyDnsRecordType::TXT,
            crate::builder::types::DnsRecordType::NS => PyDnsRecordType::NS,
            crate::builder::types::DnsRecordType::PTR => PyDnsRecordType::PTR,
            crate::builder::types::DnsRecordType::SOA => PyDnsRecordType::SOA,
            crate::builder::types::DnsRecordType::SRV => PyDnsRecordType::SRV,
        }
    }
    
    /// 转换为Rust类型（内部使用）
    pub fn to_rust(&self) -> crate::builder::types::DnsRecordType {
        match self {
            PyDnsRecordType::A => crate::builder::types::DnsRecordType::A,
            PyDnsRecordType::AAAA => crate::builder::types::DnsRecordType::AAAA,
            PyDnsRecordType::CNAME => crate::builder::types::DnsRecordType::CNAME,
            PyDnsRecordType::MX => crate::builder::types::DnsRecordType::MX,
            PyDnsRecordType::TXT => crate::builder::types::DnsRecordType::TXT,
            PyDnsRecordType::NS => crate::builder::types::DnsRecordType::NS,
            PyDnsRecordType::PTR => crate::builder::types::DnsRecordType::PTR,
            PyDnsRecordType::SOA => crate::builder::types::DnsRecordType::SOA,
            PyDnsRecordType::SRV => crate::builder::types::DnsRecordType::SRV,
        }
    }
}