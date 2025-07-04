//! DNS解析器实现模块
//! 
//! 本模块实现了高性能DNS解析器的核心功能

use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;
use rat_quickmem::QuickMemConfig;

use crate::resolver::{ResolverConfig, Resolver};
use crate::upstream_handler::UpstreamManager;
use crate::error::{DnsError, Result};
use super::{
    strategy::QueryStrategy,
    engine::SmartDecisionEngine,
    types::{DnsQueryRequest, DnsQueryResponse, DnsRecord, DnsRecordType},
};

/// 高性能DNS解析器
#[derive(Debug)]
pub struct EasyDnsResolver {
    /// 底层解析器
    resolver: Resolver,
    
    /// 上游管理器
    upstream_manager: UpstreamManager,
    
    /// QuickMem配置
    quickmem_config: QuickMemConfig,
    
    /// 智能决策引擎（可选）
    decision_engine: Option<Arc<SmartDecisionEngine>>,
    
    /// 查询策略
    query_strategy: QueryStrategy,
    
    /// 是否启用EDNS
    enable_edns: bool,
}

impl EasyDnsResolver {
    /// 创建新的DNS解析器
    pub(super) fn new(
        config: ResolverConfig,
        upstream_manager: UpstreamManager,
        quickmem_config: QuickMemConfig,
        decision_engine: Option<Arc<SmartDecisionEngine>>,
        query_strategy: QueryStrategy,
        enable_edns: bool,
    ) -> Result<Self> {
        let mut resolver = Resolver::new(config);
        
        // 根据上游管理器配置添加传输协议
        for spec in upstream_manager.get_specs() {
            match spec.transport_type {
                crate::upstream_handler::UpstreamType::Udp => {
                    // 解析服务器地址和端口
                    let (server, port) = if spec.server.contains(':') {
                        let parts: Vec<&str> = spec.server.split(':').collect();
                        (parts[0].to_string(), parts[1].parse().unwrap_or(53))
                    } else {
                        (spec.server.clone(), 53)
                    };
                    
                    let transport_config = crate::transport::TransportConfig {
                        server,
                        port,
                        timeout: std::time::Duration::from_secs(5),
                        tcp_fast_open: false,
                        tcp_nodelay: true,
                        pool_size: 10,
                    };
                    resolver.add_udp_transport(transport_config);
                },
                crate::upstream_handler::UpstreamType::Tcp => {
                    // 解析服务器地址和端口
                    let (server, port) = if spec.server.contains(':') {
                        let parts: Vec<&str> = spec.server.split(':').collect();
                        (parts[0].to_string(), parts[1].parse().unwrap_or(53))
                    } else {
                        (spec.server.clone(), 53)
                    };
                    
                    let transport_config = crate::transport::TransportConfig {
                        server,
                        port,
                        timeout: std::time::Duration::from_secs(5),
                        tcp_fast_open: false,
                        tcp_nodelay: true,
                        pool_size: 10,
                    };
                    resolver.add_tcp_transport(transport_config);
                },
                crate::upstream_handler::UpstreamType::DoH => {
                    let https_config = crate::transport::HttpsConfig {
                        base: crate::transport::TransportConfig {
                            server: "cloudflare-dns.com".to_string(),
                            port: 443,
                            timeout: std::time::Duration::from_secs(10),
                            tcp_fast_open: false,
                            tcp_nodelay: true,
                            pool_size: 5,
                        },
                        url: spec.server.clone(),
                        method: crate::transport::HttpMethod::POST,
                        user_agent: "rat_quickdns/0.1.0".to_string(),
                    };
                    resolver.add_https_transport(https_config)?;
                },
                crate::upstream_handler::UpstreamType::DoT => {
                    // 解析服务器地址和端口
                    let (server, port) = if spec.server.contains(':') {
                        let parts: Vec<&str> = spec.server.split(':').collect();
                        (parts[0].to_string(), parts[1].parse().unwrap_or(853))
                    } else {
                        (spec.server.clone(), 853)
                    };
                    
                    let tls_config = crate::transport::TlsConfig {
                        base: crate::transport::TransportConfig {
                            server: server.clone(),
                            port,
                            timeout: std::time::Duration::from_secs(10),
                            tcp_fast_open: false,
                            tcp_nodelay: true,
                            pool_size: 5,
                        },
                        server_name: server,
                        verify_cert: true,
                    };
                    resolver.add_tls_transport(tls_config)?;
                },
            }
        }
        
        Ok(Self {
            resolver,
            upstream_manager,
            quickmem_config,
            decision_engine,
            query_strategy,
            enable_edns,
        })
    }
    
    /// 执行DNS查询
    pub async fn query(&self, request: DnsQueryRequest) -> Result<DnsQueryResponse> {
        let start_time = Instant::now();
        let query_id = request.query_id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());
        
        // 通用应急检查：在执行任何策略前检查是否所有服务器都失败
        if let Some(emergency_error) = self.check_emergency_status().await {
            let duration = start_time.elapsed();
            return Ok(DnsQueryResponse {
                query_id,
                domain: request.domain,
                record_type: request.record_type,
                success: false,
                error: Some(emergency_error),
                records: Vec::new(),
                duration_ms: duration.as_millis() as u64,
                server_used: None,
                dnssec_status: Some(crate::builder::types::DnssecStatus::Indeterminate),
                dnssec_records: Vec::new(),
            });
        }
        
        // 根据策略选择上游服务器
        let result = match self.query_strategy {
            QueryStrategy::Fifo => self.query_fifo(&request).await,
            QueryStrategy::Smart => self.query_smart(&request).await,
            QueryStrategy::RoundRobin => self.query_round_robin(&request).await,
        };
        
        let duration = start_time.elapsed();
        
        match result {
            Ok((response, server_used)) => {
                // 更新性能指标
                if let Some(engine) = &self.decision_engine {
                    engine.update_metrics(&server_used, duration, true, true).await;
                }
                
                Ok(DnsQueryResponse {
                    query_id,
                    domain: request.domain,
                    record_type: request.record_type,
                    success: true,
                    error: None,
                    records: self.convert_response_to_records(response),
                    duration_ms: duration.as_millis() as u64,
                    server_used: Some(server_used),
                    dnssec_status: Some(crate::builder::types::DnssecStatus::Indeterminate),
                    dnssec_records: Vec::new(),
                })
            },
            Err(e) => {
                // 通用错误处理：在查询失败后再次检查应急状态
                let enhanced_error = self.enhance_error_with_emergency_info(e).await;
                
                Ok(DnsQueryResponse {
                    query_id,
                    domain: request.domain,
                    record_type: request.record_type,
                    success: false,
                    error: Some(enhanced_error),
                    records: Vec::new(),
                    duration_ms: duration.as_millis() as u64,
                    server_used: None,
                    dnssec_status: Some(crate::builder::types::DnssecStatus::Indeterminate),
                    dnssec_records: Vec::new(),
                })
            }
        }
    }
    
    /// FIFO查询策略
    async fn query_fifo(&self, request: &DnsQueryRequest) -> Result<(crate::Response, String)> {
        let record_type = self.convert_record_type(request.record_type);
        
        if let Some(engine) = &self.decision_engine {
            // 使用决策引擎按FIFO顺序选择服务器
            if let Some(spec) = engine.select_fifo_upstream().await {
                let start_time = Instant::now();
                
                match self.resolver.query(&request.domain, record_type, crate::types::QClass::IN).await {
                    Ok(response) => {
                        let duration = start_time.elapsed();
                        engine.update_metrics(&spec.name, duration, true, true).await;
                        Ok((response, spec.name))
                    },
                    Err(e) => {
                        let duration = start_time.elapsed();
                        engine.update_metrics(&spec.name, duration, false, false).await;
                        Err(e)
                    }
                }
            } else {
                Err(DnsError::NoUpstreamAvailable)
            }
        } else {
            // 没有决策引擎，使用基础解析器
            let response = self.resolver.query(&request.domain, record_type, crate::types::QClass::IN).await?;
            Ok((response, "fifo-fallback".to_string()))
        }
    }
    
    /// 智能查询策略
    async fn query_smart(&self, request: &DnsQueryRequest) -> Result<(crate::Response, String)> {
        let record_type = self.convert_record_type(request.record_type);
        
        if let Some(engine) = &self.decision_engine {
            // 使用决策引擎选择最优服务器
            if let Some(spec) = engine.select_smart_upstream().await {
                let start_time = Instant::now();
                
                match self.resolver.query(&request.domain, record_type, crate::types::QClass::IN).await {
                    Ok(response) => {
                        let duration = start_time.elapsed();
                        engine.update_metrics(&spec.name, duration, true, true).await;
                        Ok((response, spec.name))
                    },
                    Err(e) => {
                        let duration = start_time.elapsed();
                        engine.update_metrics(&spec.name, duration, false, false).await;
                        Err(e)
                    }
                }
            } else {
                Err(DnsError::NoUpstreamAvailable)
            }
        } else {
            Err(DnsError::InvalidConfig("Smart strategy requires decision engine".to_string()))
        }
    }
    
    /// 转换记录类型
    fn convert_record_type(&self, record_type: DnsRecordType) -> crate::types::RecordType {
        match record_type {
            DnsRecordType::A => crate::types::RecordType::A,
            DnsRecordType::AAAA => crate::types::RecordType::AAAA,
            DnsRecordType::CNAME => crate::types::RecordType::CNAME,
            DnsRecordType::MX => crate::types::RecordType::MX,
            DnsRecordType::TXT => crate::types::RecordType::TXT,
            DnsRecordType::NS => crate::types::RecordType::NS,
            DnsRecordType::PTR => crate::types::RecordType::PTR,
            DnsRecordType::SRV => crate::types::RecordType::SRV,
            DnsRecordType::SOA => crate::types::RecordType::SOA,
            // DNSSEC记录类型映射到Unknown类型，使用标准的DNSSEC记录类型代码
            DnsRecordType::RRSIG => crate::types::RecordType::Unknown(46),  // RRSIG
            DnsRecordType::DNSKEY => crate::types::RecordType::Unknown(48), // DNSKEY
            DnsRecordType::DS => crate::types::RecordType::Unknown(43),     // DS
            DnsRecordType::NSEC => crate::types::RecordType::Unknown(47),   // NSEC
            DnsRecordType::NSEC3 => crate::types::RecordType::Unknown(50),  // NSEC3
        }
    }
    
    /// 轮询查询策略（优化版本）
    async fn query_round_robin(&self, request: &DnsQueryRequest) -> Result<(crate::Response, String)> {
        let record_type = self.convert_record_type(request.record_type);
        
        if let Some(engine) = &self.decision_engine {
            let mut last_error = None;
            let mut attempted_servers = Vec::new();
            
            // 最多尝试3次不同的服务器
            for attempt in 0..3 {
                if let Some(spec) = engine.select_round_robin_upstream().await {
                    attempted_servers.push(spec.name.clone());
                    let start_time = Instant::now();
                    
                    match self.resolver.query(&request.domain, record_type, crate::types::QClass::IN).await {
                        Ok(response) => {
                            let duration = start_time.elapsed();
                            engine.update_metrics(&spec.name, duration, true, true).await;
                            return Ok((response, spec.name));
                        },
                        Err(e) => {
                            let duration = start_time.elapsed();
                            engine.update_metrics(&spec.name, duration, false, false).await;
                            last_error = Some(e);
                            
                            // 短暂延迟后重试下一个服务器
                            if attempt < 2 {
                                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                            }
                        }
                    }
                } else {
                    break;
                }
            }
            
            // 如果所有尝试都失败了，返回详细的错误信息
            if let Some(error) = last_error {
                Err(DnsError::Server(format!(
                    "Round-robin查询失败，已尝试服务器: [{}]，最后错误: {}",
                    attempted_servers.join(", "),
                    error
                )))
            } else {
                Err(DnsError::NoUpstreamAvailable)
            }
        } else {
            // 没有决策引擎，使用基础解析器
            let response = self.resolver.query(&request.domain, record_type, crate::types::QClass::IN).await?;
            Ok((response, "round-robin-fallback".to_string()))
        }
    }
    

    /// 转换响应为记录
    fn convert_response_to_records(&self, response: crate::Response) -> Vec<DnsRecord> {
        use crate::builder::types::{DnsRecord, DnsRecordValue};
        
        let mut records = Vec::new();
        
        for record in response.answers {
            let record_type = match record.rtype {
                crate::types::RecordType::A => DnsRecordType::A,
                crate::types::RecordType::AAAA => DnsRecordType::AAAA,
                crate::types::RecordType::CNAME => DnsRecordType::CNAME,
                crate::types::RecordType::MX => DnsRecordType::MX,
                crate::types::RecordType::TXT => DnsRecordType::TXT,
                crate::types::RecordType::NS => DnsRecordType::NS,
                crate::types::RecordType::PTR => DnsRecordType::PTR,
                crate::types::RecordType::SRV => DnsRecordType::SRV,
                crate::types::RecordType::SOA => DnsRecordType::SOA,
                _ => continue,
            };
            
            let value = match record.data {
                crate::types::RecordData::A(addr) => DnsRecordValue::IpAddr(addr.into()),
                crate::types::RecordData::AAAA(addr) => DnsRecordValue::IpAddr(addr.into()),
                crate::types::RecordData::CNAME(name) => DnsRecordValue::Domain(name),
                crate::types::RecordData::NS(name) => DnsRecordValue::Domain(name),
                crate::types::RecordData::PTR(name) => DnsRecordValue::Domain(name),
                crate::types::RecordData::TXT(texts) => DnsRecordValue::Text(texts.join(" ")),
                crate::types::RecordData::MX { priority, exchange } => {
                    DnsRecordValue::Mx { priority, exchange }
                },
                crate::types::RecordData::SRV { priority, weight, port, target } => {
                    DnsRecordValue::Srv { priority, weight, port, target }
                },
                _ => continue,
            };
            
            records.push(DnsRecord {
                name: record.name,
                record_type,
                value,
                ttl: record.ttl,
            });
        }
        
        records
    }
    
    /// 获取解析器统计信息
    pub async fn get_stats(&self) -> ResolverStats {
        let mut stats = ResolverStats::default();
        
        if let Some(engine) = &self.decision_engine {
            let metrics = engine.get_all_metrics().await;
            stats.total_upstreams = metrics.len();
            stats.healthy_upstreams = engine.healthy_upstream_count().await;
            
            for (name, metric) in metrics {
                stats.total_queries += metric.total_queries;
                stats.successful_queries += metric.successful_queries;
                stats.failed_queries += metric.failed_queries;
                
                if metric.avg_latency < stats.min_latency || stats.min_latency.is_zero() {
                    stats.min_latency = metric.avg_latency;
                    stats.fastest_upstream = Some(name.clone());
                }
                
                if metric.avg_latency > stats.max_latency {
                    stats.max_latency = metric.avg_latency;
                    stats.slowest_upstream = Some(name);
                }
            }
        }
        
        stats.strategy = self.query_strategy;
        stats.edns_enabled = self.enable_edns;
        
        stats
    }
    
    /// 重置所有统计信息
    pub async fn reset_stats(&self) {
        if let Some(engine) = &self.decision_engine {
            engine.reset_metrics().await;
        }
    }
    
    /// 获取上游服务器健康状态
    pub async fn get_upstream_health(&self) -> Vec<UpstreamHealth> {
        let mut health_list = Vec::new();
        
        if let Some(engine) = &self.decision_engine {
            let upstreams = engine.get_upstreams().await;
            let metrics = engine.get_all_metrics().await;
            
            for upstream in upstreams {
                let metric = metrics.get(&upstream.name).cloned().unwrap_or_default();
                
                health_list.push(UpstreamHealth {
                    name: upstream.name,
                    server: upstream.server,
                    transport_type: upstream.transport_type,
                    is_healthy: metric.is_healthy(),
                    success_rate: metric.success_rate(),
                    avg_latency: metric.avg_latency,
                    consecutive_failures: metric.consecutive_failures,
                    total_queries: metric.total_queries,
                    last_success: metric.last_success_time,
                });
            }
        }
        
        health_list
    }
    
    /// 获取查询策略
    pub fn query_strategy(&self) -> QueryStrategy {
        self.query_strategy
    }
    
    /// 是否启用EDNS
    pub fn is_edns_enabled(&self) -> bool {
        self.enable_edns
    }
    
    /// 获取决策引擎引用
    pub fn get_decision_engine(&self) -> Option<&Arc<SmartDecisionEngine>> {
        self.decision_engine.as_ref()
    }
    
    /// 通用应急状态检查
    /// 
    /// 检查是否所有上游服务器都失败，如果是则返回应急错误信息
    async fn check_emergency_status(&self) -> Option<String> {
        if let Some(engine) = &self.decision_engine {
            if engine.all_upstreams_failed().await {
                let emergency_info = engine.get_emergency_response_info().await;
                return Some(format!(
                    "🚨 应急模式激活: {} (策略: {:?})",
                    emergency_info.emergency_message,
                    self.query_strategy
                ));
            }
        }
        None
    }
    
    /// 增强错误信息，添加应急响应详情
    /// 
    /// 在查询失败后，检查应急状态并增强错误信息
    async fn enhance_error_with_emergency_info(&self, original_error: DnsError) -> String {
        if let Some(engine) = &self.decision_engine {
            let emergency_info = engine.get_emergency_response_info().await;
            
            if emergency_info.all_servers_failed {
                format!(
                    "查询失败 (策略: {:?}): {}\n🚨 应急信息: {}\n📊 失败统计: {}次\n📋 失败服务器: [{}]",
                    self.query_strategy,
                    original_error,
                    emergency_info.emergency_message,
                    emergency_info.total_failures,
                    emergency_info.failed_servers.iter()
                        .map(|s| format!("{} ({}次)", s.name, s.consecutive_failures))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            } else if emergency_info.total_failures > 0 {
                format!(
                    "查询失败 (策略: {:?}): {}\n⚠️  部分服务器不可用: {}次失败",
                    self.query_strategy,
                    original_error,
                    emergency_info.total_failures
                )
            } else {
                format!("查询失败 (策略: {:?}): {}", self.query_strategy, original_error)
            }
        } else {
            format!("查询失败 (策略: {:?}, 无决策引擎): {}", self.query_strategy, original_error)
        }
    }
    
    /// 获取上游管理器引用
    pub fn upstream_manager(&self) -> &UpstreamManager {
        &self.upstream_manager
    }
    
    /// 获取QuickMem配置
    pub fn quickmem_config(&self) -> &QuickMemConfig {
        &self.quickmem_config
    }
}

impl Clone for EasyDnsResolver {
    fn clone(&self) -> Self {
        // 由于Resolver包含trait对象，我们需要重新创建一个新的实例
        // 这里我们使用相同的配置来创建新的解析器
        let config = crate::resolver::ResolverConfig {
            strategy: crate::resolver::strategy::QueryStrategy::FastestFirst,
            default_timeout: std::time::Duration::from_secs(5),
            retry_count: 2,
            enable_cache: true,
            max_cache_ttl: std::time::Duration::from_secs(3600),
            enable_health_check: true,
            health_check_interval: std::time::Duration::from_secs(30),
            default_client_subnet: None,
            port: 53,
            concurrent_queries: 10,
            recursion_desired: true,
            buffer_size: 4096,
            log_level: zerg_creep::logger::LevelFilter::Info,
            enable_dns_log_format: true,
        };
        
        Self::new(
            config,
            self.upstream_manager.clone(),
            self.quickmem_config.clone(),
            self.decision_engine.clone(),
            self.query_strategy,
            self.enable_edns,
        ).expect("Failed to clone EasyDnsResolver")
    }
}

/// 解析器统计信息
#[derive(Debug, Clone, Default)]
pub struct ResolverStats {
    /// 查询策略
    pub strategy: QueryStrategy,
    
    /// 是否启用EDNS
    pub edns_enabled: bool,
    
    /// 总上游服务器数量
    pub total_upstreams: usize,
    
    /// 健康的上游服务器数量
    pub healthy_upstreams: usize,
    
    /// 总查询次数
    pub total_queries: u64,
    
    /// 成功查询次数
    pub successful_queries: u64,
    
    /// 失败查询次数
    pub failed_queries: u64,
    
    /// 最小延迟
    pub min_latency: std::time::Duration,
    
    /// 最大延迟
    pub max_latency: std::time::Duration,
    
    /// 最快的上游服务器
    pub fastest_upstream: Option<String>,
    
    /// 最慢的上游服务器
    pub slowest_upstream: Option<String>,
}

impl ResolverStats {
    /// 计算总体成功率
    pub fn success_rate(&self) -> f64 {
        if self.total_queries == 0 {
            0.0
        } else {
            self.successful_queries as f64 / self.total_queries as f64
        }
    }
    
    /// 计算平均延迟
    pub fn avg_latency(&self) -> std::time::Duration {
        if self.min_latency.is_zero() && self.max_latency.is_zero() {
            std::time::Duration::from_millis(0)
        } else {
            (self.min_latency + self.max_latency) / 2
        }
    }
}

/// 上游服务器健康状态
#[derive(Debug, Clone)]
pub struct UpstreamHealth {
    /// 服务器名称
    pub name: String,
    
    /// 服务器地址
    pub server: String,
    
    /// 传输类型
    pub transport_type: crate::upstream_handler::UpstreamType,
    
    /// 是否健康
    pub is_healthy: bool,
    
    /// 成功率
    pub success_rate: f64,
    
    /// 平均延迟
    pub avg_latency: std::time::Duration,
    
    /// 连续失败次数
    pub consecutive_failures: u32,
    
    /// 总查询次数
    pub total_queries: u64,
    
    /// 最后成功时间
    pub last_success: Option<std::time::Instant>,
}