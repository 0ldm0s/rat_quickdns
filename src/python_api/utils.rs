//! Python API工具模块
//! 
//! 提供了一些实用的工具函数和辅助功能。

use pyo3::prelude::*;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

/// 验证IP地址格式
/// 
/// Args:
///     ip (str): 要验证的IP地址字符串
/// 
/// Returns:
///     bool: 如果是有效的IP地址返回True，否则返回False
/// 
/// Example:
///     >>> import rat_quickdns_py
///     >>> rat_quickdns_py.is_valid_ip("192.168.1.1")
///     True
///     >>> rat_quickdns_py.is_valid_ip("invalid-ip")
///     False
#[pyfunction]
pub fn is_valid_ip(ip: &str) -> bool {
    IpAddr::from_str(ip).is_ok()
}

/// 验证IPv4地址格式
/// 
/// Args:
///     ip (str): 要验证的IPv4地址字符串
/// 
/// Returns:
///     bool: 如果是有效的IPv4地址返回True，否则返回False
/// 
/// Example:
///     >>> import rat_quickdns_py
///     >>> rat_quickdns_py.is_valid_ipv4("192.168.1.1")
///     True
///     >>> rat_quickdns_py.is_valid_ipv4("2001:db8::1")
///     False
#[pyfunction]
pub fn is_valid_ipv4(ip: &str) -> bool {
    match IpAddr::from_str(ip) {
        Ok(IpAddr::V4(_)) => true,
        _ => false,
    }
}

/// 验证IPv6地址格式
/// 
/// Args:
///     ip (str): 要验证的IPv6地址字符串
/// 
/// Returns:
///     bool: 如果是有效的IPv6地址返回True，否则返回False
/// 
/// Example:
///     >>> import rat_quickdns_py
///     >>> rat_quickdns_py.is_valid_ipv6("2001:db8::1")
///     True
///     >>> rat_quickdns_py.is_valid_ipv6("192.168.1.1")
///     False
#[pyfunction]
pub fn is_valid_ipv6(ip: &str) -> bool {
    match IpAddr::from_str(ip) {
        Ok(IpAddr::V6(_)) => true,
        _ => false,
    }
}

/// 验证域名格式
/// 
/// Args:
///     domain (str): 要验证的域名字符串
/// 
/// Returns:
///     bool: 如果是有效的域名格式返回True，否则返回False
/// 
/// Example:
///     >>> import rat_quickdns_py
///     >>> rat_quickdns_py.is_valid_domain("example.com")
///     True
///     >>> rat_quickdns_py.is_valid_domain("invalid..domain")
///     False
#[pyfunction]
pub fn is_valid_domain(domain: &str) -> bool {
    // 基本的域名验证逻辑
    if domain.is_empty() || domain.len() > 253 {
        return false;
    }
    
    // 检查是否以点开头或结尾
    if domain.starts_with('.') || domain.ends_with('.') {
        return false;
    }
    
    // 检查是否包含连续的点
    if domain.contains("..") {
        return false;
    }
    
    // 检查每个标签
    for label in domain.split('.') {
        if label.is_empty() || label.len() > 63 {
            return false;
        }
        
        // 检查标签是否以连字符开头或结尾
        if label.starts_with('-') || label.ends_with('-') {
            return false;
        }
        
        // 检查字符是否有效（字母、数字、连字符）
        if !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return false;
        }
    }
    
    true
}

/// 验证Socket地址格式
/// 
/// Args:
///     address (str): 要验证的Socket地址字符串，格式为"IP:端口"
/// 
/// Returns:
///     bool: 如果是有效的Socket地址返回True，否则返回False
/// 
/// Example:
///     >>> import rat_quickdns_py
///     >>> rat_quickdns_py.is_valid_socket_addr("8.8.8.8:53")
///     True
///     >>> rat_quickdns_py.is_valid_socket_addr("invalid-address")
///     False
#[pyfunction]
pub fn is_valid_socket_addr(address: &str) -> bool {
    SocketAddr::from_str(address).is_ok()
}

/// 解析Socket地址
/// 
/// Args:
///     address (str): Socket地址字符串，格式为"IP:端口"
/// 
/// Returns:
///     tuple: (ip, port) 元组，如果解析失败返回None
/// 
/// Example:
///     >>> import rat_quickdns_py
///     >>> rat_quickdns_py.parse_socket_addr("8.8.8.8:53")
///     ('8.8.8.8', 53)
#[pyfunction]
pub fn parse_socket_addr(address: &str) -> pyo3::PyResult<Option<(String, u16)>> {
    match SocketAddr::from_str(address) {
        Ok(addr) => Ok(Some((addr.ip().to_string(), addr.port()))),
        Err(_) => Ok(None),
    }
}

/// 格式化响应时间
/// 
/// Args:
///     duration_ms (float): 响应时间（毫秒）
/// 
/// Returns:
///     str: 格式化后的响应时间字符串
/// 
/// Example:
///     >>> import rat_quickdns_py
///     >>> rat_quickdns_py.format_duration(1234.5)
///     '1.23s'
///     >>> rat_quickdns_py.format_duration(56.7)
///     '56.7ms'
#[pyfunction]
pub fn format_duration(duration_ms: f64) -> String {
    if duration_ms >= 1000.0 {
        format!("{:.2}s", duration_ms / 1000.0)
    } else {
        format!("{:.1}ms", duration_ms)
    }
}

/// 获取默认DNS服务器列表
/// 
/// Returns:
///     List[tuple]: 默认DNS服务器列表，每个元素是(name, address)元组
/// 
/// Example:
///     >>> import rat_quickdns_py
///     >>> servers = rat_quickdns_py.get_default_dns_servers()
///     >>> print(servers)
///     [('Google', '8.8.8.8:53'), ('Cloudflare', '1.1.1.1:53'), ...]
#[pyfunction]
pub fn get_default_dns_servers() -> Vec<(String, String)> {
    vec![
        ("Google Primary".to_string(), "8.8.8.8:53".to_string()),
        ("Google Secondary".to_string(), "8.8.4.4:53".to_string()),
        ("Cloudflare Primary".to_string(), "1.1.1.1:53".to_string()),
        ("Cloudflare Secondary".to_string(), "1.0.0.1:53".to_string()),
        ("Quad9 Primary".to_string(), "9.9.9.9:53".to_string()),
        ("Quad9 Secondary".to_string(), "149.112.112.112:53".to_string()),
        ("OpenDNS Primary".to_string(), "208.67.222.222:53".to_string()),
        ("OpenDNS Secondary".to_string(), "208.67.220.220:53".to_string()),
    ]
}

/// 获取默认DoH服务器列表
/// 
/// Returns:
///     List[tuple]: 默认DoH服务器列表，每个元素是(name, url)元组
/// 
/// Example:
///     >>> import rat_quickdns_py
///     >>> servers = rat_quickdns_py.get_default_doh_servers()
///     >>> print(servers)
///     [('Cloudflare', 'https://1.1.1.1/dns-query'), ...]
#[pyfunction]
pub fn get_default_doh_servers() -> Vec<(String, String)> {
    vec![
        ("Cloudflare".to_string(), "https://1.1.1.1/dns-query".to_string()),
        ("Google".to_string(), "https://8.8.8.8/dns-query".to_string()),
        ("Quad9".to_string(), "https://9.9.9.9/dns-query".to_string()),
        ("AdGuard".to_string(), "https://dns.adguard.com/dns-query".to_string()),
    ]
}

/// 获取默认DoT服务器列表
/// 
/// Returns:
///     List[tuple]: 默认DoT服务器列表，每个元素是(name, address, hostname)元组
/// 
/// Example:
///     >>> import rat_quickdns_py
///     >>> servers = rat_quickdns_py.get_default_dot_servers()
///     >>> print(servers)
///     [('Cloudflare', '1.1.1.1:853', 'one.one.one.one'), ...]
#[pyfunction]
pub fn get_default_dot_servers() -> Vec<(String, String, String)> {
    vec![
        ("Cloudflare".to_string(), "1.1.1.1:853".to_string(), "one.one.one.one".to_string()),
        ("Google".to_string(), "8.8.8.8:853".to_string(), "dns.google".to_string()),
        ("Quad9".to_string(), "9.9.9.9:853".to_string(), "dns.quad9.net".to_string()),
    ]
}

/// 快速解析单个域名（便捷函数）
/// 
/// Args:
///     domain (str): 要解析的域名
/// 
/// Returns:
///     List[str]: IP地址列表
/// 
/// Example:
///     >>> import rat_quickdns_py as dns
///     >>> ips = dns.quick_resolve("example.com")
///     >>> print(ips)
///     ['93.184.216.34']
#[pyfunction]
pub fn quick_resolve(py: Python, domain: &str) -> pyo3::PyResult<Vec<String>> {
    use crate::python_api::builder::PyDnsResolverBuilder;
    use crate::python_api::types::PyQueryStrategy;
    
    // 创建快速配置的解析器
    let mut builder = PyDnsResolverBuilder::new();
    builder.query_strategy(&PyQueryStrategy::FIFO);
    builder.add_udp_upstream("Cloudflare".to_string(), "1.1.1.1:53".to_string());
    builder.add_udp_upstream("Google".to_string(), "8.8.8.8:53".to_string());
    builder.timeout(3.0);
    
    let resolver = builder.build(py)?;
    resolver.resolve(py, domain)
}

/// 批量解析多个域名（便捷函数）
/// 
/// Args:
///     domains (List[str]): 要解析的域名列表
/// 
/// Returns:
///     List[DnsResult]: 解析结果列表
/// 
/// Example:
///     >>> import rat_quickdns_py as dns
///     >>> results = dns.batch_resolve(["google.com", "github.com"])
///     >>> for result in results:
///     ...     if result.is_ok():
///     ...         print(f"Success: {result.unwrap()}")
///     ...     else:
///     ...         print(f"Error: {result.unwrap_err()}")
#[pyfunction]
pub fn batch_resolve(py: Python, domains: Vec<String>) -> pyo3::PyResult<Vec<crate::python_api::types::PyDnsResult>> {
    use crate::python_api::builder::PyDnsResolverBuilder;
    use crate::python_api::types::PyQueryStrategy;
    
    // 创建快速配置的解析器
    let mut builder = PyDnsResolverBuilder::new();
    builder.query_strategy(&PyQueryStrategy::FIFO);
    builder.add_udp_upstream("Cloudflare".to_string(), "1.1.1.1:53".to_string());
    builder.add_udp_upstream("Google".to_string(), "8.8.8.8:53".to_string());
    builder.timeout(3.0);
    
    let resolver = builder.build(py)?;
    resolver.batch_resolve(py, domains)
}

/// 创建快速配置的解析器构建器
/// 
/// Args:
///     preset (str): 预设配置名称 ("fast", "secure", "balanced")
/// 
/// Returns:
///     DnsResolverBuilder: 预配置的构建器实例
/// 
/// Example:
///     >>> import rat_quickdns_py
///     >>> builder = rat_quickdns_py.create_preset_builder("fast")
///     >>> resolver = builder.build()
#[pyfunction]
pub fn create_preset_builder(preset: &str) -> pyo3::PyResult<crate::python_api::builder::PyDnsResolverBuilder> {
    use crate::python_api::builder::PyDnsResolverBuilder;
    use crate::python_api::types::PyQueryStrategy;
    
    let mut builder = PyDnsResolverBuilder::new();
    
    match preset {
        "fast" => {
            // 快速配置：使用最快优先策略，添加高性能DNS服务器
            builder.query_strategy(&PyQueryStrategy::FIFO);
            builder.add_udp_upstream("Cloudflare".to_string(), "1.1.1.1:53".to_string());
            builder.add_udp_upstream("Google".to_string(), "8.8.8.8:53".to_string());
            builder.timeout(3.0);
            builder.enable_edns(true);
        },
        "secure" => {
            // 安全配置：使用DoH/DoT协议
            builder.query_strategy(&PyQueryStrategy::SMART);
            builder.add_doh_upstream("Cloudflare".to_string(), "https://1.1.1.1/dns-query".to_string())?;
            builder.add_dot_upstream("Quad9".to_string(), "9.9.9.9:853".to_string());
            builder.timeout(5.0);
            builder.enable_edns(true);
            builder.enable_health_checker(true);
        },
        "balanced" => {
            // 平衡配置：混合使用多种协议和策略
            builder.query_strategy(&PyQueryStrategy::SMART);
            builder.add_udp_upstream("Cloudflare".to_string(), "1.1.1.1:53".to_string());
            builder.add_doh_upstream("Google".to_string(), "https://8.8.8.8/dns-query".to_string())?;
            builder.add_udp_upstream("Quad9".to_string(), "9.9.9.9:53".to_string());
            builder.timeout(4.0);
            builder.enable_edns(true);
            builder.enable_health_checker(true);
        },
        _ => {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Unknown preset '{}'. Available presets: fast, secure, balanced", preset)
            ));
        }
    }
    
    Ok(builder)
}