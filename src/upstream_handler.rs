//! 上游服务器处理器模块
//! 
//! 基于handler模式的上游服务器管理，避免强制类型转换，提供最优性能

use crate::{
    transport::{Transport, TransportConfig, HttpsConfig, TlsConfig},
    utils::{parse_server_address, parse_url_components, get_user_agent},
    Result, DnsError,
    dns_info, dns_debug,
};
use std::{
    collections::HashMap,
    time::Duration,
};
use async_trait::async_trait;

/// 上游服务器类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UpstreamType {
    /// UDP传输
    Udp,
    /// TCP传输
    Tcp,
    /// DNS over TLS
    DoT,
    /// DNS over HTTPS
    DoH,
}

/// 上游服务器配置（字符串存储）
#[derive(Debug, Clone)]
pub struct UpstreamSpec {
    /// 服务器名称
    pub name: String,
    /// 传输类型
    pub transport_type: UpstreamType,
    /// 服务器地址（统一字段：IP/域名/URL）
    pub server: String,
    /// 预解析的IP地址（可选，避免运行时解析）
    pub resolved_ip: Option<String>,
    /// 权重
    pub weight: u32,
    /// 期望区域
    pub region: Option<String>,
}

/// 上游处理器trait
#[async_trait]
pub trait UpstreamHandler: Send + Sync + std::fmt::Debug {
    /// 处理器类型
    fn handler_type(&self) -> UpstreamType;
    
    /// 从规格创建传输实例
    async fn create_transport(&self, spec: &UpstreamSpec) -> Result<Box<dyn Transport>>;
    
    /// 验证规格是否有效
    fn validate_spec(&self, spec: &UpstreamSpec) -> Result<()>;
    
    /// 获取默认端口
    fn default_port(&self) -> u16;
}

/// UDP处理器
#[derive(Debug, Default)]
pub struct UdpHandler;

#[async_trait]
impl UpstreamHandler for UdpHandler {
    fn handler_type(&self) -> UpstreamType {
        UpstreamType::Udp
    }
    
    async fn create_transport(&self, spec: &UpstreamSpec) -> Result<Box<dyn Transport>> {
        let (server, port) = parse_server_address(&spec.server, self.default_port())?;
        
        // 优先使用预解析的IP，避免运行时DNS查询
        let actual_server = spec.resolved_ip.as_ref().unwrap_or(&server);
        
        let config = TransportConfig {
            server: actual_server.clone(),
            port,
            timeout: Duration::from_secs(5),
            tcp_fast_open: false,
            tcp_nodelay: true,
            pool_size: 10,
        };
        
        Ok(Box::new(crate::transport::UdpTransport::new(config)))
    }
    
    fn validate_spec(&self, spec: &UpstreamSpec) -> Result<()> {
        if spec.server.is_empty() {
            return Err(DnsError::InvalidConfig("UDP server cannot be empty".to_string()));
        }
        Ok(())
    }
    
    fn default_port(&self) -> u16 {
        53
    }
}

/// TCP处理器
#[derive(Debug, Default)]
pub struct TcpHandler;

#[async_trait]
impl UpstreamHandler for TcpHandler {
    fn handler_type(&self) -> UpstreamType {
        UpstreamType::Tcp
    }
    
    async fn create_transport(&self, spec: &UpstreamSpec) -> Result<Box<dyn Transport>> {
        let (server, port) = parse_server_address(&spec.server, self.default_port())?;
        
        // 优先使用预解析的IP，避免运行时DNS查询
        let actual_server = spec.resolved_ip.as_ref().unwrap_or(&server);
        
        let config = TransportConfig {
            server: actual_server.clone(),
            port,
            timeout: Duration::from_secs(5),
            tcp_fast_open: false,
            tcp_nodelay: true,
            pool_size: 10,
        };
        
        Ok(Box::new(crate::transport::TcpTransport::new(config)))
    }
    
    fn validate_spec(&self, spec: &UpstreamSpec) -> Result<()> {
        if spec.server.is_empty() {
            return Err(DnsError::InvalidConfig("TCP server cannot be empty".to_string()));
        }
        Ok(())
    }
    
    fn default_port(&self) -> u16 {
        53
    }
}

/// DoT处理器
#[derive(Debug, Default)]
pub struct DoTHandler;

#[async_trait]
impl UpstreamHandler for DoTHandler {
    fn handler_type(&self) -> UpstreamType {
        UpstreamType::DoT
    }
    
    async fn create_transport(&self, spec: &UpstreamSpec) -> Result<Box<dyn Transport>> {
        let (server, port) = parse_server_address(&spec.server, self.default_port())?;
        
        // 对于DoT，连接地址优先使用预解析IP，但SNI必须使用原始域名
        let connection_server = spec.resolved_ip.as_ref().unwrap_or(&server);
        let sni_name = server.clone(); // SNI使用原始域名，确保证书验证正确
            
        let config = TlsConfig {
            base: TransportConfig {
                server: connection_server.clone(),
                port,
                timeout: Duration::from_secs(10),
                tcp_fast_open: false,
                tcp_nodelay: true,
                pool_size: 5,
            },
            server_name: sni_name,
            verify_cert: true,
        };
        
        Ok(Box::new(crate::transport::TlsTransport::new(config)?))
    }
    
    fn validate_spec(&self, spec: &UpstreamSpec) -> Result<()> {
        if spec.server.is_empty() {
            return Err(DnsError::InvalidConfig("DoT server cannot be empty".to_string()));
        }
        Ok(())
    }
    
    fn default_port(&self) -> u16 {
        853
    }
}

/// DoH处理器
#[derive(Debug, Default)]
pub struct DoHHandler;

#[async_trait]
impl UpstreamHandler for DoHHandler {
    fn handler_type(&self) -> UpstreamType {
        UpstreamType::DoH
    }
    
    async fn create_transport(&self, spec: &UpstreamSpec) -> Result<Box<dyn Transport>> {
        // 对于DoH，server字段应该是完整的HTTPS URL
        let url = &spec.server;
        
        // 从URL中提取主机名和端口
        let (hostname, port) = parse_url_components(url)?;
        
        // 连接地址优先使用预解析IP，但SNI必须使用原始域名
        let connection_server = spec.resolved_ip.as_ref().unwrap_or(&hostname);
        
        let config = HttpsConfig {
            base: TransportConfig {
                server: connection_server.clone(),
                port,
                timeout: Duration::from_secs(10),
                tcp_fast_open: false,
                tcp_nodelay: true,
                pool_size: 5,
            },
            url: url.clone(),
            method: crate::transport::HttpMethod::POST,
            user_agent: get_user_agent(),
        };
        
        Ok(Box::new(crate::transport::HttpsTransport::new(config)?))
    }
    
    fn validate_spec(&self, spec: &UpstreamSpec) -> Result<()> {
        if spec.server.is_empty() {
            return Err(DnsError::InvalidConfig("DoH server cannot be empty".to_string()));
        }
        
        // 验证URL格式
        if !spec.server.starts_with("https://") {
            return Err(DnsError::InvalidConfig("DoH URL must use HTTPS".to_string()));
        }
        
        Ok(())
    }
    
    fn default_port(&self) -> u16 {
        443
    }
}

/// 上游管理器
#[derive(Debug)]
pub struct UpstreamManager {
    handlers: HashMap<UpstreamType, Box<dyn UpstreamHandler>>,
    specs: Vec<UpstreamSpec>,
}

impl Clone for UpstreamManager {
    fn clone(&self) -> Self {
        let mut new_manager = Self::default();
        new_manager.specs = self.specs.clone();
        new_manager
    }
}

impl Default for UpstreamManager {
    fn default() -> Self {
        let mut handlers: HashMap<UpstreamType, Box<dyn UpstreamHandler>> = HashMap::new();
        handlers.insert(UpstreamType::Udp, Box::new(UdpHandler));
        handlers.insert(UpstreamType::Tcp, Box::new(TcpHandler));
        handlers.insert(UpstreamType::DoT, Box::new(DoTHandler));
        handlers.insert(UpstreamType::DoH, Box::new(DoHHandler));
        
        Self {
            handlers,
            specs: Vec::new(),
        }
    }
}

impl UpstreamManager {
    /// 创建新的管理器
    pub fn new() -> Self {
        Self::default()
    }
    
    /// 添加上游服务器
    pub fn add_upstream(&mut self, spec: UpstreamSpec) -> Result<()> {
        dns_info!("Adding upstream server: {} ({:?}) -> {}", spec.name, spec.transport_type, spec.server);
        
        // 验证规格
        if let Some(handler) = self.handlers.get(&spec.transport_type) {
            handler.validate_spec(&spec)?;
        } else {
            return Err(DnsError::InvalidConfig(
                format!("Unsupported transport type: {:?}", spec.transport_type)
            ));
        }
        
        self.specs.push(spec);
        dns_debug!("Successfully added upstream server, total count: {}", self.specs.len());
        Ok(())
    }
    
    /// 创建传输实例
    pub async fn create_transport(&self, spec: &UpstreamSpec) -> Result<Box<dyn Transport>> {
        if let Some(handler) = self.handlers.get(&spec.transport_type) {
            handler.create_transport(spec).await
        } else {
            Err(DnsError::InvalidConfig(
                format!("No handler for transport type: {:?}", spec.transport_type)
            ))
        }
    }
    
    /// 获取所有上游规格
    pub fn get_specs(&self) -> &[UpstreamSpec] {
        &self.specs
    }
    
    /// 按类型筛选上游
    pub fn filter_by_type(&self, transport_type: UpstreamType) -> Vec<&UpstreamSpec> {
        self.specs.iter()
            .filter(|spec| spec.transport_type == transport_type)
            .collect()
    }
}

// 解析函数已移至 crate::utils 模块，避免代码重复

/// 构建器辅助函数
impl UpstreamSpec {
    /// 创建UDP上游配置
    pub fn udp(name: String, server: String) -> Self {
        Self {
            name,
            transport_type: UpstreamType::Udp,
            server,
            resolved_ip: None,
            weight: 1,
            region: None,
        }
    }
    
    /// 创建TCP上游配置
    pub fn tcp(name: String, server: String) -> Self {
        Self {
            name,
            transport_type: UpstreamType::Tcp,
            server,
            resolved_ip: None,
            weight: 1,
            region: None,
        }
    }
    
    /// 创建DoT上游配置
    pub fn dot(name: String, server: String) -> Self {
        Self {
            name,
            transport_type: UpstreamType::DoT,
            server,
            resolved_ip: None,
            weight: 1,
            region: None,
        }
    }
    
    /// 创建DoH上游配置
    pub fn doh(name: String, url: String) -> Self {
        Self {
            name,
            transport_type: UpstreamType::DoH,
            server: url,
            resolved_ip: None,
            weight: 1,
            region: None,
        }
    }
    
    /// 设置预解析的IP地址
    pub fn with_resolved_ip(mut self, ip: String) -> Self {
        self.resolved_ip = Some(ip);
        self
    }
    
    /// 设置权重
    pub fn with_weight(mut self, weight: u32) -> Self {
        self.weight = weight;
        self
    }
    
    /// 设置区域
    pub fn with_region(mut self, region: String) -> Self {
        self.region = Some(region);
        self
    }
}