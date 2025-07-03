//! 智能DNS解析器示例
//! 演示FIFO和智能决策模式的使用

use rat_quickdns::builder::*;
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
        .add_udp_upstream("Google DNS".to_string(), "8.8.8.8:53".parse()?, 100)
        .add_udp_upstream("Cloudflare DNS".to_string(), "1.1.1.1:53".parse()?, 100)
        .add_udp_upstream("阿里DNS".to_string(), "223.5.5.5:53".parse()?, 100)
        .build()
        .await?;
    
    // 测试域名解析
    let test_domains = vec!["google.com", "github.com", "example.com"];
    
    for domain in &test_domains {
        println!("解析域名: {}", domain);
        match fifo_resolver.resolve(domain).await {
            Ok(ips) => {
                println!("  结果: {:?}", ips);
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
        .add_udp_upstream("Google DNS".to_string(), "8.8.8.8:53".parse()?, 120)
        .add_udp_upstream("Cloudflare DNS".to_string(), "1.1.1.1:53".parse()?, 120)
        .add_udp_upstream("阿里DNS".to_string(), "223.5.5.5:53".parse()?, 100)
        .add_udp_upstream("腾讯DNS".to_string(), "119.29.29.29:53".parse()?, 100)
        .build()
        .await?;
    
    // 启动健康检查（每30秒检查一次）
    smart_resolver.start_health_check(Duration::from_secs(30)).await?;
    println!("健康检查已启动，每30秒检查一次服务器状态");
    
    // 测试智能解析
    for domain in &test_domains {
        println!("智能解析域名: {}", domain);
        match smart_resolver.resolve(domain).await {
            Ok(ips) => {
                println!("  结果: {:?}", ips);
            },
            Err(e) => {
                println!("  错误: {}", e);
            }
        }
    }
    
    // 3. 测试批量查询
    println!("\n3. 测试批量查询（并发执行）");
    let batch_domains = vec![
        "google.com".to_string(),
        "github.com".to_string(),
        "stackoverflow.com".to_string(),
        "rust-lang.org".to_string(),
    ];
    
    let batch_results = smart_resolver.batch_query(batch_domains.clone()).await;
    
    for (domain, result) in batch_domains.iter().zip(batch_results.iter()) {
        match result {
            Ok(ips) => println!("批量查询 {}: {:?}", domain, ips),
            Err(e) => println!("批量查询 {} 失败: {}", domain, e),
        }
    }
    
    // 4. 测试不同记录类型
    println!("\n4. 测试不同记录类型");
    let record_types = vec!["A", "AAAA", "MX", "TXT"];
    
    for record_type in &record_types {
        println!("查询 example.com 的 {} 记录:", record_type);
        match smart_resolver.resolve_type("example.com", record_type).await {
            Ok(records) => {
                for record in records {
                    println!("  {}", record);
                }
            },
            Err(e) => {
                println!("  错误: {}", e);
            }
        }
    }
    
    // 5. 测试序列化查询
    println!("\n5. 测试序列化查询（rat_quickmem集成）");
    let query_request = create_dns_query("cloudflare.com", "A");
    
    match encode_dns_query(&query_request) {
        Ok(encoded) => {
            println!("查询请求已编码，大小: {} 字节", encoded.len());
            
            match smart_resolver.process_encoded_query(&encoded).await {
                Ok(response_data) => {
                    println!("响应已编码，大小: {} 字节", response_data.len());
                    
                    // 解码响应（这里简化处理）
                    println!("查询处理完成");
                },
                Err(e) => {
                    println!("处理编码查询失败: {}", e);
                }
            }
        },
        Err(e) => {
            println!("编码查询请求失败: {}", e);
        }
    }
    
    println!("\n=== 示例完成 ===");
    println!("智能DNS解析器支持:");
    println!("- FIFO模式: 多服务器并发查询，最快响应优先");
    println!("- 智能决策模式: 基于性能指标自动选择最优服务器");
    println!("- 健康检查: 定期监控服务器状态和性能");
    println!("- EDNS支持: 自动启用扩展DNS功能");
    println!("- 批量查询: 高效的并发域名解析");
    println!("- 序列化集成: 与rat_quickmem库无缝集成");
    
    Ok(())
}