//! DNS传输层抽象

use crate::{Request, Response, Result};
use async_trait::async_trait;
use std::time::Duration;

pub mod udp;
pub mod tcp;
pub mod tls;
pub mod https;

pub use udp::UdpTransport;
pub use tcp::TcpTransport;
pub use tls::TlsTransport;
pub use https::HttpsTransport;

/// DNS传输层抽象接口
#[async_trait]
pub trait Transport: std::fmt::Debug + Send + Sync {
    /// 发送DNS请求并接收响应
    async fn send(&self, request: &Request) -> Result<Response>;
    
    /// 获取传输类型名称
    fn transport_type(&self) -> &'static str;
    
    /// 设置超时时间
    fn set_timeout(&mut self, timeout: Duration);
    
    /// 获取当前超时时间
    fn timeout(&self) -> Duration;
}

/// 传输配置
#[derive(Debug, Clone)]
pub struct TransportConfig {
    /// 服务器地址
    pub server: String,
    /// 端口
    pub port: u16,
    /// 超时时间
    pub timeout: Duration,
    /// 是否启用TCP快速打开
    pub tcp_fast_open: bool,
    /// 是否启用TCP无延迟
    pub tcp_nodelay: bool,
    /// 连接池大小
    pub pool_size: usize,
}

// 注意：移除了 Default 实现，因为它包含兜底行为
// 硬编码的默认值（如 "8.8.8.8"、端口53、5秒超时）是兜底代码
// 用户现在必须明确配置所有传输参数
//
// 迁移示例：
// 旧代码: TransportConfig::default()
// 新代码: TransportConfig {
//     server: "your-dns-server.com".to_string(),
//     port: 53,
//     timeout: Duration::from_secs(5),
//     tcp_fast_open: false,
//     tcp_nodelay: true,
//     pool_size: 10,
// }

/// HTTPS传输配置
#[derive(Debug, Clone)]
pub struct HttpsConfig {
    /// 基础传输配置
    pub base: TransportConfig,
    /// DoH服务器URL
    pub url: String,
    /// HTTP方法 (GET/POST)
    pub method: HttpMethod,
    /// 用户代理
    pub user_agent: String,
}

/// HTTP方法
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    /// GET方法
    GET,
    /// POST方法
    POST,
}

// 注意：移除了 Default 实现，因为它包含兜底行为
// 硬编码的默认值（如 cloudflare URL、POST方法）是兜底代码
// 用户现在必须明确配置所有HTTPS参数

/// TLS传输配置
#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// 基础传输配置
    pub base: TransportConfig,
    /// SNI服务器名称
    pub server_name: String,
    /// 是否验证证书
    pub verify_cert: bool,
}

// 注意：移除了 Default 实现，因为它包含兜底行为
// 硬编码的默认值（如 cloudflare 服务器名、端口853）是兜底代码
// 用户现在必须明确配置所有TLS参数