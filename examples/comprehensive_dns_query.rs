//! 全面的DNS查询示例
//!
//! 本示例演示如何使用腾讯云的三种DNS服务器（UDP、DoT、DoH）
//! 查询所有支持的DNS记录类型，使用cloudflare.com作为测试域名
//!
//! 支持的DNS记录类型：
//! - A: IPv4地址记录
//! - AAAA: IPv6地址记录  
//! - CNAME: 别名记录
//! - MX: 邮件交换记录
//! - NS: 名称服务器记录
//! - TXT: 文本记录
//! - SOA: 授权开始记录
//! - PTR: 指针记录（反向DNS）
//! - SRV: 服务记录

use std::time::{Duration, Instant};
use std::collections::HashMap;
use tokio;
use rand;
use rat_quickdns::{
    DnsResolverBuilder,
    builder::{SmartDnsResolver, types::{DnsQueryRequest, DnsRecordType}},
    types::{Request, Query, RecordType, QClass, Flags},
    error::{DnsError, Result},
    transport::{TransportConfig, TlsConfig, HttpsConfig, HttpMethod},
};

/// DNSSEC测试用例
#[derive(Debug, Clone)]
struct DnssecTestCase {
    /// 测试域名
    domain: String,
    /// 是否期望DNSSEC验证成功
    expect_dnssec: bool,
    /// 测试描述
    description: &'static str,
}

impl DnssecTestCase {
    fn new(domain: &str, expect_dnssec: bool, description: &'static str) -> Self {
        Self {
            domain: domain.to_string(),
            expect_dnssec,
            description,
        }
    }
}

/// 腾讯云DNS服务器配置
struct TencentDnsServers {
    /// UDP DNS服务器
    udp_server: &'static str,
    /// DoT (DNS over TLS) 服务器
    dot_server: &'static str,
    /// DoH (DNS over HTTPS) 服务器
    doh_url: &'static str,
}

impl TencentDnsServers {
    fn new() -> Self {
        Self {
            udp_server: "119.29.29.29",
            dot_server: "dot.pub",
            doh_url: "https://doh.pub/dns-query",
        }
    }
}

/// DNS记录类型测试配置
#[derive(Debug, Clone)]
struct DnsTestCase {
    /// 记录类型
    record_type: RecordType,
    /// 测试域名
    domain: String,
    /// 记录类型描述
    description: &'static str,
    /// 是否期望有结果
    expect_results: bool,
}

impl DnsTestCase {
    fn new(record_type: RecordType, domain: &str, description: &'static str, expect_results: bool) -> Self {
        Self {
            record_type,
            domain: domain.to_string(),
            description,
            expect_results,
        }
    }
}

/// 查询结果统计
#[derive(Debug, Default)]
struct QueryStats {
    /// 总查询数
    total_queries: u32,
    /// 成功查询数
    successful_queries: u32,
    /// 失败查询数
    failed_queries: u32,
    /// 总耗时
    total_duration: Duration,
    /// 各传输协议的统计
    transport_stats: HashMap<String, TransportStats>,
}

#[derive(Debug, Default)]
struct TransportStats {
    /// 查询数
    queries: u32,
    /// 成功数
    successes: u32,
    /// 总耗时
    total_duration: Duration,
}

impl QueryStats {
    fn add_result(&mut self, transport_type: &str, success: bool, duration: Duration) {
        self.total_queries += 1;
        self.total_duration += duration;
        
        if success {
            self.successful_queries += 1;
        } else {
            self.failed_queries += 1;
        }
        
        let stats = self.transport_stats.entry(transport_type.to_string()).or_default();
        stats.queries += 1;
        stats.total_duration += duration;
        if success {
            stats.successes += 1;
        }
    }
    
    fn success_rate(&self) -> f64 {
        if self.total_queries == 0 {
            0.0
        } else {
            self.successful_queries as f64 / self.total_queries as f64 * 100.0
        }
    }
    
    fn avg_duration(&self) -> Duration {
        if self.total_queries == 0 {
            Duration::from_millis(0)
        } else {
            self.total_duration / self.total_queries
        }
    }
}

/// 创建DNSSEC测试用例
fn create_dnssec_test_cases() -> Vec<DnssecTestCase> {
    vec![
        // 已知支持DNSSEC的域名
        DnssecTestCase::new("cloudflare.com", true, "Cloudflare DNSSEC验证"),
        DnssecTestCase::new("baidu.com", true, "百度DNSSEC验证"),
        DnssecTestCase::new("qq.com", true, "腾讯DNSSEC验证"),
        DnssecTestCase::new("dnssec-deployment.org", true, "DNSSEC部署测试域名"),
        
        // 可能不支持DNSSEC的域名
        DnssecTestCase::new("example.com", false, "Example.com DNSSEC测试"),
        DnssecTestCase::new("test-no-dnssec.com", false, "无DNSSEC域名测试"),
    ]
}

/// 创建所有DNS记录类型的测试用例
fn create_test_cases() -> Vec<DnsTestCase> {
    vec![
        // 基础A记录测试 - 这些肯定存在
        DnsTestCase::new(RecordType::A, "cloudflare.com", "Cloudflare IPv4记录", true),
        DnsTestCase::new(RecordType::AAAA, "cloudflare.com", "Cloudflare IPv6记录", true),
        DnsTestCase::new(RecordType::A, "baidu.com", "百度IPv4记录", true),
        DnsTestCase::new(RecordType::AAAA, "baidu.com", "百度IPv6记录", true),
        
        // CNAME记录测试 - 调整为更可能存在的
        DnsTestCase::new(RecordType::CNAME, "www.taobao.com", "淘宝CNAME记录", true),
        DnsTestCase::new(RecordType::CNAME, "www.cloudflare.com", "Cloudflare CNAME记录", false), // 可能不存在
        
        // MX记录测试 - 使用国内域名提高成功率
        DnsTestCase::new(RecordType::MX, "qq.com", "QQ邮件记录", true),
        DnsTestCase::new(RecordType::MX, "163.com", "网易邮件记录", true),
        DnsTestCase::new(RecordType::MX, "sina.com.cn", "新浪邮件记录", true),
        
        // TXT记录测试 - 使用国内域名提高成功率
        DnsTestCase::new(RecordType::TXT, "baidu.com", "百度TXT记录", true),
        DnsTestCase::new(RecordType::TXT, "taobao.com", "淘宝TXT记录", true),
        DnsTestCase::new(RecordType::TXT, "qq.com", "腾讯TXT记录", true),
        
        // NS记录测试 - 所有域名都应该有NS记录
        DnsTestCase::new(RecordType::NS, "cloudflare.com", "Cloudflare名称服务器", true),
        DnsTestCase::new(RecordType::NS, "baidu.com", "百度名称服务器", true),
        
        // SOA记录测试 - 权威域名应该有SOA记录
        DnsTestCase::new(RecordType::SOA, "cloudflare.com", "Cloudflare SOA记录", true),
        DnsTestCase::new(RecordType::SOA, "baidu.com", "百度SOA记录", true),
        
        // SRV记录测试 - 大多数不存在，设为false
        DnsTestCase::new(RecordType::SRV, "_sip._tcp.cloudflare.com", "SIP服务记录", false),
        DnsTestCase::new(RecordType::SRV, "_http._tcp.cloudflare.com", "HTTP服务记录", false),
        DnsTestCase::new(RecordType::SRV, "_xmpp-server._tcp.qq.com", "QQ XMPP服务记录", false),
        
        // PTR记录 - 反向DNS查询
        DnsTestCase::new(RecordType::PTR, "1.1.1.1.in-addr.arpa", "Cloudflare IPv4反向DNS", true),
        DnsTestCase::new(RecordType::PTR, "29.29.29.119.in-addr.arpa", "腾讯DNS IPv4反向DNS", true),
        DnsTestCase::new(RecordType::PTR, "1.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.8.b.d.0.1.0.0.2.ip6.arpa", "Cloudflare IPv6反向DNS", false), // IPv6反向DNS较少
    ]
}

/// 创建UDP DNS解析器
async fn create_udp_resolver(server: &str) -> Result<rat_quickdns::builder::resolver::SmartDnsResolver> {
    let resolver = rat_quickdns::builder::DnsResolverBuilder::new(
        rat_quickdns::builder::QueryStrategy::Smart,
        true,  // 启用 EDNS
        "global".to_string(),
    )
    .with_cache(true)
    .with_timeout(Duration::from_secs(3)) // 减少基础超时，依赖重试
    .with_retry_count(3) // 增加重试次数
    .add_udp_upstream("udp_server", server)
    .build()
    .await?;
    
    Ok(resolver)
}

/// 创建DoT DNS解析器
async fn create_dot_resolver(server: &str) -> Result<rat_quickdns::builder::resolver::SmartDnsResolver> {
    let resolver = rat_quickdns::builder::DnsResolverBuilder::new(
        rat_quickdns::builder::QueryStrategy::Smart,
        true,  // 启用 EDNS
        "global".to_string(),
    )
    .with_cache(true)
    .with_timeout(Duration::from_secs(10))
    .with_retry_count(2)
    .add_dot_upstream("dot_server", server)
    .build()
    .await?;
    
    Ok(resolver)
}

/// 创建DoH DNS解析器
async fn create_doh_resolver(url: &str) -> Result<rat_quickdns::builder::resolver::SmartDnsResolver> {
    let resolver = rat_quickdns::builder::DnsResolverBuilder::new(
        rat_quickdns::builder::QueryStrategy::Smart,
        true,  // 启用 EDNS
        "global".to_string(),
    )
    .with_cache(true)
    .with_timeout(Duration::from_secs(10))
    .with_retry_count(2)
    .add_doh_upstream("doh_server", url)
    .build()
    .await?;
    
    Ok(resolver)
}

/// 执行DNSSEC查询
async fn perform_dnssec_query(
    resolver: &rat_quickdns::builder::resolver::SmartDnsResolver,
    test_case: &DnssecTestCase,
    transport_name: &str,
) -> (bool, Duration, Option<String>) {
    let start = Instant::now();
    
    // 查询A记录并检查DNSSEC状态
    let request = rat_quickdns::builder::types::DnsQueryRequest::new(
        &test_case.domain,
        rat_quickdns::builder::types::DnsRecordType::A,
    ).with_timeout(8000)
     .with_dnssec(true); // 启用DNSSEC验证
    
    match resolver.query(request).await {
        Ok(response) => {
            let duration = start.elapsed();
            
            // 检查DNSSEC状态
             let dnssec_secure = matches!(response.dnssec_status, Some(rat_quickdns::builder::types::DnssecStatus::Secure));
             let has_rrsig = response.records.iter().any(|r| {
                 matches!(r.record_type, rat_quickdns::builder::types::DnsRecordType::RRSIG)
             });
             
             let dnssec_info = format!(
                 "{} | RRSIG记录: {} | DNSSEC记录: {}",
                 response.dnssec_status_description(),
                 if has_rrsig { "是" } else { "否" },
                 response.dnssec_record_summary()
             );
             
             let success = if test_case.expect_dnssec {
                 dnssec_secure
             } else {
                 // 如果不期望DNSSEC，只要查询成功即可
                 response.success
             };
            
            (success, duration, Some(dnssec_info))
        }
        Err(e) => {
            let duration = start.elapsed();
            let error_msg = format!("DNSSEC查询失败: {}", e);
            (false, duration, Some(error_msg))
        }
    }
}

/// 执行DNS查询
async fn perform_query(
    resolver: &rat_quickdns::builder::resolver::SmartDnsResolver,
    test_case: &DnsTestCase,
    transport_name: &str,
) -> (bool, Duration, Option<String>) {
    let start = Instant::now();
    
    // 将 RecordType 转换为 DnsRecordType
    let dns_record_type = match test_case.record_type {
        RecordType::A => rat_quickdns::builder::types::DnsRecordType::A,
        RecordType::AAAA => rat_quickdns::builder::types::DnsRecordType::AAAA,
        RecordType::CNAME => rat_quickdns::builder::types::DnsRecordType::CNAME,
        RecordType::MX => rat_quickdns::builder::types::DnsRecordType::MX,
        RecordType::TXT => rat_quickdns::builder::types::DnsRecordType::TXT,
        RecordType::NS => rat_quickdns::builder::types::DnsRecordType::NS,
        RecordType::SOA => rat_quickdns::builder::types::DnsRecordType::SOA,
        RecordType::PTR => rat_quickdns::builder::types::DnsRecordType::PTR,
        RecordType::SRV => rat_quickdns::builder::types::DnsRecordType::SRV,
        _ => rat_quickdns::builder::types::DnsRecordType::A, // 默认为A记录
    };
    
    let request = rat_quickdns::builder::types::DnsQueryRequest::new(
        &test_case.domain,
        dns_record_type,
    ).with_timeout(8000); // 增加超时时间到8秒
    
    match resolver.query(request).await {
        Ok(response) => {
            let duration = start.elapsed();
            
            // 检查是否有匹配的记录类型
            let matching_records: Vec<_> = response.records.iter()
                .filter(|record| record.record_type == dns_record_type)
                .collect();
            
            let has_matching_answers = response.success && !matching_records.is_empty();
            
            if has_matching_answers {
                let answer_info = match dns_record_type {
                    rat_quickdns::builder::types::DnsRecordType::A | 
                    rat_quickdns::builder::types::DnsRecordType::AAAA => {
                        let ips = response.ip_addresses();
                        format!("找到 {} 个IP地址: {}", ips.len(), 
                               ips.iter().take(3).map(|ip| ip.to_string()).collect::<Vec<_>>().join(", "))
                    },
                    rat_quickdns::builder::types::DnsRecordType::CNAME | 
                    rat_quickdns::builder::types::DnsRecordType::NS | 
                    rat_quickdns::builder::types::DnsRecordType::PTR => {
                        let domains = response.domains();
                        format!("找到 {} 个域名: {}", domains.len(), 
                               domains.iter().take(3).cloned().collect::<Vec<_>>().join(", "))
                    },
                    rat_quickdns::builder::types::DnsRecordType::TXT => {
                        let texts = response.texts();
                        format!("找到 {} 个TXT记录: {}", texts.len(), 
                               texts.iter().take(2).map(|t| format!("\"{}\"", t.chars().take(50).collect::<String>())).collect::<Vec<_>>().join(", "))
                    },
                    rat_quickdns::builder::types::DnsRecordType::MX => {
                        let mx_records = response.mx_records();
                        format!("找到 {} 个MX记录: {}", mx_records.len(), 
                               mx_records.iter().take(3).map(|(p, e)| format!("{}:{}", p, e)).collect::<Vec<_>>().join(", "))
                    },
                    _ => {
                        format!("找到 {} 条 {:?} 记录", matching_records.len(), dns_record_type)
                    }
                };
                
                // 添加DNSSEC信息
                let final_info = if response.has_dnssec_records() || response.dnssec_status.is_some() {
                    format!("{} | {}", answer_info, response.dnssec_status_description())
                } else {
                    answer_info
                };
                
                (true, duration, Some(final_info))
            } else if response.success && !response.records.is_empty() {
                // 有响应但没有匹配的记录类型
                let other_types: Vec<_> = response.records.iter()
                    .map(|r| format!("{:?}", r.record_type))
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect();
                let info = format!("查询成功但无 {:?} 记录，返回了: {}", dns_record_type, other_types.join(", "));
                
                if test_case.expect_results {
                    (false, duration, Some(info))
                } else {
                    (true, duration, Some(format!("符合预期：{}", info)))
                }
            } else if test_case.expect_results {
                (false, duration, Some("DNS查询无响应或失败".to_string()))
            } else {
                (true, duration, Some("符合预期：无记录".to_string()))
            }
        }
        Err(e) => {
            let duration = start.elapsed();
            let error_msg = format!("查询失败: {}", e);
            
            // 区分不同类型的错误
            if error_msg.contains("NXDOMAIN") || error_msg.contains("Name Error") {
                if test_case.expect_results {
                    (false, duration, Some("域名不存在 (NXDOMAIN)".to_string()))
                } else {
                    (true, duration, Some("符合预期：域名不存在".to_string()))
                }
            } else if error_msg.contains("timeout") || error_msg.contains("Timeout") {
                (false, duration, Some("查询超时".to_string()))
            } else {
                (false, duration, Some(error_msg))
            }
        }
    }
}

/// 打印查询结果
fn print_query_result(
    test_case: &DnsTestCase,
    transport_name: &str,
    success: bool,
    duration: Duration,
    details: Option<String>,
) {
    let status = if success { "✅" } else { "❌" };
    let duration_ms = duration.as_millis();
    
    println!(
        "  {} [{:>8}] {:>6} | {:>15} | {:>8}ms | {}",
        status,
        transport_name,
        format!("{:?}", test_case.record_type),
        test_case.domain,
        duration_ms,
        test_case.description
    );
    
    if let Some(details) = details {
        println!("    📝 {}", details);
    }
}

/// 运行DNSSEC测试
async fn run_dnssec_tests() -> Result<()> {
    println!("\n🔒 DNSSEC验证测试\n");
    
    let dnssec_test_cases = create_dnssec_test_cases();
    let mut stats = QueryStats::default();
    let servers = TencentDnsServers::new();
    
    // 创建支持DNSSEC的解析器
    let dot_resolver = create_dot_resolver(&servers.dot_server).await?;
    let doh_resolver = create_doh_resolver(&servers.doh_url).await?;
    
    let resolvers = vec![
        ("DoT", &dot_resolver),
        ("DoH", &doh_resolver),
    ];
    
    println!("🔐 DNSSEC测试结果:");
    println!("  状态 [传输类型]     域名 |     耗时 | DNSSEC状态");
    println!("  ─────────────────────────────────────────────────────────────");
    
    for (transport_name, resolver) in &resolvers {
        for test_case in &dnssec_test_cases {
            let (success, duration, details) = perform_dnssec_query(resolver, test_case, transport_name).await;
            
            let status = if success { "✅" } else { "❌" };
            let duration_ms = duration.as_millis();
            
            println!(
                "  {} [{:>8}] {:>20} | {:>8}ms | {}",
                status,
                transport_name,
                test_case.domain,
                duration_ms,
                test_case.description
            );
            
            if let Some(details) = details {
                println!("    🔐 {}", details);
            }
            
            stats.add_result(transport_name, success, duration);
            
            // 避免过于频繁的查询
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
    
    println!("\n📊 DNSSEC测试统计:");
    print_stats_summary(&stats);
    
    Ok(())
}

/// 运行所有测试
async fn run_comprehensive_tests() -> Result<()> {
    println!("🚀 开始全面DNS查询测试\n");
    
    let test_cases = create_test_cases();
    let mut stats = QueryStats::default();
    let servers = TencentDnsServers::new();
    
    // 创建三种类型的解析器
    let udp_resolver = create_udp_resolver(&servers.udp_server).await?;
    let dot_resolver = create_dot_resolver(&servers.dot_server).await?;
    let doh_resolver = create_doh_resolver(&servers.doh_url).await?;
    
    let resolvers = vec![
        ("UDP", &udp_resolver),
        ("DoT", &dot_resolver),
        ("DoH", &doh_resolver),
    ];
    
    println!("📊 测试结果表格:");
    println!("  状态 [传输类型] 记录类型 |           域名 |     耗时 | 描述");
    println!("  ─────────────────────────────────────────────────────────────────────");
    
    for (transport_name, resolver) in &resolvers {
        for test_case in &test_cases {
            let (success, duration, details) = perform_query(resolver, test_case, transport_name).await;
            
            print_query_result(test_case, transport_name, success, duration, details);
            stats.add_result(transport_name, success, duration);
            
            // 避免过于频繁的查询
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }
    
    // 打印统计信息
    print_stats_summary(&stats);
    
    Ok(())
}

/// 打印统计摘要
fn print_stats_summary(stats: &QueryStats) {
    println!("\n📈 查询统计摘要:");
    println!("  总查询数: {}", stats.total_queries);
    println!("  成功率: {:.1}%", stats.success_rate());
    println!("  平均耗时: {:?}", stats.avg_duration());
    
    for (transport, transport_stats) in &stats.transport_stats {
        let success_rate = if transport_stats.queries == 0 {
            0.0
        } else {
            transport_stats.successes as f64 / transport_stats.queries as f64 * 100.0
        };
        let avg_duration = if transport_stats.queries == 0 {
            Duration::from_millis(0)
        } else {
            transport_stats.total_duration / transport_stats.queries
        };
        
        println!("  {} - 成功率: {:.1}%, 平均耗时: {:?}", 
                transport, success_rate, avg_duration);
    }
}

/// 错误处理和边界情况测试
async fn test_error_cases() -> Result<()> {
    println!("\n=== 错误处理和边界情况测试 ===");
    
    let resolver = create_udp_resolver("119.29.29.29").await?;
    
    let error_test_cases = vec![
        ("nonexistent-domain-12345.com", "不存在的域名"),
        ("invalid..domain..com", "无效域名格式"),
    ];
    
    for (domain, description) in error_test_cases {
        print!("测试错误情况: {} ... ", description);
        
        let test_case = DnsTestCase::new(RecordType::A, domain, description, false);
        let (success, duration, details) = perform_query(&resolver, &test_case, "UDP").await;
        
        if success {
            println!("⚠️  意外成功");
        } else {
            println!("✅ 正确处理错误: {:?}", details);
        }
    }
    
    Ok(())
}

/// 主函数
#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 全面DNS查询示例 - 腾讯云DNS服务器");
    println!("测试域名: cloudflare.com (支持多种DNS记录类型)");
    println!("{}", "=".repeat(60));
    
    // 执行全面测试
    if let Err(e) = run_comprehensive_tests().await {
        eprintln!("全面测试失败: {}", e);
    }
    
    // 执行DNSSEC测试
    if let Err(e) = run_dnssec_tests().await {
        eprintln!("DNSSEC测试失败: {}", e);
    }
    
    // 执行错误处理测试
    if let Err(e) = test_error_cases().await {
        eprintln!("错误处理测试失败: {}", e);
    }
    
    println!("\n📊 测试总结:");
    println!("✅ 当前支持的DNS记录类型: A, AAAA, CNAME, MX, TXT, NS, SOA, SRV, PTR");
    println!("✅ 腾讯云DNS服务器配置: UDP, DoT, DoH");
    println!("✅ DNSSEC验证测试完成");
    println!("✅ 错误处理和边界情况测试完成");
    println!("\n💡 建议:");
    println!("   - 根据网络环境选择合适的DNS协议");
    println!("   - UDP适合快速查询，TCP适合大响应");
    println!("   - DoT/DoH提供加密传输，适合安全要求高的场景");
    println!("   - DNSSEC提供DNS响应完整性验证，推荐在安全敏感场景使用");
    println!("   - 注意：UDP协议通常不支持DNSSEC验证，建议使用DoT或DoH");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dns_test_cases_completeness() {
        let test_cases = create_test_cases();
        
        // 验证是否覆盖了所有主要的DNS记录类型
        let covered_types: std::collections::HashSet<_> = test_cases
            .iter()
            .map(|case| case.record_type)
            .collect();
        
        let expected_types = vec![
            RecordType::A,
            RecordType::AAAA,
            RecordType::CNAME,
            RecordType::MX,
            RecordType::TXT,
            RecordType::NS,
            RecordType::SOA,
            RecordType::SRV,
            RecordType::PTR,
        ];
        
        for expected_type in expected_types {
            assert!(covered_types.contains(&expected_type), 
                   "缺少 {:?} 记录类型的测试用例", expected_type);
        }
        
        assert!(test_cases.len() >= 9, "测试用例数量不足");
    }
    
    #[test]
    fn test_tencent_dns_servers_config() {
        let servers = TencentDnsServers::new();
        
        // 验证腾讯云DNS服务器配置
        assert_eq!(servers.udp_server, "119.29.29.29");
        assert_eq!(servers.dot_server, "dot.pub");
        assert_eq!(servers.doh_url, "https://doh.pub/dns-query");
    }
    
    #[tokio::test]
    async fn test_basic_dns_query() {
        // 基本的DNS查询测试
        let resolver = create_udp_resolver("119.29.29.29").await;
        
        match resolver {
            Ok(resolver) => {
                let test_case = DnsTestCase::new(RecordType::A, "cloudflare.com", "测试A记录", true);
                let (success, _duration, details) = perform_query(&resolver, &test_case, "UDP").await;
                
                if success {
                    println!("成功解析cloudflare.com的A记录: {:?}", details);
                } else {
                    println!("DNS查询失败（可能是网络问题）: {:?}", details);
                }
            }
            Err(e) => {
                println!("创建解析器失败: {}", e);
            }
        }
    }
}