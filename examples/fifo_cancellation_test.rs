//! FIFO模式取消测试
//! 验证最快优先策略的早期取消机制

use rat_quickdns::builder::*;
use std::time::Duration;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== FIFO模式取消测试 ===");
    println!("测试最快优先策略的早期取消机制");
    
    // 创建一个FIFO模式的解析器，添加多个服务器
    let resolver = DnsResolverBuilder::new()
        .query_strategy(QueryStrategy::Fifo)
        .enable_edns(true)
        .add_udp_upstream("Google DNS".to_string(), "8.8.8.8:53".parse()?, 100)
        .add_udp_upstream("Cloudflare DNS".to_string(), "1.1.1.1:53".parse()?, 100)
        .add_udp_upstream("阿里DNS".to_string(), "223.5.5.5:53".parse()?, 100)
        .add_udp_upstream("腾讯DNS".to_string(), "119.29.29.29:53".parse()?, 100)
        .build()
        .await?;
    
    // 测试域名列表
    let test_domains = vec![
        "google.com",
        "github.com",
        "example.com",
        "microsoft.com",
        "apple.com",
    ];
    
    // 测试单个域名解析
    println!("\n1. 测试单个域名解析（应该看到取消日志）");
    for domain in &test_domains {
        println!("\n解析域名: {}", domain);
        match resolver.resolve(domain).await {
            Ok(ips) => {
                println!("  结果: {:?}", ips);
            },
            Err(e) => {
                println!("  错误: {}", e);
            }
        }
        
        // 给日志输出一些时间
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    
    // 测试批量域名解析
    println!("\n2. 测试批量域名解析");
    let batch_domains = test_domains.iter().map(|d| d.to_string()).collect::<Vec<_>>();
    
    let batch_results = resolver.batch_query(batch_domains.clone()).await;
    
    for (domain, result) in batch_domains.iter().zip(batch_results.iter()) {
        match result {
            Ok(ips) => println!("批量查询 {}: {:?}", domain, ips),
            Err(e) => println!("批量查询 {} 失败: {}", domain, e),
        }
    }
    
    // 测试性能对比
    println!("\n3. 性能测试：连续解析100次相同域名");
    let test_domain = "example.com";
    let iterations = 100;
    
    let start_time = std::time::Instant::now();
    
    for i in 0..iterations {
        let _ = resolver.resolve(test_domain).await?;
        if i % 10 == 0 {
            print!(".");
        }
    }
    
    let elapsed = start_time.elapsed();
    println!("\n完成 {} 次查询，总耗时: {:?}，平均每次: {:?}", 
             iterations, elapsed, elapsed / iterations as u32);
    
    println!("\n=== 测试完成 ===");
    println!("FIFO模式现在支持早期取消机制，当最快的服务器返回结果后，其他服务器的查询会被自动取消。");
    println!("这可以减少资源浪费，提高整体性能。");
    
    Ok(())
}