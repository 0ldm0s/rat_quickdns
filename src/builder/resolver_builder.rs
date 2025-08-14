//! DNS解析器构建器模块
//! 
//! 本模块提供了构建DNS解析器的Builder模式实现

use std::sync::Arc;
use std::time::Duration;

use rat_quick_threshold::memory::UnifiedAddressSpace;
use rat_quick_threshold::memory::get_global_address_space;

use crate::resolver::CoreResolverConfig;
use crate::upstream_handler::{UpstreamManager, UpstreamSpec};
use crate::error::{DnsError, Result};
use crate::dns_error;
use super::{
    strategy::QueryStrategy,
    engine::SmartDecisionEngine,
    resolver::SmartDnsResolver,
};

/// 日志初始化策略
#[derive(Debug, Clone, PartialEq)]
pub enum LoggerInitStrategy {
    /// 不初始化日志（让上层应用控制）
    None,
    /// 使用静默模式初始化
    Silent,
    /// 启用调试级别日志（显示所有调试信息）
    Debug,
    /// 根据配置的日志级别初始化
    Auto,
}

/// DNS解析器构建器
#[derive(Debug)]
pub struct DnsResolverBuilder {
    /// 解析器配置
    config: CoreResolverConfig,
    
    /// 上游管理器
    upstream_manager: UpstreamManager,
    
    // 使用全局内存池，不需要存储内存配置
    
    /// 查询策略
    query_strategy: QueryStrategy,
    
    /// 是否启用EDNS
    enable_edns: bool,
    
    /// 当前区域
    current_region: String,
    
    /// 日志初始化策略
    logger_init_strategy: LoggerInitStrategy,
}

// 注意：移除了 Default 实现，因为它包含兜底行为
// 硬编码的默认值（如 Smart策略、EDNS启用、QuickMem配置等）是兜底代码
// 用户必须明确指定所有配置项

// 手动实现Clone，避免直接克隆UnifiedAddressSpace
impl Clone for DnsResolverBuilder {
    fn clone(&self) -> Self {
        // 使用全局地址空间实例
        let memory_config = rat_quick_threshold::memory::get_global_address_space();
        
        Self {
            config: self.config.clone(),
            upstream_manager: self.upstream_manager.clone(),
            // 使用全局内存池，不需要存储内存配置
            query_strategy: self.query_strategy.clone(),
            enable_edns: self.enable_edns,
            current_region: self.current_region.clone(),
            logger_init_strategy: self.logger_init_strategy.clone(),
        }
    }
}

impl DnsResolverBuilder {
    /// 创建新的构造器（需要明确指定所有配置）
    pub fn new(
        query_strategy: QueryStrategy,
        enable_edns: bool,
        current_region: String,
    ) -> Self {
        // 创建一个基本的配置，用户需要进一步配置
        let config = CoreResolverConfig::new(
            query_strategy,
            std::time::Duration::from_secs(5), // 临时默认值，用户应该明确设置
            2, // 临时默认值，用户应该明确设置
            false, // 临时默认值，用户应该明确设置
            std::time::Duration::from_secs(300), // 临时默认值
            false, // 临时默认值，用户应该明确设置
            std::time::Duration::from_secs(30), // 临时默认值
            53, // 临时默认值，用户应该明确设置
            1, // 临时默认值，用户应该明确设置
            true, // 临时默认值
            4096, // 临时默认值
            false, // 临时默认值
            zerg_creep::logger::LevelFilter::Info, // 临时默认值
            false, // 临时默认值
        );
        
        Self {
            config,
            upstream_manager: UpstreamManager::new(),
            query_strategy,
            enable_edns,
            current_region,
            logger_init_strategy: LoggerInitStrategy::Auto, // 默认自动模式，保持向后兼容
        }
    }
    
    /// 设置查询策略
    pub fn query_strategy(mut self, strategy: QueryStrategy) -> Self {
        self.query_strategy = strategy;
        self
    }
    
    /// 启用/禁用EDNS
    pub fn enable_edns(mut self, enable: bool) -> Self {
        self.enable_edns = enable;
        self
    }
    
    /// 设置当前区域
    pub fn region(mut self, region: impl Into<String>) -> Self {
        self.current_region = region.into();
        self
    }
    
    /// 添加UDP上游服务器
    pub fn add_udp_upstream(mut self, name: impl Into<String>, server: impl Into<String>) -> Self {
        let spec = UpstreamSpec::udp(name.into(), server.into());
        let _ = self.upstream_manager.add_upstream(spec); // 忽略错误，在build时处理
        self
    }
    
    /// 添加TCP上游服务器
    pub fn add_tcp_upstream(mut self, name: impl Into<String>, server: impl Into<String>) -> Self {
        let spec = UpstreamSpec::tcp(name.into(), server.into());
        if let Err(e) = self.upstream_manager.add_upstream(spec) {
            dns_error!("Failed to add TCP upstream: {}", e);
        }
        self
    }
    
    /// 添加DoH上游服务器
    pub fn add_doh_upstream(mut self, name: impl Into<String>, url: impl Into<String>) -> Self {
        let spec = UpstreamSpec::doh(name.into(), url.into());
        if let Err(e) = self.upstream_manager.add_upstream(spec) {
            dns_error!("Failed to add DoH upstream: {}", e);
        }
        self
    }
    
    /// 添加DoT上游服务器
    pub fn add_dot_upstream(mut self, name: impl Into<String>, server: impl Into<String>) -> Self {
        let spec = UpstreamSpec::dot(name.into(), server.into());
        if let Err(e) = self.upstream_manager.add_upstream(spec) {
            dns_error!("Failed to add DoT upstream: {}", e);
        }
        self
    }
    
    /// 添加自定义上游配置
    pub fn add_upstream(mut self, spec: UpstreamSpec) -> Result<Self> {
        self.upstream_manager.add_upstream(spec)?;
        Ok(self)
    }
    
    /// 批量添加上游服务器
    pub fn add_upstreams(mut self, specs: Vec<UpstreamSpec>) -> Result<Self> {
        for spec in specs {
            self.upstream_manager.add_upstream(spec)?;
        }
        Ok(self)
    }
    
    /// 添加常用的公共DNS服务器
    pub fn with_public_dns(mut self) -> Result<Self> {
        // 国内DNS服务器
        self = self.add_udp_upstream("阿里DNS", "223.5.5.5");
        self = self.add_udp_upstream("腾讯DNS", "119.29.29.29");
        self = self.add_udp_upstream("114DNS", "114.114.114.114");
        
        // 国际DNS服务器
        self = self.add_udp_upstream("Google DNS", "8.8.8.8");
        self = self.add_udp_upstream("Cloudflare DNS", "1.1.1.1");
        
        // DoH服务器
        self = self.add_doh_upstream("阿里DoH", "https://dns.alidns.com/dns-query");
        self = self.add_doh_upstream("腾讯DoH", "https://doh.pub/dns-query");
        self = self.add_doh_upstream("Cloudflare DoH", "https://cloudflare-dns.com/dns-query");
        
        // DoT服务器
        self = self.add_dot_upstream("阿里DoT", "223.5.5.5");
        self = self.add_dot_upstream("腾讯DoT", "1.12.12.12");
        
        Ok(self)
    }
    
    /// 设置查询超时时间
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.config.default_timeout = timeout;
        self
    }
    
    /// 为ROUND_ROBIN策略设置优化的超时时间
    /// 轮询策略使用更短的超时时间以避免客户端等待过久
    pub fn with_round_robin_timeout(mut self, timeout: Duration) -> Self {
        if matches!(self.query_strategy, QueryStrategy::RoundRobin) {
            // 轮询策略使用更短的超时，最大不超过2秒
            self.config.default_timeout = timeout.min(Duration::from_secs(2));
        }
        self
    }
    
    /// 为ROUND_ROBIN策略应用性能优化配置
    /// 包括：降低超时时间、启用健康检查、增加并发数
    pub fn optimize_for_round_robin(mut self) -> Self {
        if matches!(self.query_strategy, QueryStrategy::RoundRobin) {
            self.config.default_timeout = Duration::from_millis(1500); // 1.5秒超时
            self.config.enable_upstream_monitoring = true; // 启用上游监控
            self.config.retry_count = 1; // 减少重试次数，快速失败
            self.config.concurrent_queries = self.config.concurrent_queries.max(4); // 至少4个并发
        }
        self
    }
    
    /// 设置重试次数
    pub fn with_retry_count(mut self, count: usize) -> Self {
        self.config.retry_count = count;
        self
    }
    
    /// 启用/禁用缓存
    pub fn with_cache(mut self, enable: bool) -> Self {
        self.config.enable_cache = enable;
        self
    }
    
    /// 设置缓存TTL上限
    pub fn with_cache_ttl(mut self, ttl: Duration) -> Self {
        self.config.max_cache_ttl = ttl;
        self
    }
    
    /// 启用/禁用上游监控
    pub fn with_upstream_monitoring(mut self, enable: bool) -> Self {
        self.config.enable_upstream_monitoring = enable;
        self
    }
    
    // 不再需要设置内存配置，使用全局内存池
    
    /// 设置DNS服务器端口
    pub fn with_port(mut self, port: u16) -> Self {
        self.config.port = port;
        self
    }
    
    /// 设置并发查询数量
    pub fn with_concurrent_queries(mut self, count: usize) -> Self {
        self.config.concurrent_queries = count;
        self
    }
    
    /// 启用/禁用递归查询
    pub fn with_recursion(mut self, enable: bool) -> Self {
        self.config.recursion_desired = enable;
        self
    }
    
    /// 设置查询缓冲区大小
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.config.buffer_size = size;
        self
    }
    
    /// 设置日志级别
    pub fn with_log_level(mut self, level: zerg_creep::logger::LevelFilter) -> Self {
        self.config.log_level = level;
        self
    }
    
    /// 启用/禁用DNS专用日志格式
    pub fn with_dns_log_format(mut self, enable: bool) -> Self {
        self.config.enable_dns_log_format = enable;
        self
    }
    
    /// 设置详细日志（Debug级别）
    pub fn with_verbose_logging(mut self) -> Self {
        self.config.log_level = zerg_creep::logger::LevelFilter::Debug;
        self.config.enable_dns_log_format = true;
        self
    }
    
    /// 设置静默日志（Error级别）
    pub fn with_quiet_logging(mut self) -> Self {
        self.config.log_level = zerg_creep::logger::LevelFilter::Error;
        self
    }
    
    /// 设置日志初始化策略
    /// 
    /// # 参数
    /// * `strategy` - 日志初始化策略
    ///   - `LoggerInitStrategy::None`: 不初始化日志，让上层应用完全控制
    ///   - `LoggerInitStrategy::Silent`: 使用静默模式初始化
    ///   - `LoggerInitStrategy::Debug`: 启用调试级别日志，显示所有调试信息
    ///   - `LoggerInitStrategy::Auto`: 根据配置的日志级别自动初始化（默认）
    pub fn with_logger_init_strategy(mut self, strategy: LoggerInitStrategy) -> Self {
        self.logger_init_strategy = strategy;
        self
    }
    
    /// 禁用日志初始化（让上层应用控制）
    /// 这是 `with_logger_init_strategy(LoggerInitStrategy::None)` 的便捷方法
    pub fn disable_logger_init(mut self) -> Self {
        self.logger_init_strategy = LoggerInitStrategy::None;
        self
    }
    
    /// 使用静默日志初始化
    /// 这是 `with_logger_init_strategy(LoggerInitStrategy::Silent)` 的便捷方法
    pub fn with_silent_logger_init(mut self) -> Self {
        self.logger_init_strategy = LoggerInitStrategy::Silent;
        self
    }
    
    /// 启用调试级别日志初始化
    /// 这是 `with_logger_init_strategy(LoggerInitStrategy::Debug)` 的便捷方法
    /// 将显示所有调试信息，包括传输创建、地址解析等详细日志
    pub fn with_debug_logger_init(mut self) -> Self {
        self.logger_init_strategy = LoggerInitStrategy::Debug;
        self
    }
    
    /// 启用自动日志初始化
    /// 这是 `with_logger_init_strategy(LoggerInitStrategy::Auto)` 的便捷方法
    pub fn with_auto_logger_init(mut self) -> Self {
        self.logger_init_strategy = LoggerInitStrategy::Auto;
        self
    }
    
    /// 构建解析器
    pub async fn build(self) -> Result<SmartDnsResolver> {
        if self.upstream_manager.get_specs().is_empty() {
            return Err(DnsError::InvalidConfig("No upstream servers configured".to_string()));
        }
        
        // 根据策略初始化日志系统
        match self.logger_init_strategy {
            LoggerInitStrategy::None => {
                // 不初始化日志，让上层应用完全控制
                // 这种情况下，上层应用负责日志初始化
            },
            LoggerInitStrategy::Silent => {
                // 使用静默模式初始化
                let _ = crate::logger::init_dns_logger_silent();
            },
            LoggerInitStrategy::Debug => {
                // 启用调试级别日志，显示所有调试信息
                let _ = crate::logger::init_dns_logger(zerg_creep::logger::LevelFilter::Debug);
            },
            LoggerInitStrategy::Auto => {
                // 根据配置自动初始化（原有逻辑）
                if self.config.log_level == zerg_creep::logger::LevelFilter::Off {
                    // 日志级别为Off时使用静默模式
                    let _ = crate::logger::init_dns_logger_silent();
                } else if self.config.enable_dns_log_format {
                    // 用户显式启用了DNS格式日志
                    let _ = crate::logger::init_dns_logger(self.config.log_level);
                } else {
                    // 用户设置了日志级别但不使用DNS格式
                    let _ = crate::logger::init_dns_logger(self.config.log_level);
                }
            },
        }
        
        // 验证上游服务器配置
        for spec in self.upstream_manager.get_specs() {
            if spec.name.is_empty() {
                return Err(DnsError::InvalidConfig("Upstream name cannot be empty".to_string()));
            }
            if spec.server.is_empty() {
                return Err(DnsError::InvalidConfig(
                    format!("Server address cannot be empty for upstream '{}'", spec.name)
                ));
            }
        }
        
        let decision_engine = match self.query_strategy {
            QueryStrategy::Smart | QueryStrategy::Fifo | QueryStrategy::RoundRobin => {
                let mut engine = SmartDecisionEngine::new(self.current_region.clone());
                
                // 添加所有上游服务器到决策引擎
                for spec in self.upstream_manager.get_specs() {
                    engine.add_upstream(spec.clone()).await?;
                }
                
                Some(Arc::new(engine))
            },
        };
        
        SmartDnsResolver::new(
            self.config,
            self.upstream_manager,
            decision_engine,
            self.query_strategy,
            self.enable_edns,
        )
    }
    
    /// 获取当前配置的上游服务器数量
    pub fn upstream_count(&self) -> usize {
        self.upstream_manager.get_specs().len()
    }
    
    /// 获取当前查询策略
    pub fn current_strategy(&self) -> QueryStrategy {
        self.query_strategy
    }
    
    /// 检查是否启用了EDNS
    pub fn is_edns_enabled(&self) -> bool {
        self.enable_edns
    }
    
    /// 获取当前区域
    pub fn current_region(&self) -> &str {
        &self.current_region
    }
    
    /// 获取上游管理器的引用
    pub fn upstream_manager(&self) -> &UpstreamManager {
        &self.upstream_manager
    }
}