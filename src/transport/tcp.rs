//! TCP传输实现

use crate::{Request, Response, Result, DnsError};
use super::{Transport, TransportConfig};
use super::udp::UdpTransport;
use async_trait::async_trait;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::timeout;
use crate::{dns_debug, dns_info, dns_error, dns_transport};

/// TCP传输实现
#[derive(Debug)]
pub struct TcpTransport {
    config: TransportConfig,
}

impl TcpTransport {
    /// 创建新的TCP传输
    pub fn new(config: TransportConfig) -> Self {
        Self { config }
    }
    
    /// 使用默认配置创建TCP传输
    pub fn default() -> Self {
        Self::new(TransportConfig::default())
    }
    
    /// 序列化DNS请求为TCP格式(带长度前缀)
    fn serialize_request_tcp(request: &Request) -> Result<Vec<u8>> {
        // 复用UDP的序列化逻辑
        let udp_data = UdpTransport::serialize_request(request)?;
        
        // TCP格式需要2字节长度前缀
        let mut tcp_data = Vec::with_capacity(udp_data.len() + 2);
        tcp_data.extend_from_slice(&(udp_data.len() as u16).to_be_bytes());
        tcp_data.extend_from_slice(&udp_data);
        
        Ok(tcp_data)
    }
    
    /// 从TCP流读取完整的DNS响应
    async fn read_tcp_response(stream: &mut TcpStream) -> Result<Vec<u8>> {
        // 读取2字节长度前缀
        let mut length_buf = [0u8; 2];
        stream.read_exact(&mut length_buf).await
            .map_err(|e| DnsError::Network(format!("Failed to read length: {}", e)))?;
        
        let length = u16::from_be_bytes(length_buf) as usize;
        
        if length == 0 {
            return Err(DnsError::Protocol("Invalid response length".to_string()));
        }
        
        if length > 65535 {
            return Err(DnsError::Protocol("Response too large".to_string()));
        }
        
        // 读取实际的DNS响应数据
        let mut response_buf = vec![0u8; length];
        stream.read_exact(&mut response_buf).await
            .map_err(|e| DnsError::Network(format!("Failed to read response: {}", e)))?;
        
        Ok(response_buf)
    }
}

#[async_trait]
impl Transport for TcpTransport {
    async fn send(&self, request: &Request) -> Result<Response> {
        let server_addr = format!("{}:{}", self.config.server, self.config.port);
        
        // 建立TCP连接
        let connect_result = timeout(
            self.config.timeout,
            TcpStream::connect(&server_addr)
        ).await;
        
        let mut stream = match connect_result {
            Ok(Ok(stream)) => stream,
            Ok(Err(e)) => return Err(DnsError::Network(format!("Connection failed: {}", e))),
            Err(_) => return Err(DnsError::Timeout),
        };
        
        // 设置TCP选项
        if self.config.tcp_nodelay {
            if let Err(e) = stream.set_nodelay(true) {
                // 记录警告但不失败
                dns_error!("Failed to set TCP_NODELAY: {}", e);
            }
        }
        
        // 序列化请求
        let request_data = Self::serialize_request_tcp(request)?;
        
        // 发送请求
        let send_result = timeout(
            self.config.timeout,
            stream.write_all(&request_data)
        ).await;
        
        match send_result {
            Ok(Ok(_)) => {},
            Ok(Err(e)) => return Err(DnsError::Network(format!("Send failed: {}", e))),
            Err(_) => return Err(DnsError::Timeout),
        }
        
        // 确保数据发送完毕
        let flush_result = timeout(
            self.config.timeout,
            stream.flush()
        ).await;
        
        match flush_result {
            Ok(Ok(_)) => {},
            Ok(Err(e)) => return Err(DnsError::Network(format!("Flush failed: {}", e))),
            Err(_) => return Err(DnsError::Timeout),
        }
        
        // 读取响应
        let response_data = timeout(
            self.config.timeout,
            Self::read_tcp_response(&mut stream)
        ).await;
        
        let response_bytes = match response_data {
            Ok(Ok(data)) => data,
            Ok(Err(e)) => return Err(e),
            Err(_) => return Err(DnsError::Timeout),
        };
        
        // 复用UDP的反序列化逻辑
        UdpTransport::deserialize_response(&response_bytes)
    }
    
    fn transport_type(&self) -> &'static str {
        "TCP"
    }
    
    fn set_timeout(&mut self, timeout: Duration) {
        self.config.timeout = timeout;
    }
    
    fn timeout(&self) -> Duration {
        self.config.timeout
    }
}