//! DNS解析器日志配置示例
//! 
//! 本示例展示如何使用新的日志系统配置DNS解析器，
//! 包括不同的日志级别和格式设置。

use rat_quickdns::{
    DnsResolverBuilder, 
    types::{RecordType, QClass},
    Result,
};
use zerg_creep::logger::LevelFilter;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== DNS解析器日志配置示例 ===");
    
    // 示例1: 使用详细日志（Debug级别）
    println!("\n1. 创建详细日志解析器...");
    let verbose_resolver = DnsResolverBuilder::new()
        .add_udp_upstream("阿里DNS", "223.5.5.5")
        .add_udp_upstream("腾讯DNS", "119.29.29.29")
        .with_verbose_logging()  // 启用详细日志
        .with_timeout(Duration::from_secs(3))
        .build()
        .await?;
    
    println!("执行DNS查询（详细日志模式）...");
    match verbose_resolver.query("www.baidu.com", RecordType::A, QClass::IN).await {
        Ok(response) => {
            println!("✓ 查询成功，获得 {} 条记录", response.answers.len());
            for answer in &response.answers {
                println!("  - {}: {:?}", answer.name, answer.data);
            }
        }
        Err(e) => println!("✗ 查询失败: {}", e),
    }
    
    println!("\n" + "=".repeat(50).as_str());
    
    // 示例2: 使用静默日志（Error级别）
    println!("\n2. 创建静默日志解析器...");
    let quiet_resolver = DnsResolverBuilder::new()
        .add_udp_upstream("Google DNS", "8.8.8.8")
        .add_udp_upstream("Cloudflare DNS", "1.1.1.1")
        .with_quiet_logging()  // 启用静默日志
        .with_timeout(Duration::from_secs(3))
        .build()
        .await?;
    
    println!("执行DNS查询（静默日志模式）...");
    match quiet_resolver.query("www.google.com", RecordType::A, QClass::IN).await {
        Ok(response) => {
            println!("✓ 查询成功，获得 {} 条记录", response.answers.len());
            for answer in &response.answers {
                println!("  - {}: {:?}", answer.name, answer.data);
            }
        }
        Err(e) => println!("✗ 查询失败: {}", e),
    }
    
    println!("\n" + "=".repeat(50).as_str());
    
    // 示例3: 自定义日志配置
    println!("\n3. 创建自定义日志配置解析器...");
    let custom_resolver = DnsResolverBuilder::new()
        .add_udp_upstream("114DNS", "114.114.114.114")
        .with_log_level(LevelFilter::Info)  // 设置Info级别
        .with_dns_log_format(true)       // 启用DNS专用格式
        .with_timeout(Duration::from_secs(5))
        .with_retry_count(3)
        .build()
        .await?;
    
    println!("执行DNS查询（自定义日志配置）...");
    match custom_resolver.query("www.github.com", RecordType::A, QClass::IN).await {
        Ok(response) => {
            println!("✓ 查询成功，获得 {} 条记录", response.answers.len());
            for answer in &response.answers {
                println!("  - {}: {:?}", answer.name, answer.data);
            }
        }
        Err(e) => println!("✗ 查询失败: {}", e),
    }
    
    println!("\n" + "=".repeat(50).as_str());
    
    // 示例4: 禁用DNS专用日志格式
    println!("\n4. 创建标准日志格式解析器...");
    let standard_resolver = DnsResolverBuilder::new()
        .add_udp_upstream("OpenDNS", "208.67.222.222")
        .with_log_level(LevelFilter::Warn)   // 设置Warn级别
        .with_dns_log_format(false)       // 禁用DNS专用格式，使用标准zerg_creep格式
        .with_timeout(Duration::from_secs(3))
        .build()
        .await?;
    
    println!("执行DNS查询（标准日志格式）...");
    match standard_resolver.query("www.stackoverflow.com", RecordType::A, QClass::IN).await {
        Ok(response) => {
            println!("✓ 查询成功，获得 {} 条记录", response.answers.len());
            for answer in &response.answers {
                println!("  - {}: {:?}", answer.name, answer.data);
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