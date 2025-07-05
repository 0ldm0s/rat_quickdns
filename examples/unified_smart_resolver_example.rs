//! 统一智能DNS解析器示例
//! 
//! 本示例展示如何使用 SmartDnsResolver 作为统一入口进行DNS解析
//! 包括不同查询策略的使用方法

use rat_quickdns::builder::{DnsResolverBuilder, QueryStrategy};
use rat_quickdns::builder::types::{DnsQueryRequest, DnsRecordType};
use rat_quickmem::QuickMemConfig;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 统一智能DNS解析器示例 ===\n");
    
    // 创建 QuickMem 配置
    let quickmem_config = QuickMemConfig {
        max_data_size: 64 * 1024 * 1024, // 64MB
        max_batch_count: 10000,
        pool_initial_capacity: 1024,
        pool_max_capacity: 10240,
        enable_parallel: true,
    };
    
    // 1. 使用 Smart 策略（推荐）
    println!("1. 创建智能策略解析器");
    let smart_resolver = DnsResolverBuilder::new(
        QueryStrategy::Smart,
        true,  // 启用 EDNS
        "global".to_string(),
        quickmem_config.clone(),
    )
    .with_timeout(Duration::from_secs(5))
    .with_retry_count(2)
    .with_cache(true)
    .with_upstream_monitoring(true)
    .add_udp_upstream("阿里DNS", "223.5.5.5")
    .add_udp_upstream("腾讯DNS", "119.29.29.29")
    .add_doh_upstream("Cloudflare DoH", "https://cloudflare-dns.com/dns-query")
    .add_dot_upstream("阿里DoT", "223.5.5.5")
    .build()
    .await?;
    
    // 执行查询
    let request = DnsQueryRequest {
        domain: "www.example.com".to_string(),
        record_type: DnsRecordType::A,
        query_id: Some("smart-test-1".to_string()),
    };
    
    let response = smart_resolver.query(request).await?;
    println!("智能策略查询结果: {:?}\n", response);
    
    // 2. 使用 RoundRobin 策略
    println!("2. 创建轮询策略解析器");
    let round_robin_resolver = DnsResolverBuilder::new(
        QueryStrategy::RoundRobin,
        true,
        "global".to_string(),
        quickmem_config.clone(),
    )
    .optimize_for_round_robin()  // 应用轮询优化
    .add_udp_upstream("Google DNS", "8.8.8.8")
    .add_udp_upstream("Cloudflare DNS", "1.1.1.1")
    .add_udp_upstream("Quad9 DNS", "9.9.9.9")
    .build()
    .await?;
    
    let request = DnsQueryRequest {
        domain: "www.google.com".to_string(),
        record_type: DnsRecordType::A,
        query_id: Some("rr-test-1".to_string()),
    };
    
    let response = round_robin_resolver.query(request).await?;
    println!("轮询策略查询结果: {:?}\n", response);
    
    // 3. 使用 FIFO 策略
    println!("3. 创建FIFO策略解析器");
    let fifo_resolver = DnsResolverBuilder::new(
        QueryStrategy::Fifo,
        true,
        "global".to_string(),
        quickmem_config,
    )
    .with_timeout(Duration::from_secs(3))
    .add_udp_upstream("首选DNS", "114.114.114.114")
    .add_udp_upstream("备用DNS", "8.8.8.8")
    .build()
    .await?;
    
    let request = DnsQueryRequest {
        domain: "www.baidu.com".to_string(),
        record_type: DnsRecordType::A,
        query_id: Some("fifo-test-1".to_string()),
    };
    
    let response = fifo_resolver.query(request).await?;
    println!("FIFO策略查询结果: {:?}\n", response);
    
    // 4. 使用便捷方法创建包含公共DNS的解析器
    println!("4. 使用便捷方法创建解析器");
    let public_dns_resolver = DnsResolverBuilder::new(
        QueryStrategy::Smart,
        true,
        "global".to_string(),
        QuickMemConfig::default(),
    )
    .with_public_dns()?  // 自动添加多个公共DNS服务器
    .with_verbose_logging()  // 启用详细日志
    .build()
    .await?;
    
    let request = DnsQueryRequest {
        domain: "www.github.com".to_string(),
        record_type: DnsRecordType::A,
        query_id: Some("public-dns-test-1".to_string()),
    };
    
    let response = public_dns_resolver.query(request).await?;
    println!("公共DNS解析器查询结果: {:?}\n", response);
    
    println!("=== 示例完成 ===");
    Ok(())
}