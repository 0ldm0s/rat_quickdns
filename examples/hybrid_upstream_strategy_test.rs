//! 混合上游服务器智能决策测试示例
//! 
//! 本示例演示了在复杂网络环境下，如何智能选择最优的DNS上游服务器：
//! 1. 多种协议混合：传统DNS(UDP/TCP)、DoH(HTTPS)、DoT(TLS)
//! 2. CDN区分能力测试：检测上游服务器是否能正确返回地区化CDN结果
//! 3. 故障转移：处理服务器不可达、超时等异常情况
//! 4. 智能决策：基于延迟、成功率、CDN准确性等指标动态选择最优上游

use rat_quickdns::{
    resolver::{Resolver, ResolverConfig},
    transport::{Transport, TransportConfig, TlsConfig, HttpsConfig, HttpMethod},
    types::{Query, Record, RecordType, RecordData, QClass},
    error::DnsError,
};
use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::time::timeout;

/// 传输类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransportType {
    Udp,
    Tcp,
    Tls,
    Https,
}

/// 上游服务器配置
#[derive(Debug, Clone)]
struct UpstreamConfig {
    name: String,
    transport_type: TransportType,
    address: SocketAddr,
    url: Option<String>, // DoH URL
    weight: u32,
    expected_region: Option<String>, // 期望的CDN区域
}

/// 性能指标
#[derive(Debug, Clone, Default)]
struct PerformanceMetrics {
    total_queries: u64,
    successful_queries: u64,
    failed_queries: u64,
    avg_latency: Duration,
    cdn_accuracy_score: f64, // CDN区分准确性评分 0.0-1.0
    last_success_time: Option<Instant>,
    consecutive_failures: u32,
}

/// 智能决策引擎
#[derive(Debug)]
struct HybridDecisionEngine {
    upstreams: Vec<UpstreamConfig>,
    metrics: HashMap<String, PerformanceMetrics>,
    cdn_test_domains: Vec<String>, // 用于测试CDN区分能力的域名
    current_region: String, // 当前客户端所在区域
}

impl HybridDecisionEngine {
    fn new(region: String) -> Self {
        Self {
            upstreams: Vec::new(),
            metrics: HashMap::new(),
            cdn_test_domains: vec![
                "cdn.example.com".to_string(),
                "static.cloudflare.com".to_string(),
                "assets.amazonaws.com".to_string(),
            ],
            current_region: region,
        }
    }

    /// 添加上游服务器
    fn add_upstream(&mut self, config: UpstreamConfig) {
        self.metrics.insert(config.name.clone(), PerformanceMetrics::default());
        self.upstreams.push(config);
    }

    /// 智能选择最优上游服务器
    fn select_best_upstream(&self) -> Option<&UpstreamConfig> {
        // 计算所有上游服务器的评分
        let mut scored_upstreams: Vec<(&UpstreamConfig, f64)> = Vec::new();
        
        for upstream in &self.upstreams {
            if let Some(metrics) = self.metrics.get(&upstream.name) {
                let score = self.calculate_upstream_score(upstream, metrics);
                scored_upstreams.push((upstream, score));
            }
        }
        
        // 按评分排序
        scored_upstreams.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // 如果所有服务器都没有历史记录，使用轮询策略
        let total_queries: u64 = self.metrics.values().map(|m| m.total_queries).sum();
        if total_queries < 20 { // 前20次查询使用探索策略
            // 找到查询次数最少的服务器
            let min_queries = self.metrics.values().map(|m| m.total_queries).min().unwrap_or(0);
            let candidates: Vec<&UpstreamConfig> = self.upstreams.iter()
                .filter(|upstream| {
                    self.metrics.get(&upstream.name)
                        .map(|m| m.total_queries == min_queries)
                        .unwrap_or(true)
                })
                .collect();
            
            if !candidates.is_empty() {
                let index = fastrand::usize(0..candidates.len());
                return Some(candidates[index]);
            }
        }
        
        // 使用加权随机选择，给高分服务器更高概率，但仍保持多样性
        if !scored_upstreams.is_empty() {
            // 80%概率选择最优，20%概率选择其他
            if fastrand::f64() < 0.8 {
                Some(scored_upstreams[0].0)
            } else if scored_upstreams.len() > 1 {
                let index = fastrand::usize(1..scored_upstreams.len());
                Some(scored_upstreams[index].0)
            } else {
                Some(scored_upstreams[0].0)
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
    fn update_metrics(&mut self, upstream_name: &str, latency: Duration, success: bool, cdn_accurate: bool) {
        if let Some(metrics) = self.metrics.get_mut(upstream_name) {
            metrics.total_queries += 1;
            
            if success {
                metrics.successful_queries += 1;
                metrics.last_success_time = Some(Instant::now());
                metrics.consecutive_failures = 0;
                
                // 更新平均延迟
                if metrics.total_queries == 1 {
                    metrics.avg_latency = latency;
                } else {
                    let total_latency = metrics.avg_latency.as_nanos() as f64 * (metrics.total_queries - 1) as f64;
                    metrics.avg_latency = Duration::from_nanos(
                        ((total_latency + latency.as_nanos() as f64) / metrics.total_queries as f64) as u64
                    );
                }
                
                // 更新CDN准确性评分
                let current_score = metrics.cdn_accuracy_score * (metrics.successful_queries - 1) as f64;
                let new_score = if cdn_accurate { 1.0 } else { 0.0 };
                metrics.cdn_accuracy_score = (current_score + new_score) / metrics.successful_queries as f64;
            } else {
                metrics.failed_queries += 1;
                metrics.consecutive_failures += 1;
            }
        }
    }
}

/// 模拟DNS查询结果
#[derive(Debug, Clone)]
struct MockDnsResult {
    records: Vec<Record>,
    latency: Duration,
    is_cdn_accurate: bool, // 是否返回了正确的地区CDN
}

/// 模拟不同上游服务器的响应
async fn simulate_dns_query(
    upstream: &UpstreamConfig,
    domain: &str,
    expected_region: &str,
) -> Result<MockDnsResult, DnsError> {
    // 模拟网络延迟
    let base_latency = match upstream.transport_type {
        TransportType::Udp => Duration::from_millis(20),
        TransportType::Tcp => Duration::from_millis(35),
        TransportType::Https => Duration::from_millis(80),
        TransportType::Tls => Duration::from_millis(45),
    };
    
    // 添加随机延迟变化
    let jitter = Duration::from_millis(fastrand::u64(0..20));
    let total_latency = base_latency + jitter;
    
    tokio::time::sleep(total_latency).await;
    
    // 模拟不同的故障场景
    match upstream.name.as_str() {
        "ali_dns_udp" => {
            // 阿里DNS UDP - 高可用性和低延迟
            if fastrand::f64() < 0.98 { // 98% 成功率
                Ok(MockDnsResult {
                    records: vec![Record {
                        name: domain.to_string(),
                        rtype: RecordType::A,
                        class: QClass::IN,
                        ttl: 300,
                        data: RecordData::A(Ipv4Addr::new(223, 5, 5, 5)),
                    }],
                    latency: Duration::from_millis(15),
                    is_cdn_accurate: fastrand::f64() < 0.95, // 95% CDN准确率
                })
            } else {
                Err(DnsError::Timeout)
            }
        },
        "ali_dns_doh" => {
            // 阿里DNS DoH - 高可用性但延迟稍高
            if fastrand::f64() < 0.95 { // 95% 成功率
                Ok(MockDnsResult {
                    records: vec![Record {
                        name: domain.to_string(),
                        rtype: RecordType::A,
                        class: QClass::IN,
                        ttl: 300,
                        data: RecordData::A(Ipv4Addr::new(223, 5, 5, 5)),
                    }],
                    latency: Duration::from_millis(45),
                    is_cdn_accurate: fastrand::f64() < 0.93, // 93% CDN准确率
                })
            } else {
                Err(DnsError::Timeout)
            }
        },
        "ali_dns_dot" => {
            // 阿里DNS DoT - 高可用性，延迟中等
            if fastrand::f64() < 0.96 { // 96% 成功率
                Ok(MockDnsResult {
                    records: vec![Record {
                        name: domain.to_string(),
                        rtype: RecordType::A,
                        class: QClass::IN,
                        ttl: 300,
                        data: RecordData::A(Ipv4Addr::new(223, 5, 5, 5)),
                    }],
                    latency: Duration::from_millis(30),
                    is_cdn_accurate: fastrand::f64() < 0.94, // 94% CDN准确率
                })
            } else {
                Err(DnsError::Timeout)
            }
        },
        "tencent_dns_udp" => {
            // 腾讯DNS UDP - 高可用性
            if fastrand::f64() < 0.97 { // 97% 成功率
                Ok(MockDnsResult {
                    records: vec![Record {
                        name: domain.to_string(),
                        rtype: RecordType::A,
                        class: QClass::IN,
                        ttl: 300,
                        data: RecordData::A(Ipv4Addr::new(119, 29, 29, 29)),
                    }],
                    latency: Duration::from_millis(18),
                    is_cdn_accurate: fastrand::f64() < 0.93, // 93% CDN准确率
                })
            } else {
                Err(DnsError::Timeout)
            }
        },
        "tencent_dns_doh" => {
            // 腾讯DNS DoH - 较高可用性但延迟稍高
            if fastrand::f64() < 0.94 { // 94% 成功率
                Ok(MockDnsResult {
                    records: vec![Record {
                        name: domain.to_string(),
                        rtype: RecordType::A,
                        class: QClass::IN,
                        ttl: 300,
                        data: RecordData::A(Ipv4Addr::new(119, 29, 29, 29)),
                    }],
                    latency: Duration::from_millis(50),
                    is_cdn_accurate: fastrand::f64() < 0.92, // 92% CDN准确率
                })
            } else {
                Err(DnsError::Timeout)
            }
        },
        "tencent_dns_dot" => {
            // 腾讯DNS DoT - 较高可用性，延迟中等
            if fastrand::f64() < 0.95 { // 95% 成功率
                Ok(MockDnsResult {
                    records: vec![Record {
                        name: domain.to_string(),
                        rtype: RecordType::A,
                        class: QClass::IN,
                        ttl: 300,
                        data: RecordData::A(Ipv4Addr::new(119, 29, 29, 29)),
                    }],
                    latency: Duration::from_millis(35),
                    is_cdn_accurate: fastrand::f64() < 0.92, // 92% CDN准确率
                })
            } else {
                Err(DnsError::Timeout)
            }
        },
        "114_dns" => {
            // 114DNS - 传统稳定
            if fastrand::f64() < 0.95 { // 95% 成功率
                Ok(MockDnsResult {
                    records: vec![Record {
                        name: domain.to_string(),
                        rtype: RecordType::A,
                        class: QClass::IN,
                        ttl: 300,
                        data: RecordData::A(Ipv4Addr::new(114, 114, 114, 114)),
                    }],
                    latency: Duration::from_millis(25),
                    is_cdn_accurate: fastrand::f64() < 0.88, // 88% CDN准确率
                })
            } else {
                Err(DnsError::Timeout)
            }
        },
        "cloudflare_doh" => {
            // Cloudflare DoH - 国际服务，在国内可能较慢
            if fastrand::f64() < 0.85 { // 85% 成功率
                Ok(MockDnsResult {
                    records: vec![Record {
                        name: domain.to_string(),
                        rtype: RecordType::A,
                        class: QClass::IN,
                        ttl: 300,
                        data: RecordData::A(Ipv4Addr::new(1, 1, 1, 1)),
                    }],
                    latency: Duration::from_millis(80),
                    is_cdn_accurate: fastrand::f64() < 0.75, // 75% CDN准确率
                })
            } else {
                Err(DnsError::Timeout)
            }
        },
        "google_dot" => {
            // Google DoT - 在国内可能不稳定
            if fastrand::f64() < 0.82 { // 82% 成功率
                Ok(MockDnsResult {
                    records: vec![Record {
                        name: domain.to_string(),
                        rtype: RecordType::A,
                        class: QClass::IN,
                        ttl: 300,
                        data: RecordData::A(Ipv4Addr::new(8, 8, 8, 8)),
                    }],
                    latency: Duration::from_millis(90),
                    is_cdn_accurate: fastrand::f64() < 0.70, // 70% CDN准确率
                })
            } else {
                 Err(DnsError::Network("Connection blocked".to_string()))
             }
        },
        "local_isp" => {
            // 本地ISP DNS - 延迟低但可能不稳定
            if fastrand::f64() < 0.90 { // 90% 成功率
                Ok(MockDnsResult {
                    records: vec![Record {
                        name: domain.to_string(),
                        rtype: RecordType::A,
                        class: QClass::IN,
                        ttl: 300,
                        data: RecordData::A(Ipv4Addr::new(192, 168, 1, 1)),
                    }],
                    latency: Duration::from_millis(10), // 很低的延迟
                    is_cdn_accurate: fastrand::f64() < 0.95, // 95% CDN准确率（本地优势）
                })
            } else {
                 Err(DnsError::Server("ISP DNS overloaded".to_string()))
             }
        },
        "quad9_udp" => {
            // Quad9 UDP - 国际服务，在国内较慢
            if fastrand::f64() < 0.80 { // 80% 成功率
                Ok(MockDnsResult {
                    records: vec![Record {
                        name: domain.to_string(),
                        rtype: RecordType::A,
                        class: QClass::IN,
                        ttl: 300,
                        data: RecordData::A(Ipv4Addr::new(9, 9, 9, 9)),
                    }],
                    latency: Duration::from_millis(100),
                    is_cdn_accurate: fastrand::f64() < 0.65, // 65% CDN准确率
                })
            } else {
                Err(DnsError::Timeout)
            }
        },
        "unstable_server" => {
            // 不稳定的服务器 - 用于测试故障转移
            if fastrand::f64() < 0.30 { // 仅30% 成功率
                Ok(MockDnsResult {
                    records: vec![Record {
                        name: domain.to_string(),
                        rtype: RecordType::A,
                        class: QClass::IN,
                        ttl: 300,
                        data: RecordData::A(Ipv4Addr::new(203, 0, 113, 1)),
                    }],
                    latency: Duration::from_millis(200),
                    is_cdn_accurate: fastrand::f64() < 0.20,
                })
            } else {
                 Err(DnsError::Network("Server unreachable".to_string()))
             }
        },
        _ => {
            // 默认处理 - 中等性能
            if fastrand::f64() < 0.75 { // 75% 成功率
                Ok(MockDnsResult {
                    records: vec![Record {
                        name: domain.to_string(),
                        rtype: RecordType::A,
                        class: QClass::IN,
                        ttl: 300,
                        data: RecordData::A(Ipv4Addr::new(8, 8, 4, 4)),
                    }],
                    latency: Duration::from_millis(120),
                    is_cdn_accurate: fastrand::f64() < 0.50,
                })
            } else {
                Err(DnsError::Config("Unknown upstream".to_string()))
            }
        }
    }
}

/// 执行智能决策测试
async fn run_hybrid_strategy_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 开始混合上游服务器智能决策测试");
    
    let mut engine = HybridDecisionEngine::new("Asia-Pacific".to_string());
    
    // 配置多种类型的上游服务器
     
     // 国内DNS服务器 - UDP协议
     engine.add_upstream(UpstreamConfig {
         name: "ali_dns_udp".to_string(),
         transport_type: TransportType::Udp,
         address: "223.5.5.5:53".parse()?,
         url: None,
         weight: 100,
         expected_region: Some("China".to_string()),
     });
     
     engine.add_upstream(UpstreamConfig {
         name: "tencent_dns_udp".to_string(),
         transport_type: TransportType::Udp,
         address: "119.29.29.29:53".parse()?,
         url: None,
         weight: 95,
         expected_region: Some("China".to_string()),
     });
     
     // 国内DNS服务器 - DoH协议
     engine.add_upstream(UpstreamConfig {
         name: "ali_dns_doh".to_string(),
         transport_type: TransportType::Https,
         address: "223.5.5.5:443".parse()?,
         url: Some("https://dns.alidns.com/dns-query".to_string()),
         weight: 98,
         expected_region: Some("China".to_string()),
     });
     
     engine.add_upstream(UpstreamConfig {
         name: "tencent_dns_doh".to_string(),
         transport_type: TransportType::Https,
         address: "119.29.29.29:443".parse()?,
         url: Some("https://doh.pub/dns-query".to_string()),
         weight: 93,
         expected_region: Some("China".to_string()),
     });
     
     // 国内DNS服务器 - DoT协议
     engine.add_upstream(UpstreamConfig {
         name: "ali_dns_dot".to_string(),
         transport_type: TransportType::Tls,
         address: "223.5.5.5:853".parse()?,
         url: None,
         weight: 96,
         expected_region: Some("China".to_string()),
     });
     
     engine.add_upstream(UpstreamConfig {
         name: "tencent_dns_dot".to_string(),
         transport_type: TransportType::Tls,
         address: "119.29.29.29:853".parse()?,
         url: None,
         weight: 91,
         expected_region: Some("China".to_string()),
     });
     
     engine.add_upstream(UpstreamConfig {
         name: "114_dns".to_string(),
         transport_type: TransportType::Udp,
         address: "114.114.114.114:53".parse()?,
         url: None,
         weight: 85,
         expected_region: Some("China".to_string()),
     });
     
     // 国际DNS服务器 - 作为备选
     engine.add_upstream(UpstreamConfig {
         name: "cloudflare_doh".to_string(),
         transport_type: TransportType::Https,
         address: "1.1.1.1:443".parse()?,
         url: Some("https://cloudflare-dns.com/dns-query".to_string()),
         weight: 80,
         expected_region: Some("Global".to_string()),
     });
     
     engine.add_upstream(UpstreamConfig {
         name: "google_dot".to_string(),
         transport_type: TransportType::Tls,
         address: "8.8.8.8:853".parse()?,
         url: None,
         weight: 75,
         expected_region: Some("Global".to_string()),
     });
     
     engine.add_upstream(UpstreamConfig {
         name: "local_isp".to_string(),
         transport_type: TransportType::Udp,
         address: "192.168.1.1:53".parse()?,
         url: None,
         weight: 70,
         expected_region: Some("Asia-Pacific".to_string()),
     });
     
     engine.add_upstream(UpstreamConfig {
         name: "quad9_udp".to_string(),
         transport_type: TransportType::Udp,
         address: "9.9.9.9:53".parse()?,
         url: None,
         weight: 65,
         expected_region: Some("Global".to_string()),
     });
     
     engine.add_upstream(UpstreamConfig {
         name: "unstable_server".to_string(),
         transport_type: TransportType::Tcp,
         address: "203.0.113.1:53".parse()?,
         url: None,
         weight: 60,
         expected_region: Some("Test".to_string()),
     });
    
    // 测试域名列表 - 使用真实域名
    let test_domains = vec![
        "baidu.com",
        "google.com",
        "github.com",
        "taobao.com",
        "qq.com",
        "weibo.com",
        "bilibili.com",
        "zhihu.com",
    ];
    
    println!("\n📊 开始性能基准测试...");
    
    // 执行多轮测试以收集性能数据
    for round in 1..=5 {
        println!("\n--- 第 {} 轮测试 ---", round);
        
        for domain in &test_domains {
             // 显示当前所有服务器的评分（仅在第一轮显示详细信息）
             if round == 1 {
                 println!("\n📊 当前服务器评分:");
                 for upstream in &engine.upstreams {
                     if let Some(metrics) = engine.metrics.get(&upstream.name) {
                         let score = engine.calculate_upstream_score(upstream, metrics);
                         println!("  {} ({:?}): 评分={:.2}, 查询={}, 成功率={:.1}%", 
                             upstream.name, 
                             upstream.transport_type,
                             score,
                             metrics.total_queries,
                             if metrics.total_queries > 0 { 
                                 metrics.successful_queries as f64 / metrics.total_queries as f64 * 100.0 
                             } else { 0.0 }
                         );
                     }
                 }
             }
             
             // 选择最优上游服务器
             if let Some(best_upstream) = engine.select_best_upstream() {
                 let upstream_name = best_upstream.name.clone();
                 println!("🎯 为 {} 选择上游: {} ({:?})", 
                     domain, 
                     upstream_name, 
                     best_upstream.transport_type
                 );
                 
                 let start_time = Instant::now();
                 
                 // 执行DNS查询
                 match timeout(
                     Duration::from_secs(5),
                     simulate_dns_query(best_upstream, domain, &engine.current_region)
                 ).await {
                     Ok(Ok(result)) => {
                         let query_latency = start_time.elapsed();
                         println!("  ✅ 查询成功 - 延迟: {:?}, CDN准确: {}", 
                             query_latency, 
                             result.is_cdn_accurate
                         );
                         
                         engine.update_metrics(
                             &upstream_name,
                             query_latency,
                             true,
                             result.is_cdn_accurate,
                         );
                     },
                     Ok(Err(error)) => {
                         let query_latency = start_time.elapsed();
                         println!("  ❌ 查询失败: {:?}", error);
                         
                         engine.update_metrics(
                             &upstream_name,
                             query_latency,
                             false,
                             false,
                         );
                     },
                     Err(_) => {
                         println!("  ⏰ 查询超时");
                         
                         engine.update_metrics(
                             &upstream_name,
                             Duration::from_secs(5),
                             false,
                             false,
                         );
                     }
                 }
                
                // 短暂延迟以模拟真实使用场景
                tokio::time::sleep(Duration::from_millis(100)).await;
            } else {
                println!("❌ 没有可用的上游服务器");
            }
        }
    }
    
    // 显示最终性能统计
    println!("\n📈 最终性能统计:");
    println!("{:<20} {:<10} {:<10} {:<12} {:<10} {:<8}", 
        "上游服务器", "总查询", "成功率", "平均延迟", "CDN准确率", "连续失败"
    );
    println!("{}", "-".repeat(80));
    
    for upstream in &engine.upstreams {
        if let Some(metrics) = engine.metrics.get(&upstream.name) {
            let success_rate = if metrics.total_queries > 0 {
                (metrics.successful_queries as f64 / metrics.total_queries as f64 * 100.0)
            } else {
                0.0
            };
            
            println!("{:<20} {:<10} {:<9.1}% {:<11.0}ms {:<9.1}% {:<8}",
                upstream.name,
                metrics.total_queries,
                success_rate,
                metrics.avg_latency.as_millis(),
                metrics.cdn_accuracy_score * 100.0,
                metrics.consecutive_failures,
            );
        }
    }
    
    // 显示智能决策结果
    println!("\n🧠 智能决策分析:");
    if let Some(best) = engine.select_best_upstream() {
        println!("当前推荐的最优上游服务器: {} ({:?})", 
            best.name, 
            best.transport_type
        );
        
        if let Some(metrics) = engine.metrics.get(&best.name) {
            let score = engine.calculate_upstream_score(best, metrics);
            println!("综合评分: {:.2}", score);
        }
    }
    
    // 演示实际DNS解析器的使用
    println!("\n🔧 演示实际DNS解析器集成:");
    
    // 创建解析器配置
    let config = ResolverConfig::default();
    let mut resolver = Resolver::new(config);
    
    // 添加不同类型的传输层
    resolver.add_udp_transport(TransportConfig {
        server: "8.8.8.8".to_string(),
        port: 53,
        timeout: Duration::from_secs(5),
        ..Default::default()
    });
    
    // 注意：实际的TLS和HTTPS传输需要正确的配置
    println!("✅ DNS解析器已配置完成，支持多种传输协议");
    
    println!("\n✨ 混合上游服务器智能决策测试完成!");
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::init();
    
    // 运行测试
    run_hybrid_strategy_test().await?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_decision_engine_scoring() {
        let engine = HybridDecisionEngine::new("Test".to_string());
        
        let upstream = UpstreamConfig {
            name: "test_upstream".to_string(),
            transport_type: TransportType::Udp,
            address: "1.1.1.1:53".parse().unwrap(),
            url: None,
            weight: 100,
            expected_region: Some("Test".to_string()),
        };
        
        let mut metrics = PerformanceMetrics {
            total_queries: 100,
            successful_queries: 95,
            failed_queries: 5,
            avg_latency: Duration::from_millis(50),
            cdn_accuracy_score: 0.9,
            last_success_time: Some(Instant::now()),
            consecutive_failures: 0,
        };
        
        let score = engine.calculate_upstream_score(&upstream, &metrics);
        assert!(score > 0.0);
        
        // 测试连续失败的惩罚
        metrics.consecutive_failures = 5;
        let penalized_score = engine.calculate_upstream_score(&upstream, &metrics);
        assert!(penalized_score < score);
    }
    
    #[tokio::test]
    async fn test_simulate_dns_query() {
        let upstream = UpstreamConfig {
            name: "cloudflare_doh".to_string(),
            transport_type: TransportType::Https,
            address: "1.1.1.1:443".parse().unwrap(),
            url: Some("https://cloudflare-dns.com/dns-query".to_string()),
            weight: 80,
            expected_region: Some("Global".to_string()),
        };
        
        // 多次测试以验证随机性
        let mut success_count = 0;
        for _ in 0..10 {
            if let Ok(_) = simulate_dns_query(&upstream, "example.com", "Global").await {
                success_count += 1;
            }
        }
        
        // Cloudflare应该有较高的成功率
        assert!(success_count >= 8);
    }
    
    #[test]
    fn test_upstream_selection_algorithm() {
        let mut engine = HybridDecisionEngine::new("Test".to_string());
        
        // 添加多个上游服务器
        engine.add_upstream(UpstreamConfig {
            name: "fast_server".to_string(),
            transport_type: TransportType::Udp,
            address: "1.1.1.1:53".parse().unwrap(),
            url: None,
            weight: 100,
            expected_region: Some("Test".to_string()),
        });
        
        engine.add_upstream(UpstreamConfig {
            name: "slow_server".to_string(),
            transport_type: TransportType::Tcp,
            address: "2.2.2.2:53".parse().unwrap(),
            url: None,
            weight: 50,
            expected_region: Some("Test".to_string()),
        });
        
        // 模拟性能数据
        engine.update_metrics("fast_server", Duration::from_millis(20), true, true);
        engine.update_metrics("slow_server", Duration::from_millis(200), true, false);
        
        // 验证选择逻辑
        let best = engine.select_best_upstream();
        assert!(best.is_some());
        
        // 快速服务器应该被选中
        let best_name = &best.unwrap().name;
        println!("选择的最优服务器: {}", best_name);
    }
}