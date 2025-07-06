//! MX记录专项测试 - DoH (DNS over HTTPS) 协议
//! 测试多个DoH服务器的MX记录查询能力

use rat_quickdns::{
    DnsResolverBuilder, RecordType, QueryStrategy,
};
use rat_quickmem::QuickMemConfig;
use std::time::Duration;
use tokio;

#[derive(Debug)]
struct MxTestCase {
    domain: &'static str,
    description: &'static str,
    expected_has_mx: bool,
}

#[derive(Debug)]
struct DohServerConfig {
    name: &'static str,
    url: &'static str,
    region: &'static str,
    resolved_ip: Option<&'static str>, // 预解析的IP地址，避免DNS解析延迟
}

const TEST_DOMAINS: &[MxTestCase] = &[
    MxTestCase {
        domain: "qq.com",
        description: "腾讯QQ邮箱",
        expected_has_mx: true,
    },
    MxTestCase {
        domain: "163.com",
        description: "网易邮箱",
        expected_has_mx: true,
    },
    MxTestCase {
        domain: "sina.com.cn",
        description: "新浪邮箱",
        expected_has_mx: true,
    },
    MxTestCase {
        domain: "gmail.com",
        description: "Google邮箱",
        expected_has_mx: true,
    },
    MxTestCase {
        domain: "outlook.com",
        description: "微软邮箱",
        expected_has_mx: true,
    },
    MxTestCase {
        domain: "yahoo.com",
        description: "雅虎邮箱",
        expected_has_mx: true,
    },
    MxTestCase {
        domain: "126.com",
        description: "网易126邮箱",
        expected_has_mx: true,
    },
    MxTestCase {
        domain: "foxmail.com",
        description: "腾讯Foxmail",
        expected_has_mx: true,
    },
];

const DOH_SERVERS: &[DohServerConfig] = &[
    // 国内DoH服务器
    DohServerConfig {
        name: "腾讯DoH",
        url: "https://doh.pub/dns-query",
        region: "国内",
        resolved_ip: Some("1.12.12.12"), // 腾讯公共DNS的IP地址
    },
    DohServerConfig {
        name: "阿里DoH",
        url: "https://dns.alidns.com/dns-query",
        region: "国内",
        resolved_ip: Some("223.5.5.5"), // 阿里公共DNS的IP地址
    },
    DohServerConfig {
        name: "360DoH",
        url: "https://doh.360.cn/dns-query",
        region: "国内",
        resolved_ip: Some("101.226.4.6"), // 360公共DNS的IP地址
    },
    // 国外DoH服务器
    DohServerConfig {
        name: "Cloudflare DoH",
        url: "https://cloudflare-dns.com/dns-query",
        region: "国外",
        resolved_ip: Some("1.1.1.1"), // Cloudflare DNS的IP地址
    },
    DohServerConfig {
        name: "Google DoH",
        url: "https://dns.google/dns-query",
        region: "国外",
        resolved_ip: Some("8.8.8.8"), // Google DNS的IP地址
    },
    DohServerConfig {
        name: "Quad9 DoH",
        url: "https://dns.quad9.net/dns-query",
        region: "国外",
        resolved_ip: Some("9.9.9.9"), // Quad9 DNS的IP地址
    },
    DohServerConfig {
        name: "AdGuard DoH",
        url: "https://dns.adguard.com/dns-query",
        region: "国外",
        resolved_ip: Some("94.140.14.14"), // AdGuard DNS的IP地址
    },
    DohServerConfig {
        name: "OpenDNS DoH",
        url: "https://doh.opendns.com/dns-query",
        region: "国外",
        resolved_ip: Some("208.67.222.222"), // OpenDNS的IP地址
    },
];

async fn test_mx_record_with_doh_server(
    server: &DohServerConfig,
    test_case: &MxTestCase,
) -> Result<(bool, Duration, Vec<String>), String> {
    let start = std::time::Instant::now();
    
    // 创建QuickMem配置
    let quickmem_config = rat_quickmem::QuickMemConfig {
        max_data_size: 64 * 1024 * 1024, // 64MB
        max_batch_count: 10000,
        pool_initial_capacity: 1024,
        pool_max_capacity: 10240,
        enable_parallel: true,
    };
    
    // 创建带有预解析IP地址的DoH上游配置
    let mut doh_spec = rat_quickdns::upstream_handler::UpstreamSpec::doh(
        format!("{}-{}", server.name, server.region),
        server.url.to_string()
    );
    
    // 如果有预解析IP地址，则设置它
    if let Some(resolved_ip) = server.resolved_ip {
        doh_spec = doh_spec.with_resolved_ip(resolved_ip.to_string());
    }
    
    let resolver = DnsResolverBuilder::new(
        QueryStrategy::Smart,
        true, // 启用EDNS
        "global".to_string(), // 当前区域
        quickmem_config,
    )
        .with_timeout(Duration::from_secs(5))  // 减少超时时间，实现快速失败
        .with_retry_count(1)  // 减少重试次数，加快失败检测
        .add_upstream(doh_spec)  // 使用带有预解析IP的上游配置
        .map_err(|e| format!("添加DoH上游失败: {}", e))?
        .build()
        .await
        .map_err(|e| format!("构建DoH解析器失败: {}", e))?;

    let request = rat_quickdns::builder::types::DnsQueryRequest::new(
        test_case.domain,
        rat_quickdns::builder::types::DnsRecordType::MX,
    );
    
    match resolver.query(request).await {
        Ok(response) => {
            let duration = start.elapsed();
            if response.success {
                let mx_records: Vec<String> = response.mx_records().into_iter().map(|(priority, exchange)| {
                    format!("{}:{}", priority, exchange)
                }).collect();
                Ok((true, duration, mx_records))
            } else {
                // 查询失败，返回错误信息
                let error_msg = response.error.unwrap_or_else(|| "未知错误".to_string());
                Err(format!("DoH查询失败: {} (耗时: {:?})", error_msg, duration))
            }
        }
        Err(e) => {
            let duration = start.elapsed();
            Err(format!("DoH查询失败: {} (耗时: {:?})", e, duration))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 MX记录专项测试 - DoH (DNS over HTTPS) 协议");
    println!("测试 {} 个DoH服务器 × {} 个域名 = {} 个查询", 
             DOH_SERVERS.len(), TEST_DOMAINS.len(), 
             DOH_SERVERS.len() * TEST_DOMAINS.len());
    println!("============================================================\n");

    let mut total_queries = 0;
    let mut successful_queries = 0;
    let mut total_duration = Duration::new(0, 0);
    
    // 按DoH服务器分组测试
    for server in DOH_SERVERS {
        println!("🔒 测试DoH服务器: {} - {}", server.name, server.region);
        println!("   URL: {}", server.url);
        if let Some(resolved_ip) = server.resolved_ip {
            println!("   预解析IP: {} (避免DNS解析延迟)", resolved_ip);
        } else {
            println!("   预解析IP: 未设置 (将进行DNS解析)");
        }
        println!("  状态 |           域名 |     耗时 | MX记录数 | 描述");
        println!("  ─────────────────────────────────────────────────────────────────");
        
        let mut server_success = 0;
        let mut server_total = 0;
        
        for test_case in TEST_DOMAINS {
            total_queries += 1;
            server_total += 1;
            
            match test_mx_record_with_doh_server(server, test_case).await {
                Ok((success, duration, mx_records)) => {
                    if success {
                        successful_queries += 1;
                        server_success += 1;
                        total_duration += duration;
                        
                        if mx_records.is_empty() {
                            println!("  ⚠️  | {:>15} | {:>8} | {:>8} | {} (无MX记录)", 
                                     test_case.domain, 
                                     format!("{:?}", duration),
                                     "0",
                                     test_case.description);
                        } else {
                            println!("  ✅ | {:>15} | {:>8} | {:>8} | {}", 
                                     test_case.domain, 
                                     format!("{:?}", duration),
                                     mx_records.len(),
                                     test_case.description);
                            
                            // 显示前3个MX记录
                            for (i, mx_record) in mx_records.iter().take(3).enumerate() {
                                println!("    📧 MX{}: {}", i + 1, mx_record);
                            }
                            if mx_records.len() > 3 {
                                println!("    📧 ... 还有{}个MX记录", mx_records.len() - 3);
                            }
                        }
                    }
                }
                Err(error_msg) => {
                    println!("  ❌ | {:>15} | {:>8} | {:>8} | {} - {}", 
                             test_case.domain, 
                             "超时",
                             "0",
                             test_case.description,
                             error_msg);
                }
            }
        }
        
        let server_success_rate = (server_success as f64 / server_total as f64) * 100.0;
        println!("  📊 {} 成功率: {:.1}% ({}/{})", 
                 server.name, server_success_rate, server_success, server_total);
        println!();
    }
    
    // 总体统计
    let overall_success_rate = (successful_queries as f64 / total_queries as f64) * 100.0;
    let avg_duration = if successful_queries > 0 {
        total_duration / successful_queries as u32
    } else {
        Duration::new(0, 0)
    };
    
    println!("📈 DoH协议 MX记录查询统计摘要:");
    println!("  总查询数: {}", total_queries);
    println!("  成功查询数: {}", successful_queries);
    println!("  总体成功率: {:.1}%", overall_success_rate);
    println!("  平均查询耗时: {:?}", avg_duration);
    
    // 按地区统计
    println!("\n📊 按地区统计:");
    for region in ["国内", "国外"] {
        let region_servers: Vec<_> = DOH_SERVERS.iter().filter(|s| s.region == region).collect();
        println!("  {} DoH服务器: {} 个", region, region_servers.len());
        for server in region_servers {
            println!("    - {} ({})", server.name, server.url);
        }
    }
    
    // 性能分析
    println!("\n🔍 DoH协议特点分析:");
    println!("   ✅ 优势:");
    println!("      - 加密传输，安全性高");
    println!("      - 可穿越防火墙和网络过滤");
    println!("      - 支持HTTP/2多路复用");
    println!("      - 预解析IP地址，避免DNS解析延迟");
    println!("   ⚠️  注意事项:");
    println!("      - 首次连接需要TLS握手，延迟较高");
    println!("      - 需要HTTPS证书验证");
    println!("      - 某些网络环境可能阻止HTTPS DNS查询");
    println!("      - 预解析IP需要定期更新以保持有效性");
    
    println!("\n💡 建议:");
    println!("   - DoH适合对隐私和安全要求高的场景");
    println!("   - 国内DoH服务器通常访问速度更快");
    println!("   - 预解析IP地址可显著减少连接建立时间");
    println!("   - 如果MX查询失败，可能是DoH服务器策略限制");
    println!("   - 建议配合其他协议(UDP/DoT)使用以提高可靠性");
    println!("   - 定期验证预解析IP地址的有效性");
    
    Ok(())
}