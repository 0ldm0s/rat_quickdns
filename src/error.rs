//! 错误类型定义

use std::fmt;
use std::io;

/// DNS查询结果类型
pub type Result<T> = std::result::Result<T, DnsError>;

/// DNS错误类型
#[derive(Debug, Clone)]
pub enum DnsError {
    /// IO错误
    Io(String),
    /// 协议错误
    Protocol(String),
    /// 超时错误
    Timeout,
    /// 解析错误
    Parse(String),
    /// 网络错误
    Network(String),
    /// TLS错误
    Tls(String),
    /// HTTP错误
    Http(String),
    /// 配置错误
    Config(String),
    /// 无效配置
    InvalidConfig(String),
    /// 服务器错误
    Server(String),
    /// 域名不存在
    NxDomain,
    /// 查询被拒绝
    Refused,
    /// 服务器失败
    ServerFailure,
    /// 格式错误
    FormatError,
    /// 未实现
    NotImplemented(String),
    /// 无可用上游服务器
    NoUpstreamAvailable,
}

impl fmt::Display for DnsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DnsError::Io(msg) => write!(f, "IO error: {}", msg),
            DnsError::Protocol(msg) => write!(f, "Protocol error: {}", msg),
            DnsError::Timeout => write!(f, "Request timeout"),
            DnsError::Parse(msg) => write!(f, "Parse error: {}", msg),
            DnsError::Network(msg) => write!(f, "Network error: {}", msg),
            DnsError::Tls(msg) => write!(f, "TLS error: {}", msg),
            DnsError::Http(msg) => write!(f, "HTTP error: {}", msg),
            DnsError::Config(msg) => write!(f, "Config error: {}", msg),
            DnsError::InvalidConfig(msg) => write!(f, "Invalid config: {}", msg),
            DnsError::Server(msg) => write!(f, "Server error: {}", msg),
            DnsError::NxDomain => write!(f, "Domain not found"),
            DnsError::Refused => write!(f, "Query refused"),
            DnsError::ServerFailure => write!(f, "Server failure"),
            DnsError::FormatError => write!(f, "Format error"),
            DnsError::NotImplemented(msg) => write!(f, "Not implemented: {}", msg),
            DnsError::NoUpstreamAvailable => write!(f, "No upstream server available"),
        }
    }
}

impl std::error::Error for DnsError {}

impl From<io::Error> for DnsError {
    fn from(err: io::Error) -> Self {
        DnsError::Io(err.to_string())
    }
}

#[cfg(feature = "tokio-rustls")]
impl From<tokio_rustls::rustls::Error> for DnsError {
    fn from(err: tokio_rustls::rustls::Error) -> Self {
        DnsError::Tls(err.to_string())
    }
}

#[cfg(feature = "reqwest")]
impl From<reqwest::Error> for DnsError {
    fn from(err: reqwest::Error) -> Self {
        DnsError::Http(err.to_string())
    }
}