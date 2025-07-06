//! MX记录专项测试 - DoT (DNS over TLS) 协议
//! 测试多个DoT服务器的MX记录查询能力

use rat_quickdns::{
    builder::DnsResolverBuilder, QueryStrategy,
    upstream_handler::UpstreamSpec,
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
struct DotServerConfig {
    name: &'static str,
    hostname: &'static str, // 用于TLS SNI和连接
    port: u16,
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

const DOT_SERVERS: &[DotServerConfig] = &[
    // 国内DoT服务器
    DotServerConfig {
        name: "腾讯DoT",
        hostname: "dot.pub",
        port: 853,
        region: "国内",
        resolved_ip: Some("1.12.12.12"), // 使用腾讯DNS的IP地址
    },
    DotServerConfig {
        name: "阿里DoT",
        hostname: "dns.alidns.com",
        port: 853,
        region: "国内",
        resolved_ip: Some("223.5.5.5"), // 使用阿里DNS的IP地址
    },
    DotServerConfig {
        name: "腾讯备用DoT",
        hostname: "dot.pub",
        port: 853,
        region: "国内",
        resolved_ip: Some("120.53.53.53"), // 使用腾讯备用DNS的IP地址
    },
    // 国外DoT服务器
    DotServerConfig {
        name: "Cloudflare DoT",
        hostname: "cloudflare-dns.com",
        port: 853,
        region: "国外",
        resolved_ip: Some("1.1.1.1"), // 使用Cloudflare DNS的IP地址
    },
    DotServerConfig {
        name: "Google DoT",
        hostname: "dns.google",
        port: 853,
        region: "国外",
        resolved_ip: Some("8.8.8.8"), // 使用Google DNS的IP地址
    },
    DotServerConfig {
        name: "Quad9 DoT",
        hostname: "dns.quad9.net",
        port: 853,
        region: "国外",
        resolved_ip: Some("9.9.9.9"), // 使用Quad9 DNS的IP地址
    },
    DotServerConfig {
        name: "AdGuard DoT",
        hostname: "dns.adguard.com",
        port: 853,
        region: "国外",
        resolved_ip: Some("94.140.14.14"), // 使用AdGuard DNS的IP地址
    },
    DotServerConfig {
        name: "CleanBrowsing DoT",
        hostname: "security-filter-dns.cleanbrowsing.org",
        port: 853,
        region: "国外",
        resolved_ip: Some("185.228.168.9"), // 使用CleanBrowsing DNS的IP地址
    },
];

async fn test_mx_record_with_dot_server(
    server: &DotServerConfig,
    test_case: &MxTestCase,
) -> Result<(bool, Duration, Vec<String>), String> {
    let start = std::time::Instant::now();
    
    // 创建 QuickMem 配置
    let quickmem_config = QuickMemConfig {
        max_data_size: 64 * 1024 * 1024, // 64MB
        max_batch_count: 10000,
        pool_initial_capacity: 1024,
        pool_max_capacity: 10240,
        enable_parallel: true,
    };
    
    // 创建带有预解析IP地址的DoT上游配置
    let mut dot_spec = UpstreamSpec::dot(
        format!("{}-{}", server.name, server.region),
        format!("{}:{}", server.hostname, server.port) // 使用hostname作为server字段，用于SNI
    );
    
    // 如果有预解析IP地址，则设置它
    if let Some(resolved_ip) = server.resolved_ip {
        dot_spec = dot_spec.with_resolved_ip(resolved_ip.to_string());
    }
    
    let resolver = DnsResolverBuilder::new(
        QueryStrategy::Smart,
        true,  // 启用 EDNS
        "global".to_string(),
        quickmem_config,
    )
    .with_timeout(Duration::from_secs(3)) // 快速失败，避免长时间等待
    .with_retry_count(1) // 减少重试次数以加快失败检测
    .add_upstream(dot_spec)  // 使用带有预解析IP的上游配置
    .map_err(|e| format!("添加DoT上游失败: {}", e))?
    .build()
    .await
    .map_err(|e| format!("构建DoT解析器失败: {}", e))?;

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
                Err(format!("DoT查询失败: {} (耗时: {:?})", error_msg, duration))
            }
        }
        Err(e) => {
            let duration = start.elapsed();
            Err(format!("DoT查询失败: {} (耗时: {:?})", e, duration))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 MX记录专项测试 - DoT (DNS over TLS) 协议");
    println!("测试 {} 个DoT服务器 × {} 个域名 = {} 个查询", 
             DOT_SERVERS.len(), TEST_DOMAINS.len(), 
             DOT_SERVERS.len() * TEST_DOMAINS.len());
    println!("============================================================\n");

    let mut total_queries = 0;
    let mut successful_queries = 0;
    let mut total_duration = Duration::new(0, 0);
    
    // 按DoT服务器分组测试
    for server in DOT_SERVERS {
        println!("🔐 测试DoT服务器: {} - {}", server.name, server.region);
        println!("   地址: {}:{} (TLS: {})", server.hostname, server.port, server.hostname);
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
            
            match test_mx_record_with_dot_server(server, test_case).await {
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
    
    println!("📈 DoT协议 MX记录查询统计摘要:");
    println!("  总查询数: {}", total_queries);
    println!("  成功查询数: {}", successful_queries);
    println!("  总体成功率: {:.1}%", overall_success_rate);
    println!("  平均查询耗时: {:?}", avg_duration);
    
    // 按地区统计
    println!("\n📊 按地区统计:");
    for region in ["国内", "国外"] {
        let region_servers: Vec<_> = DOT_SERVERS.iter().filter(|s| s.region == region).collect();
        println!("  {} DoT服务器: {} 个", region, region_servers.len());
        for server in region_servers {
            println!("    - {} ({}:{})", server.name, server.hostname, server.port);
        }
    }
    
    // 性能分析
    println!("\n🔍 DoT协议特点分析:");
    println!("   ✅ 优势:");
    println!("      - 基于TLS加密，安全性高");
    println!("      - 使用标准853端口，兼容性好");
    println!("      - 支持TCP连接复用");
    println!("      - 延迟通常比DoH更低");
    println!("      - 预解析IP地址，避免DNS解析延迟");
    println!("   ⚠️  注意事项:");
    println!("      - 需要TLS握手，首次连接有延迟");
    println!("      - 某些网络可能阻止853端口");
    println!("      - 需要正确的TLS证书验证");
    println!("      - 预解析IP需要定期更新以保持有效性");
    
    // 协议对比
    println!("\n📊 协议性能对比建议:");
    println!("   🚀 UDP: 速度最快，但无加密保护");
    println!("   🔐 DoT: 平衡安全性和性能，推荐日常使用");
    println!("   🔒 DoH: 最高安全性，适合严格安全要求场景");
    
    println!("\n💡 建议:");
    println!("   - DoT是安全DNS查询的首选协议");
    println!("   - 国内DoT服务器通常连接更稳定");
    println!("   - 如果MX查询失败，可能是服务器策略或网络限制");
    println!("   - 建议配置多个DoT服务器作为备选");
    println!("   - 企业环境建议优先使用DoT协议");
    println!("   - 使用预解析IP地址可减少连接建立时间，但需定期验证有效性");
    
    Ok(())
}