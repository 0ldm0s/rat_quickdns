//! MX记录专项测试 - UDP协议
//! 测试多个DNS服务器的MX记录查询能力

use rat_quickdns::{
    builder::DnsResolverBuilder, RecordType, QueryStrategy,
    upstream_handler::UpstreamSpec,
    logger::{init_dns_logger, info, debug, error, warn, trace},
};
use rat_quickmem::QuickMemConfig;
use zerg_creep::logger::LevelFilter;
use std::time::Duration;
use tokio;

#[derive(Debug)]
struct MxTestCase {
    domain: &'static str,
    description: &'static str,
    expected_has_mx: bool,
}

#[derive(Debug)]
struct DnsServerConfig {
    name: &'static str,
    address: &'static str,
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

const DNS_SERVERS: &[DnsServerConfig] = &[
    // 国内DNS服务器
    DnsServerConfig {
        name: "腾讯DNS",
        address: "119.29.29.29",
        port: 53,
        region: "国内",
        resolved_ip: Some("119.29.29.29"), // 腾讯DNS服务器IP
    },
    DnsServerConfig {
        name: "阿里DNS",
        address: "223.5.5.5",
        port: 53,
        region: "国内",
        resolved_ip: Some("223.5.5.5"), // 阿里DNS服务器IP
    },
    DnsServerConfig {
        name: "百度DNS",
        address: "180.76.76.76",
        port: 53,
        region: "国内",
        resolved_ip: Some("180.76.76.76"), // 百度DNS服务器IP
    },
    DnsServerConfig {
        name: "114DNS",
        address: "114.114.114.114",
        port: 53,
        region: "国内",
        resolved_ip: Some("114.114.114.114"), // 114DNS服务器IP
    },
    // 国外DNS服务器
    DnsServerConfig {
        name: "Cloudflare",
        address: "1.1.1.1",
        port: 53,
        region: "国外",
        resolved_ip: Some("1.1.1.1"), // Cloudflare DNS服务器IP
    },
    DnsServerConfig {
        name: "Google",
        address: "8.8.8.8",
        port: 53,
        region: "国外",
        resolved_ip: Some("8.8.8.8"), // Google DNS服务器IP
    },
    DnsServerConfig {
        name: "Quad9",
        address: "9.9.9.9",
        port: 53,
        region: "国外",
        resolved_ip: Some("9.9.9.9"), // Quad9 DNS服务器IP
    },
    DnsServerConfig {
        name: "OpenDNS",
        address: "208.67.222.222",
        port: 53,
        region: "国外",
        resolved_ip: Some("208.67.222.222"), // OpenDNS服务器IP
    },
];

async fn test_mx_record_with_server(
    server: &DnsServerConfig,
    test_case: &MxTestCase,
) -> Result<(bool, Duration, Vec<String>), String> {
    let start = std::time::Instant::now();
    
    info!("🔍 开始查询: {} 通过 {}({})", test_case.domain, server.name, server.address);
    
    // 创建QuickMem配置
    let quickmem_config = QuickMemConfig {
        max_data_size: 64 * 1024 * 1024, // 64MB
        max_batch_count: 10000,
        pool_initial_capacity: 1024,
        pool_max_capacity: 10240,
        enable_parallel: true,
    };
    
    // 创建带有预解析IP地址的UDP上游配置
    let mut udp_spec = UpstreamSpec::udp(
        format!("{}-{}", server.name, server.region),
        format!("{}:{}", server.address, server.port)
    );
    
    // 如果有预解析IP地址，则设置它
    if let Some(resolved_ip) = server.resolved_ip {
        udp_spec = udp_spec.with_resolved_ip(resolved_ip.to_string());
    }
    
    let resolver = DnsResolverBuilder::new(
        QueryStrategy::Smart,
        true, // 启用EDNS
        "global".to_string(), // 当前区域
        quickmem_config,
    )
        .with_timeout(Duration::from_secs(10))
        .with_retry_count(2)
        .with_verbose_logging()  // 启用详细日志
        .add_upstream(udp_spec)  // 使用带有预解析IP的上游配置
        .map_err(|e| format!("添加UDP上游失败: {}", e))?
        .build()
        .await
        .map_err(|e| {
            error!("构建解析器失败: {}", e);
            format!("构建解析器失败: {}", e)
        })?;
    
    debug!("✅ 解析器构建成功，上游服务器: {}:{}", server.address, server.port);

    let request = rat_quickdns::builder::types::DnsQueryRequest::new(
        test_case.domain,
        rat_quickdns::builder::types::DnsRecordType::MX,
    );
    
    debug!("📤 发送DNS查询请求: 域名={}, 记录类型=MX", test_case.domain);
    trace!("📋 查询请求详情: {:?}", request);
    
    match resolver.query(request).await {
        Ok(response) => {
            let duration = start.elapsed();
            
            info!("📥 收到DNS响应，耗时: {:?}", duration);
            debug!("📊 响应状态: success={}, records_count={}", response.success, response.records.len());
            trace!("📄 完整响应: {:?}", response);
            
            if response.success {
                let mx_list = response.mx_records();
                debug!("📧 提取到 {} 条MX记录: {:?}", mx_list.len(), mx_list);
                
                let mx_records: Vec<String> = mx_list.into_iter().map(|(priority, exchange)| {
                    let record_str = format!("{}:{}", priority, exchange);
                    trace!("📧 MX记录格式化: {} -> {}", exchange, record_str);
                    record_str
                }).collect();
                
                if mx_records.is_empty() {
                    warn!("⚠️ 未找到MX记录: {}", test_case.domain);
                } else {
                    info!("✅ 成功获取 {} 条MX记录", mx_records.len());
                }
                
                Ok((true, duration, mx_records))
            } else {
                warn!("⚠️ DNS查询成功但响应标记为失败");
                // 查询失败，返回错误信息
                let error_msg = response.error.unwrap_or_else(|| "未知错误".to_string());
                Err(format!("UDP查询失败: {} (耗时: {:?})", error_msg, duration))
            }
        }
        Err(e) => {
            let duration = start.elapsed();
            error!("❌ DNS查询失败: {} (耗时: {:?})", e, duration);
            Err(format!("查询失败: {} (耗时: {:?})", e, duration))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化详细日志系统
    init_dns_logger(LevelFilter::Trace)?;
    
    println!("🚀 MX记录专项测试 - UDP协议");
    println!("测试 {} 个DNS服务器 × {} 个域名 = {} 个查询", 
             DNS_SERVERS.len(), TEST_DOMAINS.len(), 
             DNS_SERVERS.len() * TEST_DOMAINS.len());
    println!("============================================================\n");
    
    info!("开始MX记录专项测试，日志级别: TRACE");
    debug!("测试域名列表: {:?}", TEST_DOMAINS.iter().map(|t| t.domain).collect::<Vec<_>>());
    debug!("DNS服务器列表: {:?}", DNS_SERVERS.iter().map(|s| format!("{}({})", s.name, s.address)).collect::<Vec<_>>());

    let mut total_queries = 0;
    let mut successful_queries = 0;
    let mut total_duration = Duration::new(0, 0);
    
    // 按DNS服务器分组测试
    for server in DNS_SERVERS {
        println!("📡 测试DNS服务器: {} ({}) - {}", 
                 server.name, server.address, server.region);
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
            
            match test_mx_record_with_server(server, test_case).await {
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
    
    println!("📈 UDP协议 MX记录查询统计摘要:");
    println!("  总查询数: {}", total_queries);
    println!("  成功查询数: {}", successful_queries);
    println!("  总体成功率: {:.1}%", overall_success_rate);
    println!("  平均查询耗时: {:?}", avg_duration);
    
    // 按地区统计
    println!("\n📊 按地区统计:");
    for region in ["国内", "国外"] {
        let region_servers: Vec<_> = DNS_SERVERS.iter().filter(|s| s.region == region).collect();
        println!("  {} DNS服务器: {} 个", region, region_servers.len());
        for server in region_servers {
            println!("    - {} ({})", server.name, server.address);
        }
    }
    
    // 性能分析
    println!("\n🔍 UDP协议特点分析:");
    println!("   ✅ 优势:");
    println!("      - 无连接协议，查询速度快");
    println!("      - 网络开销小，适合高频查询");
    println!("      - 支持并发查询，性能优异");
    println!("      - 预解析IP地址，避免DNS解析延迟");
    println!("   ⚠️  注意事项:");
    println!("      - 可能受到网络丢包影响");
    println!("      - 某些网络环境可能限制UDP流量");
    println!("      - 预解析IP需要定期更新以保持有效性");
    
    println!("\n💡 建议:");
    println!("   1. 如果国内DNS服务器MX查询成功率低，可能是网络策略限制");
    println!("   2. 如果国外DNS服务器查询失败，可能是网络连接问题");
    println!("   3. 建议优先使用成功率高的DNS服务器进行MX记录查询");
    println!("   4. UDP协议查询速度快，但可能受到网络环境影响");
    println!("   5. 使用预解析IP地址可减少连接建立时间，但需定期验证有效性");
    
    Ok(())
}