//! 通用工具函数模块
//! 
//! 提供跨模块共享的工具函数，避免代码重复

use crate::{Result, DnsError};
use url;

/// 解析服务器地址和端口
/// 
/// 支持以下格式：
/// - "example.com" -> ("example.com", default_port)
/// - "example.com:8080" -> ("example.com", 8080)
/// - "192.168.1.1:53" -> ("192.168.1.1", 53)
pub fn parse_server_address(server: &str, default_port: u16) -> Result<(String, u16)> {
    if let Some(colon_pos) = server.rfind(':') {
        let (addr, port_str) = server.split_at(colon_pos);
        let port = port_str[1..].parse::<u16>()
            .map_err(|_| DnsError::InvalidConfig(format!("Invalid port in server address: {}", server)))?;
        Ok((addr.to_string(), port))
    } else {
        Ok((server.to_string(), default_port))
    }
}

/// 从URL中解析主机名和端口
/// 
/// 支持HTTPS URL格式：
/// - "https://example.com/dns-query" -> ("example.com", 443)
/// - "https://example.com:8443/dns-query" -> ("example.com", 8443)
pub fn parse_url_components(url: &str) -> Result<(String, u16)> {
    let parsed = url::Url::parse(url)
        .map_err(|e| DnsError::InvalidConfig(format!("Invalid URL: {}", e)))?;
    
    let hostname = parsed.host_str()
        .ok_or_else(|| DnsError::InvalidConfig("URL must have hostname".to_string()))?
        .to_string();
    
    let port = parsed.port().unwrap_or(443);
    
    Ok((hostname, port))
}

/// 简化的服务器地址解析（使用简单的字符串分割）
/// 
/// 这是一个更简单的实现，用于替换resolver.rs中的内联解析逻辑
pub fn parse_simple_server_address(server: &str, default_port: u16) -> (String, u16) {
    if server.contains(':') {
        let parts: Vec<&str> = server.split(':').collect();
        let port = parts.get(1)
            .and_then(|p| p.parse().ok())
            .unwrap_or(default_port);
        (parts[0].to_string(), port)
    } else {
        (server.to_string(), default_port)
    }
}

/// 验证HTTPS URL格式
pub fn validate_https_url(url: &str) -> Result<()> {
    if url.is_empty() {
        return Err(DnsError::InvalidConfig("URL cannot be empty".to_string()));
    }
    
    if !url.starts_with("https://") {
        return Err(DnsError::InvalidConfig("URL must use HTTPS".to_string()));
    }
    
    // 尝试解析URL以验证格式
    url::Url::parse(url)
        .map_err(|e| DnsError::InvalidConfig(format!("Invalid URL format: {}", e)))?;
    
    Ok(())
}

/// 获取用户代理字符串
pub fn get_user_agent() -> String {
    format!("RatQuickDNS/{}", env!("CARGO_PKG_VERSION"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_server_address() {
        assert_eq!(parse_server_address("example.com", 53).unwrap(), ("example.com".to_string(), 53));
        assert_eq!(parse_server_address("example.com:8080", 53).unwrap(), ("example.com".to_string(), 8080));
        assert_eq!(parse_server_address("192.168.1.1:53", 53).unwrap(), ("192.168.1.1".to_string(), 53));
    }

    #[test]
    fn test_parse_url_components() {
        assert_eq!(parse_url_components("https://example.com/dns-query").unwrap(), ("example.com".to_string(), 443));
        assert_eq!(parse_url_components("https://example.com:8443/dns-query").unwrap(), ("example.com".to_string(), 8443));
    }

    #[test]
    fn test_parse_simple_server_address() {
        assert_eq!(parse_simple_server_address("example.com", 53), ("example.com".to_string(), 53));
        assert_eq!(parse_simple_server_address("example.com:8080", 53), ("example.com".to_string(), 8080));
    }

    #[test]
    fn test_validate_https_url() {
        assert!(validate_https_url("https://example.com/dns-query").is_ok());
        assert!(validate_https_url("http://example.com/dns-query").is_err());
        assert!(validate_https_url("").is_err());
    }

    #[test]
    fn test_get_user_agent() {
        let ua = get_user_agent();
        assert!(ua.starts_with("RatQuickDNS/"));
    }
}