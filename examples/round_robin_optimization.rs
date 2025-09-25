//! ROUND_ROBIN策略性能优化示例
//!
//! 本示例展示如何使用优化后的ROUND_ROBIN策略进行高性能DNS查询。
//! 包括：
//! 1. 基础ROUND_ROBIN配置
//! 2. 性能优化配置
//! 3. 批量查询测试
//! 4. 性能对比分析

use std::time::{Duration, Instant};
use tokio::time::sleep;
use rat_quickdns::{
    builder::{
        DnsResolverBuilder,
        strategy::QueryStrategy,
        types::{DnsQueryRequest, DnsRecordType},
        resolver::SmartDnsResolver,
    },
    error::Result,
};

/// 性能测试统计信息
#[derive(Debug, Clone)]
struct BenchmarkStats {
    name: String,
    total_queries: usize,
    success_count: usize,
    failure_count: usize,
    total_time: Duration,
    latencies: Vec<Duration>,
}

impl BenchmarkStats {
    fn new(name: String) -> Self {
        Self {
            name,
            total_queries: 0,
            success_count: 0,
            failure_count: 0,
            total_time: Duration::ZERO,
            latencies: Vec::new(),
        }
    }
    
    fn success_rate(&self) -> f64 {
        if self.total_queries == 0 {
            0.0
        } else {
            self.success_count as f64 / self.total_queries as f64 * 100.0
        }
    }
    
    fn qps(&self) -> f64 {
        if self.total_time.as_secs_f64() == 0.0 {
            0.0
        } else {
            self.total_queries as f64 / self.total_time.as_secs_f64()
        }
    }
    
    fn avg_latency(&self) -> Duration {
        if self.latencies.is_empty() {
            Duration::ZERO
        } else {
            let total: Duration = self.latencies.iter().sum();
            total / self.latencies.len() as u32
        }
    }
    
    fn p95_latency(&self) -> Duration {
        if self.latencies.is_empty() {
            return Duration::ZERO;
        }
        
        let mut sorted = self.latencies.clone();
        sorted.sort();
        let index = (sorted.len() as f64 * 0.95) as usize;
        sorted.get(index.min(sorted.len() - 1)).copied().unwrap_or(Duration::ZERO)
    }
    
    fn min_latency(&self) -> Duration {
        self.latencies.iter().min().copied().unwrap_or(Duration::ZERO)
    }
    
    fn max_latency(&self) -> Duration {
        self.latencies.iter().max().copied().unwrap_or(Duration::ZERO)
    }
}

/// ROUND_ROBIN策略优化演示
struct RoundRobinOptimizationDemo {
    test_domains: Vec<&'static str>,
}

impl RoundRobinOptimizationDemo {
    fn new() -> Self {
        Self {
            test_domains: vec![
                "google.com",
                "github.com",
                "stackoverflow.com",
                "microsoft.com",
                "amazon.com",
                "cloudflare.com",
                "baidu.com",
                "taobao.com",
                "qq.com",
                "weibo.com",
            ],
        }
    }
    
    /// 创建基础ROUND_ROBIN解析器
    async fn create_basic_resolver(&self) -> Result<SmartDnsResolver> {
        let resolver = DnsResolverBuilder::new(
            QueryStrategy::RoundRobin,
            true,
            "global".to_string(),
        )
            .query_strategy(QueryStrategy::RoundRobin)
            // 添加多个上游服务器
            .add_udp_upstream("阿里DNS", "223.5.5.5")
            .add_udp_upstream("腾讯DNS", "119.29.29.29")
            .add_udp_upstream("114DNS", "114.114.114.114")
            .add_udp_upstream("Google DNS", "8.8.8.8")
            .add_udp_upstream("Cloudflare DNS", "1.1.1.1")
            // 基础配置
            .with_timeout(Duration::from_secs(5)) // 默认5秒超时
            .with_upstream_monitoring(true)
            .build()
            .await?;
        
        Ok(resolver)
    }
    
    /// 创建优化的ROUND_ROBIN解析器
    async fn create_optimized_resolver(&self) -> Result<SmartDnsResolver> {
        let resolver = DnsResolverBuilder::new(
            QueryStrategy::RoundRobin,
            true,
            "global".to_string(),
        )
            .query_strategy(QueryStrategy::RoundRobin)
            // 添加多个上游服务器
            .add_udp_upstream("阿里DNS", "223.5.5.5")
            .add_udp_upstream("腾讯DNS", "119.29.29.29")
            .add_udp_upstream("114DNS", "114.114.114.114")
            .add_udp_upstream("Google DNS", "8.8.8.8")
            .add_udp_upstream("Cloudflare DNS", "1.1.1.1")
            // 应用ROUND_ROBIN优化
            .optimize_for_round_robin()
            // 或者手动配置优化参数
            // .with_round_robin_timeout(Duration::from_millis(1500)) // 1.5秒超时
            // .with_health_check(true)
            // .with_retry_count(1) // 减少重试次数
            // .with_concurrent_queries(4) // 增加并发数
            .build()
            .await?;
        
        Ok(resolver)
    }
    
    /// 对解析器进行性能测试
    async fn benchmark_resolver(
        &self,
        resolver: &SmartDnsResolver,
        name: String,
        iterations: usize,
    ) -> BenchmarkStats {
        println!("\n🚀 开始测试 {} (共{}次查询)...", name, iterations);
        
        let mut stats = BenchmarkStats::new(name);
        let start_time = Instant::now();
        
        for i in 0..iterations {
            let domain = self.test_domains[i % self.test_domains.len()];
            
            let request = DnsQueryRequest {
                query_id: Some(format!("test-{}", i)),
                domain: domain.to_string(),
                record_type: DnsRecordType::A,
                enable_edns: true,
                client_address: None,
                timeout_ms: None,
                disable_cache: false,
                enable_dnssec: false,
            };
            
            let query_start = Instant::now();
            match resolver.query(request).await {
                Ok(response) => {
                    let query_duration = query_start.elapsed();
                    
                    if response.success {
                        stats.success_count += 1;
                        stats.latencies.push(query_duration);
                        
                        if i % 10 == 0 {
                            println!("  ✅ {}: {:.1}ms", domain, query_duration.as_secs_f64() * 1000.0);
                        }
                    } else {
                        stats.failure_count += 1;
                        println!("  ❌ {}: {:?}", domain, response.error);
                    }
                },
                Err(e) => {
                    stats.failure_count += 1;
                    println!("  💥 {}: {}", domain, e);
                }
            }
            
            stats.total_queries += 1;
            
            // 短暂延迟避免过于频繁的查询
            if i % 10 == 0 {
                sleep(Duration::from_millis(10)).await;
            }
        }
        
        stats.total_time = start_time.elapsed();
        stats
    }
    
    /// 打印性能对比结果
    fn print_comparison(&self, basic_stats: &BenchmarkStats, optimized_stats: &BenchmarkStats) {
        println!("\n{}", "=".repeat(80));
        println!("📊 ROUND_ROBIN策略性能对比报告");
        println!("{}", "=".repeat(80));
        
        println!("\n🔍 基础配置 vs 优化配置:");
        println!("{:<20} {:<15} {:<15} {:<15}", "指标", "基础配置", "优化配置", "改进");
        println!("{}", "-".repeat(70));
        
        // 成功率对比
        let success_improvement = optimized_stats.success_rate() - basic_stats.success_rate();
        println!(
            "{:<20} {:<14.1}% {:<14.1}% {:+.1}%",
            "成功率",
            basic_stats.success_rate(),
            optimized_stats.success_rate(),
            success_improvement
        );
        
        // QPS对比
        let qps_improvement = if basic_stats.qps() > 0.0 {
            (optimized_stats.qps() - basic_stats.qps()) / basic_stats.qps() * 100.0
        } else {
            0.0
        };
        println!(
            "{:<20} {:<14.1} {:<14.1} {:+.1}%",
            "QPS",
            basic_stats.qps(),
            optimized_stats.qps(),
            qps_improvement
        );
        
        // 平均延迟对比
        let basic_avg_ms = basic_stats.avg_latency().as_secs_f64() * 1000.0;
        let optimized_avg_ms = optimized_stats.avg_latency().as_secs_f64() * 1000.0;
        let latency_improvement = if basic_avg_ms > 0.0 {
            (basic_avg_ms - optimized_avg_ms) / basic_avg_ms * 100.0
        } else {
            0.0
        };
        println!(
            "{:<20} {:<14.1} {:<14.1} {:+.1}%",
            "平均延迟(ms)",
            basic_avg_ms,
            optimized_avg_ms,
            latency_improvement
        );
        
        // P95延迟对比
        let basic_p95_ms = basic_stats.p95_latency().as_secs_f64() * 1000.0;
        let optimized_p95_ms = optimized_stats.p95_latency().as_secs_f64() * 1000.0;
        let p95_improvement = if basic_p95_ms > 0.0 {
            (basic_p95_ms - optimized_p95_ms) / basic_p95_ms * 100.0
        } else {
            0.0
        };
        println!(
            "{:<20} {:<14.1} {:<14.1} {:+.1}%",
            "P95延迟(ms)",
            basic_p95_ms,
            optimized_p95_ms,
            p95_improvement
        );
        
        println!("\n💡 优化效果总结:");
        if qps_improvement > 0.0 {
            println!("  ✅ QPS提升 {:.1}%", qps_improvement);
        }
        if latency_improvement > 0.0 {
            println!("  ✅ 平均延迟降低 {:.1}%", latency_improvement);
        }
        if success_improvement > 0.0 {
            println!("  ✅ 成功率提升 {:.1}%", success_improvement);
        }
        
        println!("\n🎯 优化建议:");
        println!("  1. 使用 optimize_for_round_robin() 应用所有优化");
        println!("  2. 根据网络环境调整 with_round_robin_timeout()");
        println!("  3. 启用健康检查避免选择不可用服务器");
        println!("  4. 增加并发查询数量提高吞吐量");
        
        println!("\n📈 详细统计:");
        self.print_detailed_stats("基础配置", basic_stats);
        self.print_detailed_stats("优化配置", optimized_stats);
    }
    
    /// 打印详细统计信息
    fn print_detailed_stats(&self, name: &str, stats: &BenchmarkStats) {
        println!("\n  📊 {}:", name);
        println!("    总查询数: {}", stats.total_queries);
        println!("    成功数: {}", stats.success_count);
        println!("    失败数: {}", stats.failure_count);
        println!("    总耗时: {:.2}s", stats.total_time.as_secs_f64());
        println!("    最小延迟: {:.1}ms", stats.min_latency().as_secs_f64() * 1000.0);
        println!("    最大延迟: {:.1}ms", stats.max_latency().as_secs_f64() * 1000.0);
    }
    
    /// 运行完整的演示
    async fn run_demo(&self) -> Result<()> {
        println!("🔧 ROUND_ROBIN策略性能优化演示");
        println!("{}", "=".repeat(50));
        
        // 创建解析器
        println!("\n📦 创建解析器实例...");
        let basic_resolver = self.create_basic_resolver().await?;
        let optimized_resolver = self.create_optimized_resolver().await?;
        
        // 性能测试
        let iterations = 100;
        let basic_stats = self.benchmark_resolver(&basic_resolver, "基础ROUND_ROBIN".to_string(), iterations).await;
        let optimized_stats = self.benchmark_resolver(&optimized_resolver, "优化ROUND_ROBIN".to_string(), iterations).await;
        
        // 打印对比结果
        self.print_comparison(&basic_stats, &optimized_stats);
        
        println!("\n✨ 演示完成！");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    env_logger::init();
    
    let demo = RoundRobinOptimizationDemo::new();
    
    match demo.run_demo().await {
        Ok(_) => println!("\n🎉 演示成功完成！"),
        Err(e) => {
            eprintln!("\n❌ 演示过程中发生错误: {}", e);
            std::process::exit(1);
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_basic_resolver_creation() {
        let demo = RoundRobinOptimizationDemo::new();
        let resolver = demo.create_basic_resolver().await;
        assert!(resolver.is_ok());
    }
    
    #[tokio::test]
    async fn test_optimized_resolver_creation() {
        let demo = RoundRobinOptimizationDemo::new();
        let resolver = demo.create_optimized_resolver().await;
        assert!(resolver.is_ok());
    }
    
    #[test]
    fn test_benchmark_stats() {
        let mut stats = BenchmarkStats::new("test".to_string());
        stats.total_queries = 100;
        stats.success_count = 95;
        stats.failure_count = 5;
        stats.total_time = Duration::from_secs(10);
        
        assert_eq!(stats.success_rate(), 95.0);
        assert_eq!(stats.qps(), 10.0);
    }
}