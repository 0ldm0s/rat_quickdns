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
                })
            },
            Err(e) => {
                Ok(DnsQueryResponse {
                    query_id,
                    domain: request.domain,
                    record_type: request.record_type,
                    success: false,
                    error: Some(e.to_string()),
                    records: Vec::new(),
                    duration_ms: duration.as_millis() as u64,
                    server_used: None,
                })
            }
        }
    }
    
    /// FIFO查询策略
    async fn query_fifo(&self, request: &DnsQueryRequest) -> Result<(crate::Response, String)> {
        // 转换记录类型
        let record_type = match request.record_type {
            DnsRecordType::A => crate::types::RecordType::A,
            DnsRecordType::AAAA => crate::types::RecordType::AAAA,
            DnsRecordType::CNAME => crate::types::RecordType::CNAME,
            DnsRecordType::MX => crate::types::RecordType::MX,
            DnsRecordType::TXT => crate::types::RecordType::TXT,
            DnsRecordType::NS => crate::types::RecordType::NS,
            DnsRecordType::PTR => crate::types::RecordType::PTR,
            DnsRecordType::SRV => crate::types::RecordType::SRV,
            DnsRecordType::SOA => crate::types::RecordType::SOA,
        };
        
        // 使用底层解析器执行查询
        let response = self.resolver.query(&request.domain, record_type, crate::types::QClass::IN).await?;
        
        // 返回响应和使用的服务器信息
        Ok((response, "fifo-upstream".to_string()))
    }
    
    /// 智能查询策略
    async fn query_smart(&self, request: &DnsQueryRequest) -> Result<(crate::Response, String)> {
        if let Some(engine) = &self.decision_engine {
            if let Some(spec) = engine.select_best_upstream().await {
                // 转换记录类型
                let record_type = match request.record_type {
                    DnsRecordType::A => crate::types::RecordType::A,
                    DnsRecordType::AAAA => crate::types::RecordType::AAAA,
                    DnsRecordType::CNAME => crate::types::RecordType::CNAME,
                    DnsRecordType::MX => crate::types::RecordType::MX,
                    DnsRecordType::TXT => crate::types::RecordType::TXT,
                    DnsRecordType::NS => crate::types::RecordType::NS,
                    DnsRecordType::PTR => crate::types::RecordType::PTR,
                    DnsRecordType::SRV => crate::types::RecordType::SRV,
                    DnsRecordType::SOA => crate::types::RecordType::SOA,
                };
                
                // 使用底层解析器执行查询
                let response = self.resolver.query(&request.domain, record_type, crate::types::QClass::IN).await?;
                
                // 返回响应和使用的服务器信息
                Ok((response, spec.name))
            } else {
                Err(DnsError::NoUpstreamAvailable)
            }
        } else {
            Err(DnsError::InvalidConfig("Smart strategy requires decision engine".to_string()))
        }
    }
    
    /// 轮询查询策略
    async fn query_round_robin(&self, request: &DnsQueryRequest) -> Result<(crate::Response, String)> {
        // 转换记录类型
        let record_type = match request.record_type {
            DnsRecordType::A => crate::types::RecordType::A,
            DnsRecordType::AAAA => crate::types::RecordType::AAAA,
            DnsRecordType::CNAME => crate::types::RecordType::CNAME,
            DnsRecordType::MX => crate::types::RecordType::MX,
            DnsRecordType::TXT => crate::types::RecordType::TXT,
            DnsRecordType::NS => crate::types::RecordType::NS,
            DnsRecordType::PTR => crate::types::RecordType::PTR,
            DnsRecordType::SRV => crate::types::RecordType::SRV,
            DnsRecordType::SOA => crate::types::RecordType::SOA,
        };
        
        // 使用底层解析器执行查询
        let response = self.resolver.query(&request.domain, record_type, crate::types::QClass::IN).await?;
        
        // 返回响应和使用的服务器信息
        Ok((response, "round-robin-upstream".to_string()))
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
    
    /// 获取上游管理器引用
    pub fn upstream_manager(&self) -> &UpstreamManager {
        &self.upstream_manager
    }
    
    /// 获取QuickMem配置
    pub fn quickmem_config(&self) -> &QuickMemConfig {
        &self.quickmem_config
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