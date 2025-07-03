//! 开箱即用的DNS解析器构造器
//! 
//! 提供简化的API接口，集成bincode2序列化和rat_quickmem高性能内存管理
//! 支持FIFO模式和智能决策模式，默认启用EDNS

use crate::{
    resolver::{Resolver, ResolverConfig}, DnsError, Result,
    types::{Query, RecordType, QClass},
    transport::{TransportConfig, TlsConfig, HttpsConfig},
};
use rat_quickmem::{encode, decode, QuickMemConfig};
use serde::{Serialize, Deserialize};
use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use uuid::Uuid;

/// 查询策略模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryStrategy {
    /// FIFO模式：同时查询多个服务器，返回最快响应
    Fifo,
    /// 智能决策模式：基于性能指标选择最优服务器
    Smart,
}

/// 传输类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportType {
    Udp,
    Tcp,
    Tls,
    Https,
}

/// 上游服务器配置
#[derive(Debug, Clone)]
pub struct UpstreamConfig {
    pub name: String,
    pub transport_type: TransportType,
    pub address: SocketAddr,
    pub url: Option<String>, // DoH URL
    pub weight: u32,
    pub expected_region: Option<String>, // 期望的CDN区域
}

/// 性能指标
#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    pub total_queries: u64,
    pub successful_queries: u64,
    pub failed_queries: u64,
    pub avg_latency: Duration,
    pub cdn_accuracy_score: f64, // CDN区分准确性评分 0.0-1.0
    pub last_success_time: Option<Instant>,
    pub consecutive_failures: u32,
}

/// 客户端子网信息（简化实现）
#[derive(Debug, Clone)]
pub struct ClientSubnet {
    address: IpAddr,
    prefix_len: u8,
}

impl ClientSubnet {
    pub fn new(address: impl Into<IpAddr>, prefix_len: u8) -> Self {
        Self {
            address: address.into(),
            prefix_len,
        }
    }
    
    pub fn to_bytes(&self) -> Vec<u8> {
        // 简化实现，实际应该按照RFC 7871格式编码
        match self.address {
            IpAddr::V4(ip) => {
                let mut bytes = vec![0, 1]; // Family: IPv4
                bytes.push(self.prefix_len);
                bytes.push(0); // Scope prefix-len
                bytes.extend_from_slice(&ip.octets());
                bytes
            },
            IpAddr::V6(ip) => {
                let mut bytes = vec![0, 2]; // Family: IPv6
                bytes.push(self.prefix_len);
                bytes.push(0); // Scope prefix-len
                bytes.extend_from_slice(&ip.octets());
                bytes
            }
        }
    }
}

/// EDNS选项代码
#[derive(Debug, Clone, Copy)]
pub enum EdnsCode {
    ClientSubnet = 8,
}

/// EDNS选项
#[derive(Debug, Clone, Default)]
pub struct EdnsOption {
    udp_payload_size: u16,
    version: u8,
    dnssec_ok: bool,
    options: Vec<(EdnsCode, Vec<u8>)>,
}

impl EdnsOption {
    pub fn set_udp_payload_size(&mut self, size: u16) {
        self.udp_payload_size = size;
    }
    
    pub fn set_version(&mut self, version: u8) {
        self.version = version;
    }
    
    pub fn set_dnssec_ok(&mut self, dnssec_ok: bool) {
        self.dnssec_ok = dnssec_ok;
    }
    
    pub fn add_option(&mut self, code: EdnsCode, data: Vec<u8>) {
        self.options.push((code, data));
    }
}

/// 混合决策引擎
#[derive(Debug)]
pub struct HybridDecisionEngine {
    upstreams: Vec<UpstreamConfig>,
    metrics: Arc<RwLock<HashMap<String, PerformanceMetrics>>>,
    cdn_test_domains: Vec<String>,
    current_region: String,
}

/// 开箱即用的DNS解析器构造器
#[derive(Debug)]
pub struct DnsResolverBuilder {
    config: ResolverConfig,
    upstreams: Vec<UpstreamConfig>,
    quickmem_config: QuickMemConfig,
    query_strategy: QueryStrategy,
    enable_edns: bool,
    current_region: String,
}

/// DNS查询请求（序列化友好）
#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct DnsQueryRequest {
    /// 查询域名
    pub domain: String,
    /// 记录类型
    pub record_type: String,
    /// 客户端IP（可选）
    pub client_ip: Option<String>,
    /// 查询ID（用于追踪）
    pub query_id: Option<String>,
    /// 额外参数
    pub options: HashMap<String, String>,
}

/// DNS查询响应（序列化友好）
#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct DnsQueryResponse {
    /// 查询ID
    pub query_id: String,
    /// 查询域名
    pub domain: String,
    /// 记录类型
    pub record_type: String,
    /// 是否成功
    pub success: bool,
    /// 错误信息（如果失败）
    pub error: Option<String>,
    /// 解析结果
    pub records: Vec<DnsRecord>,
    /// 查询耗时（毫秒）
    pub duration_ms: u64,
    /// 使用的服务器
    pub server_used: Option<String>,
}

/// DNS记录（序列化友好）
#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct DnsRecord {
    /// 记录名称
    pub name: String,
    /// 记录类型
    pub record_type: String,
    /// TTL
    pub ttl: u32,
    /// 记录值
    pub value: String,
    /// 优先级（MX、SRV记录）
    pub priority: Option<u16>,
    /// 权重（SRV记录）
    pub weight: Option<u16>,
    /// 端口（SRV记录）
    pub port: Option<u16>,
}

impl HybridDecisionEngine {
    pub fn new(region: String) -> Self {
        Self {
            upstreams: Vec::new(),
            metrics: Arc::new(RwLock::new(HashMap::new())),
            cdn_test_domains: vec![
                "cdn.example.com".to_string(),
                "static.cloudflare.com".to_string(),
                "assets.amazonaws.com".to_string(),
            ],
            current_region: region,
        }
    }

    /// 添加上游服务器
    pub async fn add_upstream(&mut self, config: UpstreamConfig) {
        let mut metrics = self.metrics.write().await;
        metrics.insert(config.name.clone(), PerformanceMetrics::default());
        self.upstreams.push(config);
    }

    /// 智能选择最优上游服务器
    pub async fn select_best_upstream(&self) -> Option<UpstreamConfig> {
        let metrics = self.metrics.read().await;
        
        // 计算所有上游服务器的评分
        let mut scored_upstreams: Vec<(UpstreamConfig, f64)> = Vec::new();
        
        for upstream in &self.upstreams {
            if let Some(metric) = metrics.get(&upstream.name) {
                let score = self.calculate_upstream_score(upstream, metric);
                scored_upstreams.push((upstream.clone(), score));
            }
        }
        
        // 按评分排序
        scored_upstreams.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // 如果所有服务器都没有历史记录，使用轮询策略
        let total_queries: u64 = metrics.values().map(|m| m.total_queries).sum();
        if total_queries < 20 { // 前20次查询使用探索策略
            // 找到查询次数最少的服务器
            let min_queries = metrics.values().map(|m| m.total_queries).min().unwrap_or(0);
            let candidates: Vec<UpstreamConfig> = self.upstreams.iter()
                .filter(|upstream| {
                    metrics.get(&upstream.name)
                        .map(|m| m.total_queries == min_queries)
                        .unwrap_or(true)
                })
                .cloned()
                .collect();
            
            if !candidates.is_empty() {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                
                let mut hasher = DefaultHasher::new();
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default().as_nanos().hash(&mut hasher);
                let index = (hasher.finish() as usize) % candidates.len();
                return Some(candidates[index].clone());
            }
        }
        
        // 使用加权随机选择，给高分服务器更高概率，但仍保持多样性
        if !scored_upstreams.is_empty() {
            // 80%概率选择最优，20%概率选择其他
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            
            let mut hasher = DefaultHasher::new();
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default().as_nanos().hash(&mut hasher);
            let random_val = hasher.finish() as f64 / u64::MAX as f64;
            
            if random_val < 0.8 {
                Some(scored_upstreams[0].0.clone())
            } else if scored_upstreams.len() > 1 {
                let index = 1 + (random_val * (scored_upstreams.len() - 1) as f64) as usize;
                Some(scored_upstreams[index.min(scored_upstreams.len() - 1)].0.clone())
            } else {
                Some(scored_upstreams[0].0.clone())
            }
        } else {
            None
        }
    }

    /// 计算上游服务器综合评分
    fn calculate_upstream_score(&self, upstream: &UpstreamConfig, metrics: &PerformanceMetrics) -> f64 {
        // 基础权重
        let base_score = upstream.weight as f64;

        // 成功率权重 (40%)
        let success_rate = if metrics.total_queries > 0 {
            metrics.successful_queries as f64 / metrics.total_queries as f64
        } else {
            0.8 // 新服务器给予较高的初始评分
        };
        let success_component = base_score * 0.4 * success_rate;

        // 延迟权重 (30%) - 延迟越低分数越高
        let latency_score = if metrics.avg_latency.as_millis() > 0 {
            1000.0 / (metrics.avg_latency.as_millis() as f64 + 100.0)
        } else {
            1.0 // 新服务器给予默认延迟评分
        };
        let latency_component = base_score * 0.3 * latency_score;

        // CDN准确性权重 (20%)
        let cdn_score = if metrics.total_queries > 0 {
            metrics.cdn_accuracy_score
        } else {
            0.7 // 新服务器给予默认CDN评分
        };
        let cdn_component = base_score * 0.2 * cdn_score;

        // 连续失败惩罚 (10%)
        let failure_penalty = if metrics.consecutive_failures > 3 {
            0.1 // 严重惩罚
        } else if metrics.consecutive_failures > 0 {
            1.0 - (metrics.consecutive_failures as f64 * 0.2)
        } else {
            1.0
        };
        let penalty_component = base_score * 0.1 * failure_penalty;

        let mut total_score = success_component + latency_component + cdn_component + penalty_component;

        // 最近成功时间加成
        if let Some(last_success) = metrics.last_success_time {
            let time_since_success = Instant::now().duration_since(last_success);
            if time_since_success < Duration::from_secs(60) {
                total_score *= 1.1; // 最近成功的服务器加成
            }
        }

        total_score
    }

    /// 更新性能指标
    pub async fn update_metrics(&self, upstream_name: &str, latency: Duration, success: bool, cdn_accurate: bool) {
        let mut metrics = self.metrics.write().await;
        if let Some(metric) = metrics.get_mut(upstream_name) {
            metric.total_queries += 1;
            
            if success {
                metric.successful_queries += 1;
                metric.last_success_time = Some(Instant::now());
                metric.consecutive_failures = 0;
                
                // 更新平均延迟
                if metric.total_queries == 1 {
                    metric.avg_latency = latency;
                } else {
                    let total_latency = metric.avg_latency.as_nanos() as f64 * (metric.total_queries - 1) as f64;
                    metric.avg_latency = Duration::from_nanos(
                        ((total_latency + latency.as_nanos() as f64) / metric.total_queries as f64) as u64
                    );
                }
                
                // 更新CDN准确性评分
                let current_score = metric.cdn_accuracy_score * (metric.successful_queries - 1) as f64;
                let new_score = if cdn_accurate { 1.0 } else { 0.0 };
                metric.cdn_accuracy_score = (current_score + new_score) / metric.successful_queries as f64;
            } else {
                metric.failed_queries += 1;
                metric.consecutive_failures += 1;
            }
        }
    }

    /// 获取所有上游服务器
    pub fn get_upstreams(&self) -> &[UpstreamConfig] {
        &self.upstreams
    }
}

/// 开箱即用的DNS解析器
#[derive(Debug, Clone)]
pub struct EasyDnsResolver {
    resolver: Resolver,
    quickmem_config: QuickMemConfig,
    decision_engine: Option<Arc<HybridDecisionEngine>>,
    query_strategy: QueryStrategy,
    enable_edns: bool,
}

impl Default for DnsResolverBuilder {
    fn default() -> Self {
        Self {
            config: ResolverConfig::default(),
            upstreams: Vec::new(),
            quickmem_config: QuickMemConfig {
                max_data_size: 10 * 1024 * 1024, // 10MB
                max_batch_count: 1000,
                pool_initial_capacity: 1024,
                pool_max_capacity: 10 * 1024 * 1024, // 10MB
                enable_parallel: true,
            },
            query_strategy: QueryStrategy::Fifo,
            enable_edns: true,
            current_region: "default".to_string(),
        }
    }
}

impl DnsResolverBuilder {
    /// 创建新的构造器
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置查询策略
    pub fn query_strategy(mut self, strategy: QueryStrategy) -> Self {
        self.query_strategy = strategy;
        self
    }

    /// 启用/禁用EDNS
    pub fn enable_edns(mut self, enable: bool) -> Self {
        self.enable_edns = enable;
        self
    }

    /// 设置当前区域
    pub fn region(mut self, region: String) -> Self {
        self.current_region = region;
        self
    }

    /// 添加UDP上游服务器
    pub fn add_udp_upstream(mut self, name: String, addr: SocketAddr, weight: u32) -> Self {
        self.upstreams.push(UpstreamConfig {
            name,
            transport_type: TransportType::Udp,
            address: addr,
            url: None,
            weight,
            expected_region: None,
        });
        self
    }

    /// 添加TCP上游服务器
    pub fn add_tcp_upstream(mut self, name: String, addr: SocketAddr, weight: u32) -> Self {
        self.upstreams.push(UpstreamConfig {
            name,
            transport_type: TransportType::Tcp,
            address: addr,
            url: None,
            weight,
            expected_region: None,
        });
        self
    }

    /// 添加DoH上游服务器
    pub fn add_doh_upstream(mut self, name: String, addr: SocketAddr, url: String, weight: u32) -> Self {
        self.upstreams.push(UpstreamConfig {
            name,
            transport_type: TransportType::Https,
            address: addr,
            url: Some(url),
            weight,
            expected_region: None,
        });
        self
    }

    /// 添加DoT上游服务器
    pub fn add_dot_upstream(mut self, name: String, addr: SocketAddr, weight: u32) -> Self {
        self.upstreams.push(UpstreamConfig {
            name,
            transport_type: TransportType::Tls,
            address: addr,
            url: None,
            weight,
            expected_region: None,
        });
        self
    }

    /// 添加自定义上游配置
    pub fn add_upstream(mut self, config: UpstreamConfig) -> Self {
        self.upstreams.push(config);
        self
    }

    /// 批量添加上游服务器
    pub fn add_upstreams(mut self, configs: Vec<UpstreamConfig>) -> Self {
        self.upstreams.extend(configs);
        self
    }
    
    /// 添加常用的公共DNS服务器
    pub fn with_public_dns(mut self) -> Self {
        // 国内DNS服务器
        self = self.add_udp_upstream("阿里DNS".to_string(), "223.5.5.5:53".parse().unwrap(), 100);
        self = self.add_udp_upstream("腾讯DNS".to_string(), "119.29.29.29:53".parse().unwrap(), 100);
        self = self.add_udp_upstream("114DNS".to_string(), "114.114.114.114:53".parse().unwrap(), 80);
        
        // 国际DNS服务器
        self = self.add_udp_upstream("Google DNS".to_string(), "8.8.8.8:53".parse().unwrap(), 120);
        self = self.add_udp_upstream("Cloudflare DNS".to_string(), "1.1.1.1:53".parse().unwrap(), 120);
        
        // DoH服务器
        self = self.add_doh_upstream("阿里DoH".to_string(), "223.5.5.5:443".parse().unwrap(), "https://dns.alidns.com/dns-query".to_string(), 110);
        self = self.add_doh_upstream("腾讯DoH".to_string(), "1.12.12.12:443".parse().unwrap(), "https://doh.pub/dns-query".to_string(), 110);
        self = self.add_doh_upstream("Cloudflare DoH".to_string(), "1.1.1.1:443".parse().unwrap(), "https://cloudflare-dns.com/dns-query".to_string(), 130);
        
        // DoT服务器
        self = self.add_dot_upstream("阿里DoT".to_string(), "223.5.5.5:853".parse().unwrap(), 105);
        self = self.add_dot_upstream("腾讯DoT".to_string(), "1.12.12.12:853".parse().unwrap(), 105);
        
        self
    }
    
    /// 设置查询超时时间
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.config.default_timeout = timeout;
        self
    }
    
    /// 设置重试次数
    pub fn with_retry_count(mut self, count: usize) -> Self {
        self.config.retry_count = count;
        self
    }
    
    /// 启用/禁用缓存
    pub fn with_cache(mut self, enable: bool) -> Self {
        self.config.enable_cache = enable;
        self
    }
    
    /// 设置缓存TTL上限
    pub fn with_cache_ttl(mut self, ttl: Duration) -> Self {
        self.config.max_cache_ttl = ttl;
        self
    }
    
    /// 启用/禁用健康检查
    pub fn with_health_check(mut self, enable: bool) -> Self {
        self.config.enable_health_check = enable;
        self
    }
    
    /// 设置QuickMem配置
    pub fn with_quickmem_config(mut self, config: QuickMemConfig) -> Self {
        self.quickmem_config = config;
        self
    }
    
    /// 构建解析器
    pub async fn build(self) -> Result<EasyDnsResolver> {
        let mut resolver = Resolver::new(self.config);
        
        // 根据查询策略构建不同的解析器
        let decision_engine = match self.query_strategy {
            QueryStrategy::Fifo => {
                // FIFO模式：添加所有上游服务器到resolver
                for upstream in &self.upstreams {
                    match upstream.transport_type {
                        TransportType::Udp => {
                            let config = TransportConfig {
                                server: upstream.address.ip().to_string(),
                                port: upstream.address.port(),
                                timeout: Duration::from_secs(5),
                                tcp_fast_open: false,
                                tcp_nodelay: true,
                                pool_size: 10,
                            };
                            resolver.add_udp_transport(config);
                        },
                        TransportType::Tcp => {
                            let config = TransportConfig {
                                server: upstream.address.ip().to_string(),
                                port: upstream.address.port(),
                                timeout: Duration::from_secs(5),
                                tcp_fast_open: false,
                                tcp_nodelay: true,
                                pool_size: 10,
                            };
                            resolver.add_tcp_transport(config);
                        },
                        TransportType::Https => {
                            if let Some(url) = &upstream.url {
                                let config = HttpsConfig {
                                    base: TransportConfig {
                                        server: upstream.address.ip().to_string(),
                                        port: upstream.address.port(),
                                        timeout: Duration::from_secs(10),
                                        tcp_fast_open: false,
                                        tcp_nodelay: true,
                                        pool_size: 10,
                                    },
                                    url: url.clone(),
                                    method: crate::transport::HttpMethod::POST,
                                    user_agent: "RatQuickDNS/1.0".to_string(),
                                };
                                resolver.add_https_transport(config)?;
                            }
                        },
                        TransportType::Tls => {
                            let config = TlsConfig {
                                base: TransportConfig {
                                    server: upstream.address.ip().to_string(),
                                    port: upstream.address.port(),
                                    timeout: Duration::from_secs(10),
                                    tcp_fast_open: false,
                                    tcp_nodelay: true,
                                    pool_size: 10,
                                },
                                server_name: upstream.address.ip().to_string(),
                                verify_cert: true,
                            };
                            resolver.add_tls_transport(config)?;
                        }
                    }
                }
                None
            },
            QueryStrategy::Smart => {
                // 智能决策模式：创建决策引擎
                let mut engine = HybridDecisionEngine::new(self.current_region.clone());
                
                // 添加所有上游服务器到决策引擎
                for upstream in self.upstreams {
                    engine.add_upstream(upstream).await;
                }
                
                // 为智能模式添加一个默认的resolver配置（用于fallback）
                if !engine.get_upstreams().is_empty() {
                    let first_upstream = &engine.get_upstreams()[0];
                    let config = TransportConfig {
                        server: first_upstream.address.ip().to_string(),
                        port: first_upstream.address.port(),
                        timeout: Duration::from_secs(5),
                        tcp_fast_open: false,
                        tcp_nodelay: true,
                        pool_size: 10,
                    };
                    resolver.add_udp_transport(config);
                }
                
                Some(Arc::new(engine))
            }
        };
        
        Ok(EasyDnsResolver {
            resolver,
            quickmem_config: self.quickmem_config,
            decision_engine,
            query_strategy: self.query_strategy,
            enable_edns: self.enable_edns,
        })
    }
}

impl EasyDnsResolver {
    /// 创建默认的DNS解析器（包含常用公共DNS）
    pub async fn default() -> Result<Self> {
        DnsResolverBuilder::new()
            .with_public_dns()
            .build()
            .await
    }
    
    /// 创建快速配置的DNS解析器
    pub async fn quick_setup() -> Result<Self> {
        DnsResolverBuilder::new()
            .with_public_dns()
            .with_timeout(Duration::from_secs(3))
            .with_retry_count(2)
            .with_cache(true)
            .with_health_check(true)
            .build()
            .await
    }
    
    /// 启动健康检查进程（仅智能决策模式有效）
    pub async fn start_health_check(&self, interval: Duration) -> Result<()> {
        if let Some(engine) = &self.decision_engine {
            let engine_clone = engine.clone();
            
            // 启动健康检查任务
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(interval);
                loop {
                    interval.tick().await;
                    Self::perform_health_check(&engine_clone).await;
                }
            });
            
            Ok(())
        } else {
            Err(DnsError::Config("Health check is only available in Smart query mode".to_string()))
        }
    }
    
    /// 执行健康检查
    async fn perform_health_check(engine: &Arc<HybridDecisionEngine>) {
        let upstreams = engine.get_upstreams().to_vec();
        
        for upstream in upstreams {
            let start = Instant::now();
            let test_domain = "google.com";
            
            // 创建临时解析器进行健康检查
            let mut resolver = Resolver::new(ResolverConfig::default());
            
            let success = match upstream.transport_type {
                TransportType::Udp => {
                    let config = TransportConfig {
                        server: upstream.address.ip().to_string(),
                        port: upstream.address.port(),
                        timeout: Duration::from_secs(2),
                        tcp_fast_open: false,
                        tcp_nodelay: true,
                        pool_size: 1,
                    };
                    resolver.add_udp_transport(config);
                    
                    match resolver.query(test_domain, RecordType::A, QClass::IN).await {
                        Ok(_) => {
                            let latency = start.elapsed();
                            engine.update_metrics(&upstream.name, latency, true, true).await;
                            true
                        },
                        Err(_) => {
                            engine.update_metrics(&upstream.name, Duration::from_secs(10), false, false).await;
                            false
                        }
                    }
                },
                _ => {
                    // 其他传输类型的健康检查实现
                    engine.update_metrics(&upstream.name, Duration::from_secs(10), false, false).await;
                    false
                }
            };
        }
    }
    
    /// 解析域名（简化接口）
    pub async fn resolve(&self, domain: &str) -> Result<Vec<IpAddr>> {
        match self.query_strategy {
            QueryStrategy::Fifo => {
                // FIFO模式：直接使用resolver查询
                let response = self.resolver.query(domain, RecordType::A, QClass::IN).await?;
                
                let mut ips = Vec::new();
                for record in response.answers {
                    match record.data {
                        crate::types::RecordData::A(ip) => ips.push(IpAddr::V4(ip)),
                        crate::types::RecordData::AAAA(ip) => ips.push(IpAddr::V6(ip)),
                        _ => {}
                    }
                }
                
                Ok(ips)
            },
            QueryStrategy::Smart => {
                if let Some(engine) = &self.decision_engine {
                    // 智能决策模式：选择最佳上游服务器
                    if let Some(upstream) = engine.select_best_upstream().await {
                        let start = Instant::now();
                        
                        // 创建临时解析器
                        let mut resolver = Resolver::new(ResolverConfig::default());
                        
                        let result = match upstream.transport_type {
                            TransportType::Udp => {
                                let config = TransportConfig {
                                    server: upstream.address.ip().to_string(),
                                    port: upstream.address.port(),
                                    timeout: Duration::from_secs(5),
                                    tcp_fast_open: false,
                                    tcp_nodelay: true,
                                    pool_size: 10,
                                };
                                resolver.add_udp_transport(config);
                                resolver.query(domain, RecordType::A, QClass::IN).await
                            },
                            _ => {
                                // 其他传输类型的实现
                                self.resolver.query(domain, RecordType::A, QClass::IN).await
                            }
                        };
                        
                        // 更新性能指标
                        let latency = start.elapsed();
                        let success = result.is_ok();
                        engine.update_metrics(&upstream.name, latency, success, true).await;
                        
                        match result {
                            Ok(response) => {
                                let mut ips = Vec::new();
                                for record in response.answers {
                                    match record.data {
                                        crate::types::RecordData::A(ip) => ips.push(IpAddr::V4(ip)),
                                        crate::types::RecordData::AAAA(ip) => ips.push(IpAddr::V6(ip)),
                                        _ => {}
                                    }
                                }
                                Ok(ips)
                            },
                            Err(e) => Err(e)
                        }
                    } else {
                        // 没有可用的上游服务器，使用默认resolver
                        let response = self.resolver.query(domain, RecordType::A, QClass::IN).await?;
                        
                        let mut ips = Vec::new();
                        for record in response.answers {
                            match record.data {
                                crate::types::RecordData::A(ip) => ips.push(IpAddr::V4(ip)),
                                crate::types::RecordData::AAAA(ip) => ips.push(IpAddr::V6(ip)),
                                _ => {}
                            }
                        }
                        
                        Ok(ips)
                    }
                } else {
                    // 决策引擎未初始化，使用默认resolver
                    let response = self.resolver.query(domain, RecordType::A, QClass::IN).await?;
                    
                    let mut ips = Vec::new();
                    for record in response.answers {
                        match record.data {
                            crate::types::RecordData::A(ip) => ips.push(IpAddr::V4(ip)),
                            crate::types::RecordData::AAAA(ip) => ips.push(IpAddr::V6(ip)),
                            _ => {}
                        }
                    }
                    
                    Ok(ips)
                }
            }
        }
    }
    
    /// 解析域名（指定记录类型）
    pub async fn resolve_type(&self, domain: &str, record_type: &str) -> Result<Vec<String>> {
        let rtype = match record_type.to_uppercase().as_str() {
            "A" => RecordType::A,
            "AAAA" => RecordType::AAAA,
            "CNAME" => RecordType::CNAME,
            "MX" => RecordType::MX,
            "NS" => RecordType::NS,
            "TXT" => RecordType::TXT,
            "PTR" => RecordType::PTR,
            "SRV" => RecordType::SRV,
            _ => return Err(DnsError::Config(format!("不支持的记录类型: {}", record_type))),
        };
        
        let response = self.resolver.query(domain, rtype, QClass::IN).await?;
        
        let mut results = Vec::new();
        for record in response.answers {
            let value = match record.data {
                crate::types::RecordData::A(ip) => ip.to_string(),
                crate::types::RecordData::AAAA(ip) => ip.to_string(),
                crate::types::RecordData::CNAME(name) => name,
                crate::types::RecordData::NS(name) => name,
                crate::types::RecordData::PTR(name) => name,
                crate::types::RecordData::MX { exchange, .. } => exchange,
                crate::types::RecordData::TXT(texts) => texts.join(" "),
                crate::types::RecordData::SRV { target, .. } => target,
                _ => continue,
            };
            results.push(value);
        }
        
        Ok(results)
    }
    
    /// 处理序列化的查询请求（bincode2 + rat_quickmem）
    pub async fn process_encoded_query(&self, encoded_data: &[u8]) -> Result<Vec<u8>> {
        // 使用rat_quickmem解码请求
        let request: DnsQueryRequest = decode(encoded_data)
            .map_err(|e| DnsError::Config(format!("解码请求失败: {}", e)))?;
        
        let start_time = std::time::Instant::now();
        
        // 执行DNS查询
        let result = self.resolve_type(&request.domain, &request.record_type).await;
        
        let duration_ms = start_time.elapsed().as_millis() as u64;
        
        // 构建响应
        let response = match result {
            Ok(values) => {
                let records = values.into_iter().map(|value| DnsRecord {
                    name: request.domain.clone(),
                    record_type: request.record_type.clone(),
                    ttl: 300, // 默认TTL
                    value,
                    priority: None,
                    weight: None,
                    port: None,
                }).collect();
                
                DnsQueryResponse {
                    query_id: request.query_id.unwrap_or_else(|| "unknown".to_string()),
                    domain: request.domain.clone(),
                    record_type: request.record_type.clone(),
                    success: true,
                    error: None,
                    records,
                    duration_ms,
                    server_used: Some("mixed".to_string()),
                }
            }
            Err(e) => DnsQueryResponse {
                query_id: request.query_id.unwrap_or_else(|| "unknown".to_string()),
                domain: request.domain.clone(),
                record_type: request.record_type.clone(),
                success: false,
                error: Some(e.to_string()),
                records: Vec::new(),
                duration_ms,
                server_used: None,
            }
        };
        
        // 使用rat_quickmem编码响应
        encode(&response)
            .map(|bytes| bytes.to_vec())
            .map_err(|e| DnsError::Config(format!("编码响应失败: {}", e)))
    }
    
    /// 批量查询（并发执行）
    pub async fn batch_query(&self, domains: Vec<String>) -> Vec<Result<Vec<IpAddr>>> {
        let mut handles = Vec::with_capacity(domains.len());
        
        for domain in domains {
            let resolver_clone = self.clone();
            let handle = tokio::spawn(async move {
                resolver_clone.resolve(&domain).await
            });
            handles.push(handle);
        }
        
        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => results.push(Err(DnsError::Config(format!("Join error: {}", e)))),
            }
        }
        
        results
    }
    
    /// 批量处理查询请求
    pub async fn process_batch_queries(&self, requests: Vec<DnsQueryRequest>) -> Result<Vec<DnsQueryResponse>> {
        let mut handles = Vec::with_capacity(requests.len());
        
        for request in requests {
            let resolver_clone = self.clone();
            let handle = tokio::spawn(async move {
                let start_time = std::time::Instant::now();
                let result = resolver_clone.resolve_type(&request.domain, &request.record_type).await;
                let duration_ms = start_time.elapsed().as_millis() as u64;
                
                match result {
                    Ok(values) => {
                        let records = values.into_iter().map(|value| DnsRecord {
                            name: request.domain.clone(),
                            record_type: request.record_type.clone(),
                            ttl: 300,
                            value,
                            priority: None,
                            weight: None,
                            port: None,
                        }).collect();
                        
                        DnsQueryResponse {
                            query_id: request.query_id.unwrap_or_else(|| "unknown".to_string()),
                            domain: request.domain.clone(),
                            record_type: request.record_type.clone(),
                            success: true,
                            error: None,
                            records,
                            duration_ms,
                            server_used: Some("mixed".to_string()),
                        }
                    }
                    Err(e) => DnsQueryResponse {
                        query_id: request.query_id.unwrap_or_else(|| "unknown".to_string()),
                        domain: request.domain.clone(),
                        record_type: request.record_type.clone(),
                        success: false,
                        error: Some(e.to_string()),
                        records: Vec::new(),
                        duration_ms,
                        server_used: None,
                    }
                }
            });
            handles.push(handle);
        }
        
        let mut responses = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(response) => responses.push(response),
                Err(e) => {
                    responses.push(DnsQueryResponse {
                        query_id: "error".to_string(),
                        domain: "unknown".to_string(),
                        record_type: "A".to_string(),
                        success: false,
                        error: Some(format!("Join error: {}", e)),
                        records: Vec::new(),
                        duration_ms: 0,
                        server_used: None,
                    });
                }
            }
        }
        
        Ok(responses)
    }
    
    /// 获取QuickMem配置
    pub fn quickmem_config(&self) -> &QuickMemConfig {
        &self.quickmem_config
    }
}

/// 便捷函数：创建DNS查询请求
pub fn create_dns_query(domain: &str, record_type: &str) -> DnsQueryRequest {
    DnsQueryRequest {
        domain: domain.to_string(),
        record_type: record_type.to_string(),
        client_ip: None,
        query_id: Some(Uuid::new_v4().to_string()),
        options: HashMap::new(),
    }
}

/// 便捷函数：编码DNS查询请求
pub fn encode_dns_query(request: &DnsQueryRequest) -> Result<Vec<u8>> {
    encode(request)
        .map(|bytes| bytes.to_vec())
        .map_err(|e| DnsError::Config(format!("编码查询请求失败: {}", e)))
}

/// 便捷函数：解码DNS查询响应
pub fn decode_dns_response(data: &[u8]) -> Result<DnsQueryResponse> {
    decode(data)
        .map_err(|e| DnsError::Config(format!("解码查询响应失败: {}", e)))
}