//! 全面的EDNS Client Subnet测试示例
//! 使用多个DNS服务器和域名来验证地理位置解析差异

use rat_quickdns::{
    resolver::{Resolver, ResolverConfig},
    transport::{TransportConfig},
    types::{RecordType, QClass, Response, RecordData},
    error::Result,
};
use std::net::{Ipv4Addr, IpAddr};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== 全面EDNS Client Subnet地理位置解析测试 ===");
    
    // 测试配置
    let test_configs = vec![
        // 腾讯云DNS
        ("119.29.29.29", "腾讯云DNS"),
        // 阿里云DNS  
        ("223.5.5.5", "阿里云DNS"),
        // 百度DNS
        ("180.76.76.76", "百度DNS"),
        // Cloudflare DNS (支持EDNS)
        ("1.1.1.1", "Cloudflare DNS"),
        // Google DNS (支持EDNS)
        ("8.8.8.8", "Google DNS"),
    ];
    
    // 测试域名 - 选择有CDN且支持地理位置解析的域名
    let test_domains = vec![
        "www.taobao.com",    // 阿里巴巴，有全球CDN
        "www.baidu.com",     // 百度，有地理位置优化
        "www.qq.com",        // 腾讯，有全球CDN
        "cdn.jsdelivr.net",  // 全球CDN服务
        "ajax.googleapis.com", // Google CDN
    ];
    
    // 测试IP地址
    let test_ips = vec![
        ("8.8.8.8", "美国Google"),
        ("1.1.1.1", "美国Cloudflare"), 
        ("114.114.114.114", "中国大陆"),
        ("202.96.128.86", "中国上海"),
        ("168.95.1.1", "中国台湾"),
        ("203.80.96.10", "中国香港"),
        ("139.130.4.5", "日本"),
        ("168.126.63.1", "韩国"),
    ];
    
    for (dns_server, dns_name) in test_configs {
        println!("\n🌐 使用 {} ({})", dns_name, dns_server);
        println!("{}", "=".repeat(60));
        
        let mut resolver = Resolver::new(ResolverConfig::default());
        let config = TransportConfig {
            server: dns_server.to_string(),
            port: 53,
            timeout: Duration::from_secs(5),
            ..Default::default()
        };
        resolver.add_udp_transport(config);
        
        for domain in &test_domains {
            println!("\n📍 测试域名: {}", domain);
            println!("{}", "-".repeat(40));
            
            // 1. 普通查询（无EDNS）
            println!("\n1️⃣  普通查询（无EDNS Client Subnet）:");
            let normal_result = resolver.query(domain, RecordType::A, QClass::IN).await;
            print_result(&normal_result, "普通查询");
            
            // 2. 使用不同地理位置IP测试
            for (ip, location) in &test_ips {
                let client_ip: Ipv4Addr = ip.parse().unwrap();
                
                println!("\n2️⃣  使用{}IP({}):", location, ip);
                let edns_result = resolver.query_with_client_ip(domain, RecordType::A, QClass::IN, Some(IpAddr::V4(client_ip))).await;
                print_result(&edns_result, &format!("{}查询", location));
                
                // 比较结果差异
                if let (Ok(normal), Ok(edns)) = (&normal_result, &edns_result) {
                    compare_results(normal, edns, location);
                }
            }
            
            println!("\n{}", "=".repeat(40));
        }
    }
    
    println!("\n✅ 全面EDNS Client Subnet测试完成");
    println!("\n📊 测试总结:");
    println!("- 如果看到不同IP返回不同的A记录，说明EDNS Client Subnet生效");
    println!("- 如果所有查询返回相同结果，可能是:");
    println!("  1. DNS服务器不支持EDNS Client Subnet");
    println!("  2. 域名没有配置地理位置解析");
    println!("  3. CDN策略相同");
    println!("- 建议重点关注Cloudflare和Google DNS的结果差异");
    
    Ok(())
}



fn print_result(result: &Result<Response>, query_type: &str) {
    match result {
        Ok(response) => {
            if response.answers.is_empty() {
                println!("   ❌ {} - 无解析结果", query_type);
            } else {
                println!("   ✅ {} - 获得 {} 条记录:", query_type, response.answers.len());
                for (i, record) in response.answers.iter().enumerate() {
                    println!("      {}. {}: {:?}", i + 1, record.name, record.data);
                }
            }
        }
        Err(e) => {
            println!("   ❌ {} - 查询失败: {}", query_type, e);
        }
    }
}

fn compare_results(
    normal: &Response,
    edns: &Response,
    location: &str,
) {
    let normal_ips = extract_a_records(normal);
    let edns_ips = extract_a_records(edns);
    
    if normal_ips != edns_ips {
        println!("   🎯 检测到地理位置差异! {}IP返回了不同的结果", location);
        println!("      普通查询: {:?}", normal_ips);
        println!("      {}查询: {:?}", location, edns_ips);
    } else if !normal_ips.is_empty() {
        println!("   ℹ️  {}IP返回相同结果: {:?}", location, normal_ips);
    }
}

fn extract_a_records(response: &Response) -> Vec<std::net::Ipv4Addr> {
    response
        .answers
        .iter()
        .filter_map(|record| {
            if let RecordData::A(ip) = record.data {
                Some(ip)
            } else {
                None
            }
        })
        .collect()
}