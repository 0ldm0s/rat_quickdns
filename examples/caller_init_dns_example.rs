//! 调用者初始化DNS日志系统示例
//!
//! 这个示例展示了如何作为调用者来正确初始化日志系统
//! 然后使用rat_quickdns进行DNS查询

use rat_quickdns::{
    builder::{DnsResolverBuilder, QueryStrategy},
    logger::init_dns_logger,
    dns_query, dns_response, dns_error, dns_timeout,
};
use rat_logger::{LoggerBuilder, LevelFilter, handler::term::TermConfig};
use std::time::Duration;
use tokio;

/// 初始化日志系统的推荐方式
fn init_logging_system() -> Result<(), Box<dyn std::error::Error>> {
    // 创建DNS专用格式化器的配置
    let term_config = TermConfig {
        enable_color: true,
        format: None, // 使用默认格式
        color: None,  // 使用默认颜色
    };

    // 调用者负责初始化日志系统
    LoggerBuilder::new()
        .with_level(LevelFilter::Info)  // 设置日志级别
        .add_terminal_with_config(term_config)
        .init_global_logger()?;

    // 然后初始化DNS日志格式（可选）
    init_dns_logger(LevelFilter::Info)?;

    println!("✅ 日志系统初始化成功");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // === 调用者负责初始化日志系统 ===
    println!("=== 调用者初始化DNS日志系统示例 ===\n");

    // 1. 首先初始化日志系统（调用者责任）
    init_logging_system()?;

    println!("✅ 所有系统初始化完成\n");

    // 2. 配置DNS解析器
    println!("配置DNS解析器...");

    let resolver = DnsResolverBuilder::new(
        QueryStrategy::Smart,
        true,  // 启用EDNS
        "global".to_string(),
    )
        .add_udp_upstream("阿里DNS", "223.5.5.5")
        .add_udp_upstream("腾讯DNS", "119.29.29.29")
        .add_udp_upstream("Google DNS", "8.8.8.8")
        .add_udp_upstream("Cloudflare DNS", "1.1.1.1")
        .with_timeout(Duration::from_secs(5))
        .with_retry_count(3)
        .build()
        .await?;

    println!("✅ DNS解析器配置完成");

    // 3. DNS查询示例
    println!("\n=== DNS查询示例 ===");

    // 查询示例
    let test_domains = vec![
        "google.com",
        "github.com",
        "baidu.com",
        "qq.com",
    ];

    for domain in test_domains {
        println!("\n查询域名: {}", domain);

        // 使用DNS专用日志宏
        dns_query!(domain, "A");

        let request = rat_quickdns::builder::types::DnsQueryRequest {
            query_id: Some(format!("query-{}", domain)),
            domain: domain.to_string(),
            record_type: rat_quickdns::builder::types::DnsRecordType::A,
            enable_edns: true,
            client_address: None,
            timeout_ms: None,
            disable_cache: false,
            enable_dnssec: false,
        };

        match resolver.query(request).await {
            Ok(response) => {
                dns_response!(domain, response.records.len(), response.duration_ms);
                println!("  查询成功:");
                for record in &response.records {
                    println!("    - {:?}", record);
                }
            },
            Err(e) => {
                dns_error!("查询失败: {} - {}", domain, e);
            }
        }
    }

    // 4. 缓存演示
    println!("\n=== 缓存演示 ===");
    let domain = "cached.example.com";

    dns_query!(domain, "A");
    dns_timeout!(domain, 5000);
    dns_error!("模拟缓存未命中: {}", domain);

    // 5. 上游服务器演示
    println!("\n=== 上游服务器演示 ===");
    println!("使用不同的上游服务器进行查询:");

    for (name, server) in &[
        ("阿里DNS", "223.5.5.5"),
        ("腾讯DNS", "119.29.29.29"),
        ("Google DNS", "8.8.8.8"),
        ("Cloudflare DNS", "1.1.1.1"),
    ] {
        println!("  通过 {} ({}) 查询 example.com", name, server);
    }

    println!("\n=== 示例完成 ===");
    println!("这个示例展示了：");
    println!("1. 调用者如何自定义和初始化日志系统");
    println!("2. rat_quickdns库不再自动初始化日志");
    println!("3. 日志系统完全由调用者控制");
    println!("4. DNS专用日志宏的使用方法");

    Ok(())
}