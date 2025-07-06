//! DnsResolverBuilder 统一架构示例
//! 
//! 本示例展示如何使用 DnsResolverBuilder 统一架构进行DNS解析
//! 包括FIFO、智能决策和轮询三种查询策略的使用方法
//! 
//! 主要特性:
//! - 统一的构建接口，支持所有DNS协议(UDP/DoH/DoT)
//! - 预解析IP地址，避免DNS解析延迟
//! - 多种查询策略：FIFO、Smart、RoundRobin
//! - 自动健康检查和性能监控
//! - 详细的统计信息和日志输出

use rat_quickdns::builder::{DnsResolverBuilder, QueryStrategy};
use rat_quickdns::builder::types::{DnsQueryRequest, DnsRecordType};
use rat_quickdns::upstream_handler::UpstreamSpec;
use rat_quickmem::QuickMemConfig;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== DnsResolverBuilder 统一架构示例 ===");
    
    // 创建 QuickMem 配置
    let quickmem_config = QuickMemConfig {
        max_data_size: 64 * 1024 * 1024, // 64MB
        max_batch_count: 10000,
        pool_initial_capacity: 1024,
        pool_max_capacity: 10240,
        enable_parallel: true,
    };
    
    // 1. 测试FIFO模式（默认）
    println!("\n1. 测试FIFO模式（多服务器并发查询）");
    let fifo_resolver = DnsResolverBuilder::new(
        QueryStrategy::Fifo,
        true,
        "global".to_string(),
        quickmem_config.clone(),
    )
    .with_timeout(Duration::from_secs(5))
    .with_retry_count(2)
    .with_debug_logger_init()  // 启用调试级别日志
    .add_upstream(UpstreamSpec::udp("阿里DNS".to_string(), "223.5.5.5".to_string()))?
    .add_upstream(UpstreamSpec::udp("腾讯DNS".to_string(), "119.29.29.29".to_string()))?
    .add_upstream(UpstreamSpec::udp("114DNS".to_string(), "114.114.114.114".to_string()))?
    .build()
    .await?;
    
    println!("FIFO解析器已创建，使用标准UDP上游服务器");
    
    // 测试域名解析
    let test_domains = vec!["google.com", "github.com", "example.com"];
    
    for domain in &test_domains {
        println!("解析域名: {}", domain);
        let request = DnsQueryRequest {
            query_id: None,
            domain: domain.to_string(),
            record_type: DnsRecordType::A,
            enable_edns: true,
            client_address: None,
            timeout_ms: None,
            disable_cache: false,
            enable_dnssec: false,
        };
        match fifo_resolver.query(request).await {
            Ok(response) => {
                if response.success {
                    println!("  结果: {} 条记录", response.records.len());
                } else {
                    println!("  查询失败: {:?}", response.error);
                }
            },
            Err(e) => {
                println!("  错误: {}", e);
            }
        }
    }
    
    // 2. 测试智能决策模式
    println!("\n2. 测试智能决策模式（自动选择最优服务器）");
    let smart_resolver = DnsResolverBuilder::new(
        QueryStrategy::Smart,
        true,
        "CN".to_string(),
        quickmem_config.clone(),
    )
    .with_timeout(Duration::from_secs(5))
    .with_retry_count(2)
    .with_debug_logger_init()  // 启用调试级别日志
    .add_upstream(UpstreamSpec::udp("阿里DNS".to_string(), "223.5.5.5".to_string()).with_resolved_ip("223.5.5.5".to_string()))?
    .add_upstream(UpstreamSpec::udp("腾讯DNS".to_string(), "119.29.29.29".to_string()).with_resolved_ip("119.29.29.29".to_string()))?
    .add_upstream(UpstreamSpec::udp("114DNS".to_string(), "114.114.114.114".to_string()).with_resolved_ip("114.114.114.114".to_string()))?
    .add_upstream(UpstreamSpec::udp("百度DNS".to_string(), "180.76.76.76".to_string()).with_resolved_ip("180.76.76.76".to_string()))?
    .build()
    .await?;
    
    // 注意：健康检查功能在构建时自动启动
    println!("智能解析器已启动，包含自动健康检查功能");
    println!("所有上游服务器已配置预解析IP地址，避免DNS解析延迟");
    
    // 测试智能解析
    for domain in &test_domains {
        println!("智能解析域名: {}", domain);
        let request = DnsQueryRequest {
                query_id: None,
                domain: domain.to_string(),
                record_type: DnsRecordType::A,
                enable_edns: true,
                client_address: None,
                timeout_ms: None,
                disable_cache: false,
                enable_dnssec: false,
            };
        match smart_resolver.query(request).await {
            Ok(response) => {
                if response.success {
                    println!("  结果: {} 条记录", response.records.len());
                } else {
                    println!("  查询失败: {:?}", response.error);
                }
            },
            Err(e) => {
                println!("  错误: {}", e);
            }
        }
    }
    
    // 3. 测试轮询模式
    println!("\n3. 测试轮询模式（负载均衡）");
    let round_robin_resolver = DnsResolverBuilder::new(
        QueryStrategy::RoundRobin,
        true,
        "global".to_string(),
        quickmem_config.clone(),
    )
    .with_timeout(Duration::from_secs(5))
    .with_retry_count(2)
    .with_debug_logger_init()  // 启用调试级别日志
    .add_upstream(UpstreamSpec::udp("Google DNS".to_string(), "8.8.8.8".to_string()).with_resolved_ip("8.8.8.8".to_string()))?
    .add_upstream(UpstreamSpec::udp("Cloudflare DNS".to_string(), "1.1.1.1".to_string()).with_resolved_ip("1.1.1.1".to_string()))?
    .add_upstream(UpstreamSpec::udp("阿里DNS".to_string(), "223.5.5.5".to_string()).with_resolved_ip("223.5.5.5".to_string()))?
    .add_upstream(UpstreamSpec::udp("腾讯DNS".to_string(), "119.29.29.29".to_string()).with_resolved_ip("119.29.29.29".to_string()))?
    .build()
    .await?;

    println!("轮询解析器已创建，启用轮询优化和预解析IP地址");

    // 4. 测试多个域名查询
    println!("\n4. 测试多个域名查询");
    let batch_domains = vec!["google.com", "github.com", "stackoverflow.com", "rust-lang.org"];
    
    for domain in &batch_domains {
        let request = DnsQueryRequest {
                 query_id: None,
                 domain: domain.to_string(),
                 record_type: DnsRecordType::A,
                 enable_edns: true,
                 client_address: None,
                 timeout_ms: None,
                 disable_cache: false,
                 enable_dnssec: false,
             };
        match smart_resolver.query(request).await {
            Ok(response) => {
                if response.success {
                    println!("查询 {}: 成功，ID: {}", domain, response.query_id);
                } else {
                    println!("查询 {}: 失败，错误: {:?}", domain, response.error);
                }
            },
            Err(e) => println!("查询 {} 失败: {}", domain, e),
        }
    }
    
    // 5. 测试不同记录类型
    println!("\n5. 测试不同记录类型");
    let record_types = vec![
        ("A", DnsRecordType::A),
        ("AAAA", DnsRecordType::AAAA),
        ("MX", DnsRecordType::MX),
        ("TXT", DnsRecordType::TXT),
    ];
    
    for (name, record_type) in &record_types {
        println!("查询 example.com 的 {} 记录:", name);
        let request = DnsQueryRequest {
                 query_id: None,
                 domain: "example.com".to_string(),
                 record_type: *record_type,
                 enable_edns: true,
                 client_address: None,
                 timeout_ms: None,
                 disable_cache: false,
                 enable_dnssec: false,
             };
        match smart_resolver.query(request).await {
            Ok(response) => {
                if response.success {
                    println!("  成功，响应ID: {}, 记录数: {}", response.query_id, response.records.len());
                } else {
                    println!("  查询失败: {:?}", response.error);
                }
            },
            Err(e) => {
                println!("  错误: {}", e);
            }
        }
    }
    
    // 6. 测试解析器统计信息
    println!("\n6. 查看解析器统计信息");
    let stats = smart_resolver.get_stats().await;
    println!("总查询数: {}", stats.total_queries);
    println!("成功查询数: {}", stats.successful_queries);
    println!("失败查询数: {}", stats.failed_queries);
    println!("总上游服务器: {}", stats.total_upstreams);
    println!("可用上游服务器: {}", stats.available_upstreams);
    
    println!("\n=== 示例完成 ===");
    println!("DnsResolverBuilder 统一架构支持:");
    println!("- FIFO模式: 多服务器并发查询，最快响应优先");
    println!("- 智能决策模式: 基于性能指标自动选择最优服务器");
    println!("- 轮询模式: 负载均衡，轮流使用各个上游服务器");
    println!("- 预解析IP: 避免DNS解析延迟，提升连接速度");
    println!("- 健康检查: 定期监控服务器状态和性能");
    println!("- EDNS支持: 自动启用扩展DNS功能");
    println!("- 多域名查询: 支持多个域名的顺序查询");
    println!("- 统计信息: 提供详细的查询统计数据");
    println!("- 统一接口: 所有协议(UDP/DoH/DoT)使用相同的构建方式");
    
    Ok(())
}