//! DNS解析器日志配置示例
//! 
//! 本示例展示如何使用新的日志系统配置DNS解析器，
//! 包括不同的日志级别和格式设置。

use rat_quickdns::builder::{DnsResolverBuilder, QueryStrategy};
use rat_quickdns::builder::types::{DnsQueryRequest, DnsRecordType};
use zerg_creep::logger::LevelFilter;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== DNS解析器日志配置示例 ===");
    
    
    // 示例1: 使用详细日志（Debug级别）
    println!("\n1. 创建详细日志解析器...");
    let verbose_resolver = DnsResolverBuilder::new(
        QueryStrategy::Smart,
        true,  // 启用 EDNS
        "global".to_string(),
    )
    .add_udp_upstream("阿里DNS", "223.5.5.5")
    .add_udp_upstream("腾讯DNS", "119.29.29.29")
    .with_verbose_logging()  // 启用详细日志
    .with_timeout(Duration::from_secs(3))
    .build()
    .await?;
    
    println!("执行DNS查询（详细日志模式）...");
    let request = DnsQueryRequest {
        domain: "www.baidu.com".to_string(),
        record_type: DnsRecordType::A,
        query_id: Some("verbose-test".to_string()),
        enable_edns: true,
        client_address: None,
        timeout_ms: None,
        disable_cache: false,
        enable_dnssec: false,
    };
    
    match verbose_resolver.query(request).await {
        Ok(response) => {
            println!("✓ 查询成功，获得 {} 条记录", response.records.len());
            for record in &response.records {
                println!("  - {}: {:?}", record.name, record.value);
            }
        }
        Err(e) => println!("✗ 查询失败: {}", e),
    }
    
    println!("\n{}", "=".repeat(50));
    
    // 示例2: 使用静默日志（Error级别）
    println!("\n2. 创建静默日志解析器...");
    let quiet_resolver = DnsResolverBuilder::new(
        QueryStrategy::Smart,
        true,  // 启用 EDNS
        "global".to_string(),
    )
        .add_udp_upstream("Google DNS", "8.8.8.8")
        .add_udp_upstream("Cloudflare DNS", "1.1.1.1")
        .with_quiet_logging()  // 启用静默日志
        .with_timeout(Duration::from_secs(3))
        .build()
        .await?;
    
    println!("执行DNS查询（静默日志模式）...");
    let request2 = DnsQueryRequest {
        domain: "www.google.com".to_string(),
        record_type: DnsRecordType::A,
        query_id: Some("quiet-test".to_string()),
        enable_edns: true,
        client_address: None,
        timeout_ms: None,
        disable_cache: false,
        enable_dnssec: false,
    };
    match quiet_resolver.query(request2).await {
        Ok(response) => {
            println!("✓ 查询成功，获得 {} 条记录", response.records.len());
            for record in &response.records {
                println!("  - {}: {:?}", record.name, record.value);
            }
        }
        Err(e) => println!("✗ 查询失败: {}", e),
    }
    
    println!("\n{}", "=".repeat(50));
    
    // 示例3: 自定义日志配置
    println!("\n3. 创建自定义日志配置解析器...");
    let custom_resolver = DnsResolverBuilder::new(
        QueryStrategy::Smart,
        true,  // 启用 EDNS
        "global".to_string(),
    )
        .add_udp_upstream("114DNS", "114.114.114.114")
        .with_log_level(LevelFilter::Info)  // 设置Info级别
        .with_dns_log_format(true)       // 启用DNS专用格式
        .with_timeout(Duration::from_secs(5))
        .with_retry_count(3)
        .build()
        .await?;
    
    println!("执行DNS查询（自定义日志配置）...");
    let request3 = DnsQueryRequest {
        domain: "www.github.com".to_string(),
        record_type: DnsRecordType::A,
        query_id: Some("custom-test".to_string()),
        enable_edns: true,
        client_address: None,
        timeout_ms: None,
        disable_cache: false,
        enable_dnssec: false,
    };
    match custom_resolver.query(request3).await {
        Ok(response) => {
            println!("✓ 查询成功，获得 {} 条记录", response.records.len());
            for record in &response.records {
                println!("  - {}: {:?}", record.name, record.value);
            }
        }
        Err(e) => println!("✗ 查询失败: {}", e),
    }
    
    println!("\n{}", "=".repeat(50));
    
    // 示例4: 禁用DNS专用日志格式
    println!("\n4. 创建标准日志格式解析器...");
    let standard_resolver = DnsResolverBuilder::new(
        QueryStrategy::Smart,
        true,  // 启用 EDNS
        "global".to_string(),
    )
        .add_udp_upstream("OpenDNS", "208.67.222.222")
        .with_log_level(LevelFilter::Warn)   // 设置Warn级别
        .with_dns_log_format(false)       // 禁用DNS专用格式，使用标准zerg_creep格式
        .with_timeout(Duration::from_secs(3))
        .build()
        .await?;
    
    println!("执行DNS查询（标准日志格式）...");
    let request4 = DnsQueryRequest {
        domain: "www.stackoverflow.com".to_string(),
        record_type: DnsRecordType::A,
        query_id: Some("standard-test".to_string()),
        enable_edns: true,
        client_address: None,
        timeout_ms: None,
        disable_cache: false,
        enable_dnssec: false,
    };
    match standard_resolver.query(request4).await {
        Ok(response) => {
            println!("✓ 查询成功，获得 {} 条记录", response.records.len());
            for record in &response.records {
                println!("  - {}: {:?}", record.name, record.value);
            }
        }
        Err(e) => println!("✗ 查询失败: {}", e),
    }
    
    println!("\n=== 示例完成 ===");
    println!("\n日志配置说明:");
    println!("- verbose_logging(): Debug级别 + DNS专用格式");
    println!("- quiet_logging(): Error级别 + DNS专用格式");
    println!("- with_log_level(): 自定义日志级别");
    println!("- with_dns_log_format(true): 启用DNS专用日志格式");
    println!("- with_dns_log_format(false): 使用标准zerg_creep日志格式");
    
    Ok(())
}