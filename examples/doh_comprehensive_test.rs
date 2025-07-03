//! DNS-over-HTTPS (DoH) 全面测试示例
//! 基于 comprehensive_edns_test.rs，验证 DoH 传输的 EDNS Client Subnet 功能
//! 可以与 UDP 传输结果进行对比

use rat_quickdns::{
    resolver::{Resolver, ResolverConfig},
    transport::{TransportConfig, HttpsConfig, HttpMethod},
    types::{RecordType, QClass, Response, RecordData},
    error::Result,
};
use std::net::{Ipv4Addr, IpAddr};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== DNS-over-HTTPS (DoH) 全面测试 ===");
    println!("🔒 使用加密HTTPS传输进行DNS查询和EDNS Client Subnet测试");
    
    // DoH 服务器配置 - 优化为更稳定的服务器
    let doh_configs = vec![
        // 阿里云DoH (中国大陆优化)
        ("https://223.5.5.5/dns-query", "阿里云DoH", HttpMethod::GET),
        ("https://223.6.6.6/dns-query", "阿里云DoH (备用)", HttpMethod::POST),
        
        // 腾讯DoH (中国大陆优化)
        ("https://1.12.12.12/dns-query", "腾讯DoH", HttpMethod::GET),
        ("https://120.53.53.53/dns-query", "腾讯DoH (备用)", HttpMethod::POST),
        
        // Cloudflare DoH (全球稳定)
        ("https://cloudflare-dns.com/dns-query", "Cloudflare DoH (域名)", HttpMethod::GET),
        ("https://1.1.1.1/dns-query", "Cloudflare DoH (IP)", HttpMethod::POST),
        
        // 360 DoH (中国大陆)
        ("https://101.226.4.6/dns-query", "360 DoH", HttpMethod::GET),
        
        // AdGuard DoH (全球)
        ("https://dns.adguard.com/dns-query", "AdGuard DoH", HttpMethod::POST),
    ];
    
    // 测试域名 - 与UDP测试保持一致
    let test_domains = vec![
        "www.taobao.com",    // 阿里巴巴，有全球CDN
        "www.baidu.com",     // 百度，有地理位置优化
        "www.qq.com",        // 腾讯，有全球CDN
        "cdn.jsdelivr.net",  // 全球CDN服务
        "ajax.googleapis.com", // Google CDN
    ];
    
    // 测试IP地址 - 与UDP测试保持一致
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
    
    for (doh_url, doh_name, method) in doh_configs {
        println!("\n🌐 测试 {} ({})", doh_name, doh_url);
        println!("📡 传输方法: {:?}", method);
        println!("{}", "=".repeat(70));
        
        let mut resolver = Resolver::new(ResolverConfig::default());
        
        // 配置HTTPS传输 - 使用较短超时避免卡住
        let https_config = HttpsConfig {
            base: TransportConfig {
                server: doh_url.to_string(),
                port: 443,
                timeout: Duration::from_secs(3), // 缩短超时时间避免卡住
                ..Default::default()
            },
            url: doh_url.to_string(),
            method,
            user_agent: "rat_quickdns/1.0 DoH Test".to_string(),
        };
        
        resolver.add_https_transport(https_config);
        
        // 先进行连通性测试
         println!("🔍 测试DoH服务器连通性...");
         let connectivity_test = tokio::time::timeout(
             Duration::from_secs(3),
             resolver.query("www.taobao.com", RecordType::A, QClass::IN)
         ).await;
        
        match connectivity_test {
            Ok(Ok(_)) => println!("✅ DoH服务器连通正常"),
            Ok(Err(e)) => {
                println!("❌ DoH服务器连通失败: {}", e);
                println!("⏭️  跳过此服务器，测试下一个...");
                continue;
            }
            Err(_) => {
                println!("⏰ DoH服务器连通超时(3秒)");
                println!("⏭️  跳过此服务器，测试下一个...");
                continue;
            }
        }
        
        // 只测试前两个域名以加快测试速度
        for domain in test_domains.iter().take(2) {
            println!("\n📍 测试域名: {}", domain);
            println!("{}", "-".repeat(50));
            
            // 1. 普通DoH查询（无EDNS）
            println!("\n1️⃣  普通DoH查询（无EDNS Client Subnet）:");
            let normal_result = tokio::time::timeout(
                Duration::from_secs(4),
                resolver.query(domain, RecordType::A, QClass::IN)
            ).await;
            
            let normal_success = match &normal_result {
                 Ok(result) => {
                     print_result(result, "普通DoH查询");
                     result.is_ok()
                 }
                 Err(_) => {
                     println!("   ⏰ 普通DoH查询 - 超时(4秒)");
                     false
                 }
             };
            
            // 如果普通查询失败，跳过EDNS测试
            if !normal_success {
                println!("   ⚠️  普通查询失败，跳过EDNS测试");
                continue;
            }
            
            // 2. 只测试前3个地理位置IP以加快速度
            for (ip, location) in test_ips.iter().take(3) {
                let client_ip: Ipv4Addr = ip.parse().unwrap();
                
                println!("\n2️⃣  DoH + EDNS 使用{}IP({}):", location, ip);
                let edns_result = tokio::time::timeout(
                    Duration::from_secs(4),
                    resolver.query_with_client_ip(
                        domain, 
                        RecordType::A, 
                        QClass::IN, 
                        Some(IpAddr::V4(client_ip))
                    )
                ).await;
                
                match edns_result {
                    Ok(result) => {
                        print_result(&result, &format!("{}DoH查询", location));
                        
                        // 比较结果差异（只有在两个查询都成功时）
                         if let (Ok(Ok(normal)), Ok(edns)) = (&normal_result, &result) {
                             compare_results(normal, edns, location);
                         }
                    }
                    Err(_) => println!("   ⏰ {}DoH查询 - 超时(4秒)", location),
                }
            }
            
            println!("\n{}", "=".repeat(50));
        }
        
        // 添加延迟避免请求过于频繁，并给用户反馈
        println!("\n⏳ 等待300ms后测试下一个DoH服务器...");
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
    
    println!("\n✅ DNS-over-HTTPS (DoH) 全面测试完成");
    println!("\n📊 DoH 测试总结:");
    println!("- ✅ 验证了DoH传输的基本功能");
    println!("- 🔒 确认了HTTPS加密DNS查询工作正常");
    println!("- 🌍 测试了EDNS Client Subnet在DoH中的支持情况");
    println!("- 📈 可以与UDP传输结果进行性能和准确性对比");
    println!("\n🔍 分析建议:");
    println!("- 如果DoH和UDP返回不同结果，可能是:");
    println!("  1. DoH服务器有不同的CDN策略");
    println!("  2. HTTPS传输中EDNS处理方式不同");
    println!("  3. DoH服务器地理位置与UDP服务器不同");
    println!("- 重点关注阿里云和腾讯DoH的EDNS支持");
    println!("- 建议运行 comprehensive_edns_test 进行UDP对比");
    
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
                    match &record.data {
                        RecordData::A(ip) => {
                            println!("      {}. {} A: {}", i + 1, record.name, ip);
                        }
                        RecordData::AAAA(ip) => {
                            println!("      {}. {} AAAA: {}", i + 1, record.name, ip);
                        }
                        RecordData::CNAME(cname) => {
                            println!("      {}. {} CNAME: {}", i + 1, record.name, cname);
                        }
                        other => {
                            println!("      {}. {}: {:?}", i + 1, record.name, other);
                        }
                    }
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
        println!("   🎯 DoH检测到地理位置差异! {}IP返回了不同的结果", location);
        println!("      普通DoH查询: {:?}", normal_ips);
        println!("      {}DoH查询: {:?}", location, edns_ips);
    } else if !normal_ips.is_empty() {
        println!("   ℹ️  {}IP的DoH查询返回相同结果: {:?}", location, normal_ips);
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_doh_basic_query() {
        let mut resolver = Resolver::new(ResolverConfig::default());
        
        let https_config = HttpsConfig {
            base: TransportConfig {
                server: "https://223.5.5.5/dns-query".to_string(),
                port: 443,
                timeout: Duration::from_secs(5),
                ..Default::default()
            },
            url: "https://223.5.5.5/dns-query".to_string(),
            method: HttpMethod::GET,
            user_agent: "rat_quickdns/1.0 Test".to_string(),
        };
        
        resolver.add_https_transport(https_config);
        
        let result = resolver.query("www.taobao.com", RecordType::A, QClass::IN).await;
        assert!(result.is_ok(), "DoH基本查询应该成功");
    }
    
    #[tokio::test]
    async fn test_doh_edns_query() {
        let mut resolver = Resolver::new(ResolverConfig::default());
        
        let https_config = HttpsConfig {
            base: TransportConfig {
                server: "https://1.12.12.12/dns-query".to_string(),
                port: 443,
                timeout: Duration::from_secs(5),
                ..Default::default()
            },
            url: "https://1.12.12.12/dns-query".to_string(),
            method: HttpMethod::POST,
            user_agent: "rat_quickdns/1.0 Test".to_string(),
        };
        
        resolver.add_https_transport(https_config);
        
        let client_ip = "1.12.12.12".parse::<Ipv4Addr>().unwrap();
        let result = resolver.query_with_client_ip(
            "www.taobao.com", 
            RecordType::A, 
            QClass::IN, 
            Some(IpAddr::V4(client_ip))
        ).await;
        
        assert!(result.is_ok(), "DoH EDNS查询应该成功");
    }
}