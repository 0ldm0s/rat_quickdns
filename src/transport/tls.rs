//! TLS传输实现 (DNS over TLS)

use crate::{Request, Response, Result, DnsError};
use super::{Transport, TlsConfig};
use super::udp::UdpTransport;
use async_trait::async_trait;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::timeout;
use std::sync::{Arc, Mutex};
use crate::{dns_debug, dns_info, dns_error, dns_transport};
use tokio_rustls::{TlsConnector, rustls::{ClientConfig, ServerName}};
/// TLS传输实现
pub struct TlsTransport {
    #[allow(dead_code)]
    connector: Arc<TlsConnector>,
    config: Arc<Mutex<TlsConfig>>,
}

impl std::fmt::Debug for TlsTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TlsTransport")
            .field("config", &self.config)
            .finish()
    }
}

impl TlsTransport {
    /// 创建新的TLS传输
    pub fn new(config: TlsConfig) -> Result<Self> {
        let mut client_config = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(Self::load_root_certs()?)
            .with_no_client_auth();
        
        if !config.verify_cert {
            client_config.dangerous()
                .set_certificate_verifier(Arc::new(NoVerifier));
        }
        
        let connector = TlsConnector::from(Arc::new(client_config));
        
        Ok(Self {
            config: Arc::new(Mutex::new(config)),
            connector: Arc::new(connector),
        })
    }
    
    // 注意：移除了 default() 方法，因为它依赖兜底配置
    // 用户现在必须明确提供 TlsConfig，不能依赖隐式默认值
    // 
    // 迁移示例：
    // 旧代码: TlsTransport::default()
    // 新代码: TlsTransport::new(TlsConfig {
    //     base: TransportConfig {
    //         server: "your-dns-server.com".to_string(),
    //         port: 853,
    //         timeout: Duration::from_secs(5),
    //         tcp_fast_open: false,
    //         tcp_nodelay: true,
    //         pool_size: 10,
    //     },
    //     server_name: "your-dns-server.com".to_string(),
    //     verify_cert: true,
    // })
    
    /// 加载根证书
    fn load_root_certs() -> Result<tokio_rustls::rustls::RootCertStore> {
        let mut root_store = tokio_rustls::rustls::RootCertStore::empty();
        
        // 加载系统根证书
        match rustls_native_certs::load_native_certs() {
            Ok(certs) => {
                for cert in certs {
                    if let Err(e) = root_store.add(&tokio_rustls::rustls::Certificate(cert.0)) {
                        dns_error!("Failed to add certificate: {:?}", e);
                    }
                }
            }
            Err(e) => {
                return Err(DnsError::Tls(format!("Failed to load native certs: {}", e)));
            }
        }
        
        // 如果没有加载到任何证书，使用webpki根证书
        if root_store.is_empty() {
            root_store.add_server_trust_anchors(
                webpki_roots::TLS_SERVER_ROOTS.iter().map(|ta| {
                    tokio_rustls::rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
                        ta.subject,
                        ta.spki,
                        ta.name_constraints,
                    )
                })
            );
        }
        
        Ok(root_store)
    }
    
    /// 从TCP流读取完整的DNS响应
    async fn read_tls_response(
        stream: &mut tokio_rustls::client::TlsStream<TcpStream>
    ) -> Result<Vec<u8>> {
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
    
    /// 序列化DNS请求为TLS格式(带长度前缀)
    fn serialize_request_tls(request: &Request) -> Result<Vec<u8>> {
        // TLS使用与TCP相同的格式
        let udp_data = UdpTransport::serialize_request(request)?;
        
        let mut tls_data = Vec::with_capacity(udp_data.len() + 2);
        tls_data.extend_from_slice(&(udp_data.len() as u16).to_be_bytes());
        tls_data.extend_from_slice(&udp_data);
        
        Ok(tls_data)
    }
}

#[async_trait]
impl Transport for TlsTransport {
    async fn send(&self, request: &Request) -> Result<Response> {
        use crate::{dns_debug, dns_info};
        let (server_addr, timeout_duration, server_name) = {
            let config = self.config.lock().unwrap();
            dns_info!("🔒 DoT请求开始: {} -> {}:{}", request.query.name, config.base.server, config.base.port);
            (
                format!("{}:{}", config.base.server, config.base.port),
                config.base.timeout,
                config.server_name.clone()
            )
        };
        
        // 建立TCP连接
        let connect_result = timeout(
            timeout_duration,
            TcpStream::connect(&server_addr)
        ).await;
        
        let tcp_stream = match connect_result {
            Ok(Ok(stream)) => stream,
            Ok(Err(e)) => return Err(DnsError::Network(format!("Connection failed: {}", e))),
            Err(_) => return Err(DnsError::Timeout),
        };
        
        // 建立TLS连接
        let server_name = ServerName::try_from(server_name.as_str())
            .map_err(|e| DnsError::Tls(format!("Invalid server name: {}", e)))?;
        
        let tls_result = timeout(
            timeout_duration,
            self.connector.connect(server_name, tcp_stream)
        ).await;
        
        let mut tls_stream = match tls_result {
            Ok(Ok(stream)) => stream,
            Ok(Err(e)) => return Err(DnsError::Tls(format!("TLS handshake failed: {}", e))),
            Err(_) => return Err(DnsError::Timeout),
        };
        
        // 序列化请求
        let request_data = Self::serialize_request_tls(request)?;
        
        // 发送请求
        let send_result = timeout(
            timeout_duration,
            tls_stream.write_all(&request_data)
        ).await;
        
        match send_result {
            Ok(Ok(_)) => {},
            Ok(Err(e)) => return Err(DnsError::Network(format!("Send failed: {}", e))),
            Err(_) => return Err(DnsError::Timeout),
        }
        
        // 确保数据发送完毕
        let flush_result = timeout(
            timeout_duration,
            tls_stream.flush()
        ).await;
        
        match flush_result {
            Ok(Ok(_)) => {},
            Ok(Err(e)) => return Err(DnsError::Network(format!("Flush failed: {}", e))),
            Err(_) => return Err(DnsError::Timeout),
        }
        
        // 读取响应
        let response_data = timeout(
            timeout_duration,
            Self::read_tls_response(&mut tls_stream)
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
        "TLS"
    }
    
    fn set_timeout(&mut self, timeout: Duration) {
        if let Ok(mut config) = self.config.lock() {
            config.base.timeout = timeout;
        }
    }
    
    fn timeout(&self) -> Duration {
        self.config.lock().unwrap().base.timeout
    }
}



/// 不验证证书的验证器(仅用于测试)
struct NoVerifier;

impl tokio_rustls::rustls::client::ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &tokio_rustls::rustls::Certificate,
        _intermediates: &[tokio_rustls::rustls::Certificate],
        _server_name: &tokio_rustls::rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> std::result::Result<tokio_rustls::rustls::client::ServerCertVerified, tokio_rustls::rustls::Error> {
        Ok(tokio_rustls::rustls::client::ServerCertVerified::assertion())
    }
}