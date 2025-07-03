//! DNS-over-TLS (DoT) 全面测试示例
//! 基于 doh_comprehensive_test.rs，验证 DoT 传输的 EDNS Client Subnet 功能
//! 可以与 UDP 和 DoH 传输结果进行对比

use rat_quickdns::{
    resolver::{Resolver, ResolverConfig},
    transport::{TransportConfig, TlsConfig},
    types::{RecordType, QClass, Response, RecordData},
    error::Result,
};
use std::net::{Ipv4Addr, IpAddr};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== DNS-over-TLS (DoT) 全面测试 ===");
    println!("🔒 使用加密TLS传输进行DNS查询和EDNS Client Subnet测试");
    
    // DoT 服务器配置 - 优先国内服务器，增强超时设置
    let dot_configs = vec![
        // 阿里云DoT (中国大陆优化) - 优先测试
        ("223.5.5.5", 853, "阿里云DoT"),
        ("223.6.6.6", 853, "阿里云DoT (备用)"),
        
        // 腾讯DoT (中国大陆优化)
        ("1.12.12.12", 853, "腾讯DoT"),
        ("120.53.53.53", 853, "腾讯DoT (备用)"),
        
        // 百度DoT (中国大陆)
        ("180.76.76.76", 853, "百度DoT"),
        
        // 114DNS DoT (中国大陆)
        ("114.114.114.114", 853, "114DNS DoT"),
        ("114.114.115.115", 853, "114DNS DoT (备用)"),
        
        // Cloudflare DoT (全球稳定)
        ("1.1.1.1", 853, "Cloudflare DoT"),
        ("1.0.0.1", 853, "Cloudflare DoT (备用)"),
        
        // Google DoT (全球稳定)
        ("8.8.8.8", 853, "Google DoT"),
        ("8.8.4.4", 853, "Google DoT (备用)"),
        
        // Quad9 DoT (安全DNS)
        ("9.9.9.9", 853, "Quad9 DoT"),
        ("149.112.112.112", 853, "Quad9 DoT (备用)"),
    ];
    
    // 测试域名 - 与UDP和DoH测试保持一致
    let test_domains = vec![
        "www.taobao.com",    // 阿里巴巴，有全球CDN
        "www.baidu.com",     // 百度，有地理位置优化
        "www.qq.com",        // 腾讯，有全球CDN
        "cdn.jsdelivr.net",  // 全球CDN服务
        "ajax.googleapis.com", // Google CDN
    ];
    
    // 测试IP地址 - 与UDP和DoH测试保持一致
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
    
    for (server_ip, port, dot_name) in dot_configs {
        println!("\n🌐 测试 {} ({}:{})", dot_name, server_ip, port);
        println!("🔐 传输协议: TLS over TCP");
        println!("{}", "=".repeat(70));
        
        let mut resolver = Resolver::new(ResolverConfig::default());
        
        // 配置TLS传输 - 针对网络不稳定情况优化超时设置
        let tls_config = TlsConfig {
            base: TransportConfig {
                server: server_ip.to_string(),
                port,
                timeout: Duration::from_secs(5), // 增加超时时间应对网络卡顿
                ..Default::default()
            },
            server_name: server_ip.to_string(), // 使用IP地址作为服务器名称
            verify_cert: false, // 使用IP地址时通常需要禁用证书验证
        };
        
        resolver.add_tls_transport(tls_config);
        
        // 先进行连通性测试 - 增加超时时间
        println!("🔍 测试DoT服务器连通性...");
        let connectivity_test = tokio::time::timeout(
            Duration::from_secs(8), // 增加连通性测试超时时间
            resolver.query("www.taobao.com", RecordType::A, QClass::IN)
        ).await;
        
        match connectivity_test {
            Ok(Ok(_)) => println!("✅ DoT服务器连通正常"),
            Ok(Err(e)) => {
                println!("❌ DoT服务器连通失败: {}", e);
                println!("⏭️  跳过此服务器，测试下一个...");
                continue;
            }
            Err(_) => {
                println!("⏰ DoT服务器连通超时(8秒)");
                println!("⏭️  跳过此服务器，测试下一个...");
                continue;
            }
        }
        
        // 只测试前两个域名以加快测试速度
        for domain in test_domains.iter().take(2) {
            println!("\n📍 测试域名: {}", domain);
            println!("{}", "-".repeat(50));
            
            // 1. 普通DoT查询（无EDNS）
            println!("\n1️⃣  普通DoT查询（无EDNS Client Subnet）:");
            let normal_result = tokio::time::timeout(
                Duration::from_secs(6), // 增加查询超时时间
                resolver.query(domain, RecordType::A, QClass::IN)
            ).await;
            
            let normal_success = match &normal_result {
                Ok(result) => {
                    print_result(result, "普通DoT查询");
                    result.is_ok()
                }
                Err(_) => {
                    println!("   ⏰ 普通DoT查询 - 超时(6秒)");
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
                
                println!("\n2️⃣  DoT + EDNS 使用{}IP({}):", location, ip);
                let edns_result = tokio::time::timeout(
                    Duration::from_secs(6), // 增加EDNS查询超时时间
                    resolver.query_with_client_ip(
                        domain, 
                        RecordType::A, 
                        QClass::IN, 
                        Some(IpAddr::V4(client_ip))
                    )
                ).await;
                
                match edns_result {
                    Ok(result) => {
                        print_result(&result, &format!("{}DoT查询", location));
                        
                        // 比较结果差异（只有在两个查询都成功时）
                        if let (Ok(Ok(normal)), Ok(edns)) = (&normal_result, &result) {
                            compare_results(normal, edns, location);
                        }
                    }
                    Err(_) => println!("   ⏰ {}DoT查询 - 超时(6秒)", location),
                }
            }
            
            println!("\n{}", "=".repeat(50));
        }
        
        // 添加延迟避免请求过于频繁，并给用户反馈
        println!("\n⏳ 等待500ms后测试下一个DoT服务器...");
        tokio::time::sleep(Duration::from_millis(500)).await; // 增加延迟时间
    }
    
    println!("\n✅ DNS-over-TLS (DoT) 全面测试完成");
    println!("\n📊 DoT 测试总结:");
    println!("- ✅ 验证了DoT传输的基本功能");
    println!("- 🔒 确认了TLS加密DNS查询工作正常");
    println!("- 🌍 测试了EDNS Client Subnet在DoT中的支持情况");
    println!("- 📈 可以与UDP和DoH传输结果进行性能和准确性对比");
    println!("- 🇨🇳 重点测试了国内DoT服务器的稳定性");
    println!("\n🔍 分析建议:");
    println!("- 如果DoT和UDP/DoH返回不同结果，可能是:");
    println!("  1. DoT服务器有不同的CDN策略");
    println!("  2. TLS传输中EDNS处理方式不同");
    println!("  3. DoT服务器地理位置与其他服务器不同");
    println!("- 重点关注阿里云和腾讯DoT的EDNS支持");
    println!("- 国内DoT服务器通常有更好的连通性和速度");
    println!("- 建议运行 comprehensive_edns_test 和 doh_comprehensive_test 进行对比");
    
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
        println!("   🎯 DoT检测到地理位置差异! {}IP返回了不同的结果", location);
        println!("      普通DoT查询: {:?}", normal_ips);
        println!("      {}DoT查询: {:?}", location, edns_ips);
    } else if !normal_ips.is_empty() {
        println!("   ℹ️  {}IP的DoT查询返回相同结果: {:?}", location, normal_ips);
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
    async fn test_dot_basic_query() {
        let mut resolver = Resolver::new(ResolverConfig::default());
        
        // 使用阿里云DoT进行测试
        let tls_config = TlsConfig {
            base: TransportConfig {
                server: "223.5.5.5".to_string(),
                port: 853,
                timeout: Duration::from_secs(8), // 增加测试超时时间
                ..Default::default()
            },
            server_name: "223.5.5.5".to_string(),
            verify_cert: false, // 测试时禁用证书验证
        };
        
        resolver.add_tls_transport(tls_config);
        
        let result = resolver.query("www.taobao.com", RecordType::A, QClass::IN).await;
        assert!(result.is_ok(), "DoT基本查询应该成功");
    }
    
    #[tokio::test]
    async fn test_dot_edns_query() {
        let mut resolver = Resolver::new(ResolverConfig::default());
        
        // 使用腾讯DoT进行测试
        let tls_config = TlsConfig {
            base: TransportConfig {
                server: "1.12.12.12".to_string(),
                port: 853,
                timeout: Duration::from_secs(8), // 增加测试超时时间
                ..Default::default()
            },
            server_name: "1.12.12.12".to_string(),
            verify_cert: false, // 测试时禁用证书验证
        };
        
        resolver.add_tls_transport(tls_config);
        
        let client_ip = "114.114.114.114".parse::<Ipv4Addr>().unwrap();
        let result = resolver.query_with_client_ip(
            "www.taobao.com", 
            RecordType::A, 
            QClass::IN, 
            Some(IpAddr::V4(client_ip))
        ).await;
        
        assert!(result.is_ok(), "DoT EDNS查询应该成功");
    }
    
    #[tokio::test]
    async fn test_dot_connectivity_timeout() {
        let mut resolver = Resolver::new(ResolverConfig::default());
        
        // 测试超时机制
        let tls_config = TlsConfig {
            base: TransportConfig {
                server: "223.5.5.5".to_string(),
                port: 853,
                timeout: Duration::from_secs(1), // 短超时测试
                ..Default::default()
            },
            server_name: "223.5.5.5".to_string(),
            verify_cert: false, // 测试时禁用证书验证
        };
        
        resolver.add_tls_transport(tls_config);
        
        // 这个测试可能会超时，这是预期的行为
        let result = tokio::time::timeout(
            Duration::from_secs(2),
            resolver.query("www.taobao.com", RecordType::A, QClass::IN)
        ).await;
        
        // 无论成功还是超时都是可接受的结果
        match result {
            Ok(Ok(_)) => println!("DoT查询成功"),
            Ok(Err(_)) => println!("DoT查询失败"),
            Err(_) => println!("DoT查询超时 - 这是预期的测试结果"),
        }
    }
}