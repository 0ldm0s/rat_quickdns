//! 智能DNS解析器示例
//! 演示FIFO和智能决策模式的使用

use rat_quickdns::builder::*;
use rat_quickdns::builder::types::{DnsQueryRequest, DnsRecordType};
use std::time::Duration;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 智能DNS解析器示例 ===");
    
    // 1. 测试FIFO模式（默认）
    println!("\n1. 测试FIFO模式（多服务器并发查询）");
    let fifo_resolver = DnsResolverBuilder::new()
        .query_strategy(QueryStrategy::Fifo)
        .enable_edns(true)
        .add_udp_upstream("Google DNS", "8.8.8.8")
        .add_udp_upstream("Cloudflare DNS", "1.1.1.1")
        .add_udp_upstream("阿里DNS", "223.5.5.5")
        .build()
        .await?;
    
    // 测试域名解析
    let test_domains = vec!["google.com", "github.com", "example.com"];
    
    for domain in &test_domains {
        println!("解析域名: {}", domain);
        let request = DnsQueryRequest {
            query_id: None,
            domain: domain.to_string(),
            record_type: DnsRecordType::A,
            enable_edns: true,
            client_subnet: None,
            timeout_ms: None,
            disable_cache: false,
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
    let smart_resolver = DnsResolverBuilder::new()
        .query_strategy(QueryStrategy::Smart)
        .enable_edns(true)
        .region("CN".to_string())
        .add_udp_upstream("Google DNS", "8.8.8.8")
        .add_udp_upstream("Cloudflare DNS", "1.1.1.1")
        .add_udp_upstream("阿里DNS", "223.5.5.5")
        .add_udp_upstream("腾讯DNS", "119.29.29.29")
        .build()
        .await?;
    
    // 注意：健康检查功能在构建时自动启动
    println!("智能解析器已启动，包含自动健康检查功能");
    
    // 测试智能解析
    for domain in &test_domains {
        println!("智能解析域名: {}", domain);
        let request = DnsQueryRequest {
             query_id: None,
             domain: domain.to_string(),
             record_type: DnsRecordType::A,
             enable_edns: true,
             client_subnet: None,
             timeout_ms: None,
             disable_cache: false,
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
    
    // 3. 测试多个域名查询
    println!("\n3. 测试多个域名查询");
    let batch_domains = vec!["google.com", "github.com", "stackoverflow.com", "rust-lang.org"];
    
    for domain in &batch_domains {
        let request = DnsQueryRequest {
             query_id: None,
             domain: domain.to_string(),
             record_type: DnsRecordType::A,
             enable_edns: true,
             client_subnet: None,
             timeout_ms: None,
             disable_cache: false,
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
    
    // 4. 测试不同记录类型
    println!("\n4. 测试不同记录类型");
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
             client_subnet: None,
             timeout_ms: None,
             disable_cache: false,
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
    
    // 5. 测试解析器统计信息
    println!("\n5. 查看解析器统计信息");
    let stats = smart_resolver.get_stats().await;
    println!("总查询数: {}", stats.total_queries);
    println!("成功查询数: {}", stats.successful_queries);
    println!("失败查询数: {}", stats.failed_queries);
    println!("总上游服务器: {}", stats.total_upstreams);
    println!("健康上游服务器: {}", stats.healthy_upstreams);
    
    println!("\n=== 示例完成 ===");
    println!("智能DNS解析器支持:");
    println!("- FIFO模式: 多服务器并发查询，最快响应优先");
    println!("- 智能决策模式: 基于性能指标自动选择最优服务器");
    println!("- 健康检查: 定期监控服务器状态和性能");
    println!("- EDNS支持: 自动启用扩展DNS功能");
    println!("- 多域名查询: 支持多个域名的顺序查询");
    println!("- 统计信息: 提供详细的查询统计数据");
    
    Ok(())
}