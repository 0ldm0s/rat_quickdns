//! HTTPS传输实现 (DNS over HTTPS)

use crate::{Request, Response, Result, DnsError};
use super::{Transport, HttpsConfig, HttpMethod};
use super::udp::UdpTransport;
use async_trait::async_trait;
use std::time::Duration;
use tokio::time::timeout;

#[cfg(feature = "reqwest")]
use reqwest::{Client, Method};

/// HTTPS传输实现
#[derive(Debug)]
pub struct HttpsTransport {
    config: HttpsConfig,
    #[cfg(feature = "reqwest")]
    client: Client,
}

impl HttpsTransport {
    /// 创建新的HTTPS传输
    #[cfg(feature = "reqwest")]
    pub fn new(config: HttpsConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(config.base.timeout)
            .user_agent(&config.user_agent)
            .build()
            .map_err(|e| DnsError::Http(format!("Failed to create HTTP client: {}", e)))?;
        
        Ok(Self {
            config,
            client,
        })
    }
    
    #[cfg(not(feature = "reqwest"))]
    pub fn new(_config: HttpsConfig) -> Result<Self> {
        Err(DnsError::Config("HTTPS support requires 'reqwest' feature".to_string()))
    }
    
    /// 使用默认配置创建HTTPS传输
    pub fn default() -> Result<Self> {
        Self::new(HttpsConfig::default())
    }
    
    /// 将DNS请求编码为base64url格式(用于GET方法)
    #[cfg(feature = "reqwest")]
    fn encode_dns_query_base64url(request: &Request) -> Result<String> {
        use base64::{Engine as _, engine::general_purpose};
        let dns_data = UdpTransport::serialize_request(request)?;
        Ok(general_purpose::URL_SAFE_NO_PAD.encode(&dns_data))
    }
    
    /// 发送GET请求
    #[cfg(feature = "reqwest")]
    async fn send_get_request(&self, request: &Request) -> Result<Response> {
        let dns_query = Self::encode_dns_query_base64url(request)?;
        
        let response = timeout(
            self.config.base.timeout,
            self.client
                .get(&self.config.url)
                .query(&[("dns", dns_query)])
                .header("Accept", "application/dns-message")
                .send()
        ).await;
        
        let http_response = match response {
            Ok(Ok(resp)) => resp,
            Ok(Err(e)) => return Err(DnsError::Http(format!("HTTP request failed: {}", e))),
            Err(_) => return Err(DnsError::Timeout),
        };
        
        if !http_response.status().is_success() {
            return Err(DnsError::Http(format!(
                "HTTP error: {} {}", 
                http_response.status().as_u16(),
                http_response.status().canonical_reason().unwrap_or("Unknown")
            )));
        }
        
        let content_type = http_response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        
        if !content_type.contains("application/dns-message") {
            return Err(DnsError::Http(format!(
                "Invalid content type: expected 'application/dns-message', got '{}'",
                content_type
            )));
        }
        
        let body_result = timeout(
            self.config.base.timeout,
            http_response.bytes()
        ).await;
        
        let body = match body_result {
            Ok(Ok(bytes)) => bytes,
            Ok(Err(e)) => return Err(DnsError::Http(format!("Failed to read response body: {}", e))),
            Err(_) => return Err(DnsError::Timeout),
        };
        
        UdpTransport::deserialize_response(&body)
    }
    
    /// 发送POST请求
    #[cfg(feature = "reqwest")]
    async fn send_post_request(&self, request: &Request) -> Result<Response> {
        let dns_data = UdpTransport::serialize_request(request)?;
        
        let response = timeout(
            self.config.base.timeout,
            self.client
                .post(&self.config.url)
                .header("Content-Type", "application/dns-message")
                .header("Accept", "application/dns-message")
                .body(dns_data)
                .send()
        ).await;
        
        let http_response = match response {
            Ok(Ok(resp)) => resp,
            Ok(Err(e)) => return Err(DnsError::Http(format!("HTTP request failed: {}", e))),
            Err(_) => return Err(DnsError::Timeout),
        };
        
        if !http_response.status().is_success() {
            return Err(DnsError::Http(format!(
                "HTTP error: {} {}", 
                http_response.status().as_u16(),
                http_response.status().canonical_reason().unwrap_or("Unknown")
            )));
        }
        
        let content_type = http_response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        
        if !content_type.contains("application/dns-message") {
            return Err(DnsError::Http(format!(
                "Invalid content type: expected 'application/dns-message', got '{}'",
                content_type
            )));
        }
        
        let body_result = timeout(
            self.config.base.timeout,
            http_response.bytes()
        ).await;
        
        let body = match body_result {
            Ok(Ok(bytes)) => bytes,
            Ok(Err(e)) => return Err(DnsError::Http(format!("Failed to read response body: {}", e))),
            Err(_) => return Err(DnsError::Timeout),
        };
        
        UdpTransport::deserialize_response(&body)
    }
}

#[cfg(feature = "reqwest")]
#[async_trait]
impl Transport for HttpsTransport {
    async fn send(&self, request: &Request) -> Result<Response> {
        match self.config.method {
            HttpMethod::GET => self.send_get_request(request).await,
            HttpMethod::POST => self.send_post_request(request).await,
        }
    }
    
    fn transport_type(&self) -> &'static str {
        "HTTPS"
    }
    
    fn set_timeout(&mut self, timeout: Duration) {
        self.config.base.timeout = timeout;
        // 注意: reqwest客户端的超时时间在创建时设置，无法动态修改
        // 如果需要动态修改超时时间，需要重新创建客户端
    }
    
    fn timeout(&self) -> Duration {
        self.config.base.timeout
    }
}

#[cfg(not(feature = "reqwest"))]
#[async_trait]
impl Transport for HttpsTransport {
    async fn send(&self, _request: &Request) -> Result<Response> {
        Err(DnsError::Config("HTTPS support requires 'reqwest' feature".to_string()))
    }
    
    fn transport_type(&self) -> &'static str {
        "HTTPS (disabled)"
    }
    
    fn set_timeout(&mut self, timeout: Duration) {
        self.config.base.timeout = timeout;
    }
    
    fn timeout(&self) -> Duration {
        self.config.base.timeout
    }
}

/// 常用的DoH服务器配置
impl HttpsConfig {
    /// Cloudflare DoH配置
    pub fn cloudflare() -> Self {
        Self {
            url: "https://cloudflare-dns.com/dns-query".to_string(),
            method: HttpMethod::POST,
            ..Default::default()
        }
    }
    
    /// Google DoH配置
    pub fn google() -> Self {
        Self {
            url: "https://dns.google/dns-query".to_string(),
            method: HttpMethod::POST,
            ..Default::default()
        }
    }
    
    /// Quad9 DoH配置
    pub fn quad9() -> Self {
        Self {
            url: "https://dns.quad9.net/dns-query".to_string(),
            method: HttpMethod::POST,
            ..Default::default()
        }
    }
    
    /// OpenDNS DoH配置
    pub fn opendns() -> Self {
        Self {
            url: "https://doh.opendns.com/dns-query".to_string(),
            method: HttpMethod::POST,
            ..Default::default()
        }
    }
}