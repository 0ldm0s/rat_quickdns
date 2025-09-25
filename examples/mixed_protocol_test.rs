//! 混合协议测试示例
//! 
//! 本示例专门测试 DoH/DoT/UDP 混合协议的功能
//! 验证主库的传输协议是否正常工作

use rat_quickdns::builder::{DnsResolverBuilder, QueryStrategy};
use rat_quickdns::builder::types::{DnsQueryRequest, DnsRecordType};
use rat_quickdns::upstream_handler::UpstreamSpec;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 混合协议测试示例 ===");
    
    
    // 1. 测试纯 DoH 配置
    println!("\n1. 测试纯 DoH 配置");
    let doh_resolver = DnsResolverBuilder::new(
        QueryStrategy::Fifo,
        true,
        "global".to_string(),
    )
    .with_timeout(Duration::from_secs(10))
    .with_retry_count(2)
    .add_upstream(UpstreamSpec::doh("阿里DoH".to_string(), "https://dns.alidns.com/dns-query".to_string()))?
    .add_upstream(UpstreamSpec::doh("腾讯DoH".to_string(), "https://doh.pub/dns-query".to_string()))?
    .build()
    .await?;
    
    println!("DoH解析器已创建");
    
    // 测试 DoH 解析
    let request = DnsQueryRequest {
        query_id: None,
        domain: "example.com".to_string(),
        record_type: DnsRecordType::A,
        enable_edns: true,
        client_address: None,
        timeout_ms: None,
        disable_cache: false,
        enable_dnssec: false,
    };
    
    match doh_resolver.query(request).await {
        Ok(response) => {
            if response.success {
                println!("  DoH查询成功: {} 条记录", response.records.len());
            } else {
                println!("  DoH查询失败: {:?}", response.error);
            }
        },
        Err(e) => {
            println!("  DoH错误: {}", e);
        }
    }
    
    // 2. 测试纯 DoT 配置
    println!("\n2. 测试纯 DoT 配置");
    let dot_resolver = DnsResolverBuilder::new(
        QueryStrategy::Fifo,
        true,
        "global".to_string(),
    )
    .with_timeout(Duration::from_secs(10))
    .with_retry_count(2)
    .add_upstream(UpstreamSpec::dot("阿里DoT".to_string(), "dns.alidns.com:853".to_string()))?
    .add_upstream(UpstreamSpec::dot("腾讯DoT".to_string(), "dot.pub:853".to_string()))?
    .build()
    .await?;
    
    println!("DoT解析器已创建");
    
    let request = DnsQueryRequest {
        query_id: None,
        domain: "example.com".to_string(),
        record_type: DnsRecordType::A,
        enable_edns: true,
        client_address: None,
        timeout_ms: None,
        disable_cache: false,
        enable_dnssec: false,
    };
    
    match dot_resolver.query(request).await {
        Ok(response) => {
            if response.success {
                println!("  DoT查询成功: {} 条记录", response.records.len());
            } else {
                println!("  DoT查询失败: {:?}", response.error);
            }
        },
        Err(e) => {
            println!("  DoT错误: {}", e);
        }
    }
    
    // 3. 测试混合协议配置
    println!("\n3. 测试混合协议配置 (UDP + DoH + DoT)");
    let mixed_resolver = DnsResolverBuilder::new(
        QueryStrategy::Smart,
        true,
        "CN".to_string(),
    )
    .with_timeout(Duration::from_secs(10))
    .with_retry_count(2)
    // UDP 服务器
    .add_upstream(UpstreamSpec::udp("阿里UDP".to_string(), "223.5.5.5".to_string()).with_resolved_ip("223.5.5.5".to_string()))?
    .add_upstream(UpstreamSpec::udp("腾讯UDP".to_string(), "119.29.29.29".to_string()).with_resolved_ip("119.29.29.29".to_string()))?
    // DoH 服务器
    .add_upstream(UpstreamSpec::doh("阿里DoH".to_string(), "https://dns.alidns.com/dns-query".to_string()))?
    .add_upstream(UpstreamSpec::doh("腾讯DoH".to_string(), "https://doh.pub/dns-query".to_string()))?
    // DoT 服务器
    .add_upstream(UpstreamSpec::dot("阿里DoT".to_string(), "dns.alidns.com:853".to_string()))?
    .add_upstream(UpstreamSpec::dot("腾讯DoT".to_string(), "dot.pub:853".to_string()))?
    .build()
    .await?;
    
    println!("混合协议解析器已创建 (UDP + DoH + DoT)");
    
    // 测试多次查询以观察不同协议的使用情况
    let test_domains = vec!["google.com", "github.com", "example.com", "rust-lang.org"];
    
    for domain in &test_domains {
        println!("混合协议查询域名: {}", domain);
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
        
        match mixed_resolver.query(request).await {
            Ok(response) => {
                if response.success {
                    println!("  成功: {} 条记录", response.records.len());
                } else {
                    println!("  失败: {:?}", response.error);
                }
            },
            Err(e) => {
                println!("  错误: {}", e);
            }
        }
        
        // 短暂延迟以观察不同协议的使用
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    
    // 4. 查看统计信息
    println!("\n4. 查看混合协议解析器统计信息");
    let stats = mixed_resolver.get_stats().await;
    println!("总查询数: {}", stats.total_queries);
    println!("成功查询数: {}", stats.successful_queries);
    println!("失败查询数: {}", stats.failed_queries);
    println!("总上游服务器: {}", stats.total_upstreams);
    println!("可用上游服务器: {}", stats.available_upstreams);
    
    println!("\n=== 测试完成 ===");
    println!("如果看到所有协议都能正常工作，说明主库功能正常");
    println!("如果只有UDP工作，说明DoH/DoT特性可能未启用");
    
    Ok(())
}