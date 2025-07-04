//! DNS 日志系统使用示例
//! 
//! 展示如何使用基于 zerg_creep 的高性能 DNS 日志系统

use rat_quickdns::logger::{init_dns_logger, info, debug, error, warn, trace};
use rat_quickdns::{dns_query, dns_response, dns_error, dns_timeout, dns_cache_hit, dns_cache_miss, dns_upstream, dns_strategy};
use zerg_creep::logger::LevelFilter;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化 DNS 日志系统
    init_dns_logger(LevelFilter::Trace)?;
    
    info!("🚀 DNS 查询器日志系统启动");
    
    // 模拟 DNS 查询流程
    simulate_dns_queries().await;
    
    // 模拟错误场景
    simulate_error_scenarios().await;
    
    // 模拟缓存操作
    simulate_cache_operations().await;
    
    // 模拟负载均衡策略
    simulate_load_balancing().await;
    
    info!("✅ DNS 查询器日志系统演示完成");
    
    Ok(())
}

/// 模拟正常的 DNS 查询流程
async fn simulate_dns_queries() {
    info!("📋 开始模拟 DNS 查询流程");
    
    // 查询不同类型的记录
    dns_query!("example.com", "A");
    sleep(Duration::from_millis(50)).await;
    dns_response!("example.com", 2, 45);
    
    dns_query!("mail.example.com", "MX");
    sleep(Duration::from_millis(30)).await;
    dns_response!("mail.example.com", 1, 28);
    
    dns_query!("_sip._tcp.example.com", "SRV");
    sleep(Duration::from_millis(75)).await;
    dns_response!("_sip._tcp.example.com", 3, 72);
    
    dns_query!("2001:db8::1", "PTR");
    sleep(Duration::from_millis(40)).await;
    dns_response!("2001:db8::1", 1, 38);
}

/// 模拟错误场景
async fn simulate_error_scenarios() {
    warn!("⚠️  开始模拟错误场景");
    
    // 域名不存在
    dns_query!("nonexistent.invalid", "A");
    sleep(Duration::from_millis(100)).await;
    dns_error!("nonexistent.invalid", "NXDOMAIN");
    
    // 查询超时
    dns_query!("slow.example.com", "AAAA");
    sleep(Duration::from_millis(5000)).await;
    dns_timeout!("slow.example.com", 5000);
    
    // 服务器错误
    dns_query!("broken.example.com", "TXT");
    sleep(Duration::from_millis(20)).await;
    dns_error!("broken.example.com", "SERVFAIL");
    
    // 网络错误
    error!("🌐 网络连接失败: 无法连接到上游 DNS 服务器 8.8.8.8:53");
}

/// 模拟缓存操作
async fn simulate_cache_operations() {
    debug!("💾 开始模拟缓存操作");
    
    // 缓存未命中，需要查询
    dns_cache_miss!("fresh.example.com");
    dns_query!("fresh.example.com", "A");
    sleep(Duration::from_millis(35)).await;
    dns_response!("fresh.example.com", 1, 32);
    
    // 缓存命中，直接返回
    dns_cache_hit!("cached.example.com");
    trace!("📊 缓存统计: 命中率 85.2%, 总查询 1247 次");
    
    // 缓存过期，重新查询
    debug!("⏰ 缓存条目过期: popular.example.com (TTL: 300s)");
    dns_cache_miss!("popular.example.com");
    dns_query!("popular.example.com", "A");
    sleep(Duration::from_millis(25)).await;
    dns_response!("popular.example.com", 4, 23);
}

/// 模拟负载均衡策略
async fn simulate_load_balancing() {
    debug!("⚖️  开始模拟负载均衡策略");
    
    // 轮询策略
    dns_strategy!("RoundRobin", "balanced.example.com");
    dns_upstream!("8.8.8.8:53", "balanced.example.com");
    sleep(Duration::from_millis(40)).await;
    dns_response!("balanced.example.com", 2, 38);
    
    // 最快响应策略
    dns_strategy!("FastestResponse", "speed.example.com");
    dns_upstream!("1.1.1.1:53", "speed.example.com");
    sleep(Duration::from_millis(25)).await;
    dns_response!("speed.example.com", 1, 22);
    
    // 健康检查
    trace!("🏥 上游服务器健康检查:");
    trace!("  - 8.8.8.8:53 ✅ 延迟: 25ms");
    trace!("  - 8.8.4.4:53 ✅ 延迟: 30ms");
    trace!("  - 1.1.1.1:53 ✅ 延迟: 18ms");
    trace!("  - 208.67.222.222:53 ⚠️  延迟: 150ms");
    
    // 故障转移
    warn!("🔄 主服务器 8.8.8.8:53 响应超时，切换到备用服务器");
    dns_upstream!("1.1.1.1:53", "failover.example.com");
    sleep(Duration::from_millis(20)).await;
    dns_response!("failover.example.com", 1, 18);
    
    info!("📈 负载均衡统计:");
    info!("  - 总查询: 1,247 次");
    info!("  - 平均延迟: 32ms");
    info!("  - 成功率: 99.2%");
    info!("  - 缓存命中率: 85.2%");
}