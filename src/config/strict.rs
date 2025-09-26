//! 严格配置模式 - 移除所有兜底默认值
//!
//! 这个模块实现了严格的DNS配置，强制用户明确每个配置项，
//! 不提供任何隐式的默认值或自动修复功能。

use std::time::Duration;
use serde::{Deserialize, Serialize};
use crate::builder::strategy::QueryStrategy;

/// 严格DNS配置错误类型
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// 缺少必需的配置参数
    #[error("Missing required configuration: {0}")]
    MissingRequired(String),
    /// 无效的配置值
    #[error("Invalid configuration value: {0}")]
    InvalidValue(String),
    /// 未配置上游服务器
    #[error("No upstreams configured - at least one upstream server is required")]
    NoUpstreams,
    /// 无效的超时设置
    #[error("Invalid timeout: {0}")]
    InvalidTimeout(String),
    /// 无效的重试次数
    #[error("Invalid retry count: {0}")]
    InvalidRetryCount(String),
    /// 无效的缓冲区大小
    #[error("Invalid buffer size: {0}")]
    InvalidBufferSize(String),
    /// 无效的端口号
    #[error("Invalid port: {0}")]
    InvalidPort(String),
}

/// 上游服务器规格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamSpec {
    /// 服务器地址（必须包含端口）
    pub address: String,
    /// 协议类型
    pub protocol: String,
    /// 权重（用于负载均衡）
    pub weight: u32,
    /// 是否启用
    pub enabled: bool,
}

/// 严格DNS配置 - 强制用户明确每个配置项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrictDnsConfig {
    /// 查询策略（必须明确指定）
    pub strategy: QueryStrategy,
    /// 默认超时时间（必须明确指定）
    pub default_timeout: Duration,
    /// 重试次数（必须明确指定）
    pub retry_count: usize,
    /// 是否启用缓存（必须明确指定）
    pub enable_cache: bool,
    /// 最大缓存TTL（必须明确指定）
    pub max_cache_ttl: Duration,
    /// 是否启用上游监控（必须明确指定）
    pub enable_upstream_monitoring: bool,
    /// 上游监控间隔（必须明确指定）
    pub upstream_monitoring_interval: Duration,
    /// 端口（必须明确指定）
    pub port: u16,
    /// 并发查询数（必须明确指定）
    pub concurrent_queries: usize,
    /// 缓冲区大小（必须明确指定）
    pub buffer_size: usize,
    /// 上游服务器列表（必须明确配置）
    pub upstreams: Vec<UpstreamSpec>,
    /// 是否启用统计收集（必须明确指定）
    pub enable_stats: bool,
    /// 应急模式阈值（必须明确指定）
    pub emergency_threshold: f64,
}

/// 严格配置构建器 - 强制用户明确每个配置项
pub struct StrictConfigBuilder {
    strategy: Option<QueryStrategy>,
    default_timeout: Option<Duration>,
    retry_count: Option<usize>,
    enable_cache: Option<bool>,
    max_cache_ttl: Option<Duration>,
    enable_upstream_monitoring: Option<bool>,
    upstream_monitoring_interval: Option<Duration>,
    port: Option<u16>,
    concurrent_queries: Option<usize>,
    buffer_size: Option<usize>,
    upstreams: Vec<UpstreamSpec>,
    enable_stats: Option<bool>,
    emergency_threshold: Option<f64>,
}

impl StrictConfigBuilder {
    /// 创建新的严格配置构建器
    /// 
    /// 注意：所有配置项都必须明确指定，不提供任何默认值
    pub fn new() -> Self {
        Self {
            strategy: None,
            default_timeout: None,
            retry_count: None,
            enable_cache: None,
            max_cache_ttl: None,
            enable_upstream_monitoring: None,
            upstream_monitoring_interval: None,
            port: None,
            concurrent_queries: None,
            buffer_size: None,
            upstreams: Vec::new(),
            enable_stats: None,
            emergency_threshold: None,
        }
    }
    
    /// 设置查询策略
    pub fn strategy(mut self, strategy: QueryStrategy) -> Self {
        self.strategy = Some(strategy);
        self
    }
    
    /// 设置默认超时时间
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = Some(timeout);
        self
    }
    
    /// 设置重试次数
    pub fn retry_count(mut self, count: usize) -> Self {
        self.retry_count = Some(count);
        self
    }
    
    /// 设置是否启用缓存
    pub fn enable_cache(mut self, enable: bool) -> Self {
        self.enable_cache = Some(enable);
        self
    }
    
    /// 设置最大缓存TTL
    pub fn cache_ttl(mut self, ttl: Duration) -> Self {
        self.max_cache_ttl = Some(ttl);
        self
    }
    
    /// 设置是否启用上游监控
    pub fn enable_upstream_monitoring(mut self, enable: bool) -> Self {
        self.enable_upstream_monitoring = Some(enable);
        self
    }
    
    /// 设置上游监控间隔
    pub fn upstream_monitoring_interval(mut self, interval: Duration) -> Self {
        self.upstream_monitoring_interval = Some(interval);
        self
    }
    
    /// 设置端口
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }
    
    /// 设置并发查询数
    pub fn concurrent_queries(mut self, count: usize) -> Self {
        self.concurrent_queries = Some(count);
        self
    }
    
    /// 设置缓冲区大小
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = Some(size);
        self
    }
    
    /// 添加上游服务器
    /// 
    /// 注意：不会自动添加端口或进行任何格式修正
    pub fn add_upstream(mut self, spec: UpstreamSpec) -> Self {
        self.upstreams.push(spec);
        self
    }
    
    /// 设置是否启用统计收集
    pub fn enable_stats(mut self, enable: bool) -> Self {
        self.enable_stats = Some(enable);
        self
    }
    
    /// 设置应急模式阈值
    pub fn emergency_threshold(mut self, threshold: f64) -> Self {
        self.emergency_threshold = Some(threshold);
        self
    }
    
    /// 构建严格配置
    /// 
    /// 如果任何必需的配置项缺失，将返回错误
    pub fn build(self) -> Result<StrictDnsConfig, ConfigError> {
        let config = StrictDnsConfig {
            strategy: self.strategy.ok_or_else(|| 
                ConfigError::MissingRequired("strategy".to_string()))?,
            default_timeout: self.default_timeout.ok_or_else(|| 
                ConfigError::MissingRequired("default_timeout".to_string()))?,
            retry_count: self.retry_count.ok_or_else(|| 
                ConfigError::MissingRequired("retry_count".to_string()))?,
            enable_cache: self.enable_cache.ok_or_else(|| 
                ConfigError::MissingRequired("enable_cache".to_string()))?,
            max_cache_ttl: self.max_cache_ttl.ok_or_else(|| 
                ConfigError::MissingRequired("max_cache_ttl".to_string()))?,
            enable_upstream_monitoring: self.enable_upstream_monitoring.ok_or_else(|| 
                ConfigError::MissingRequired("enable_upstream_monitoring".to_string()))?,
            upstream_monitoring_interval: self.upstream_monitoring_interval.ok_or_else(|| 
                ConfigError::MissingRequired("upstream_monitoring_interval".to_string()))?,
            port: self.port.ok_or_else(|| 
                ConfigError::MissingRequired("port".to_string()))?,
            concurrent_queries: self.concurrent_queries.ok_or_else(|| 
                ConfigError::MissingRequired("concurrent_queries".to_string()))?,
            buffer_size: self.buffer_size.ok_or_else(|| 
                ConfigError::MissingRequired("buffer_size".to_string()))?,
            enable_stats: self.enable_stats.ok_or_else(|| 
                ConfigError::MissingRequired("enable_stats".to_string()))?,
            emergency_threshold: self.emergency_threshold.ok_or_else(|| 
                ConfigError::MissingRequired("emergency_threshold".to_string()))?,
            upstreams: if self.upstreams.is_empty() {
                return Err(ConfigError::NoUpstreams);
            } else {
                self.upstreams
            },
        };
        
        // 构建后立即验证
        config.validate()?;
        Ok(config)
    }
}

impl StrictDnsConfig {
    /// 创建严格配置构建器
    pub fn builder() -> StrictConfigBuilder {
        StrictConfigBuilder::new()
    }
    
    /// 严格验证配置，不容忍任何无效值
    /// 
    /// 这个方法不会尝试修复任何配置问题，而是直接报错
    pub fn validate(&self) -> Result<(), ConfigError> {
        // 验证超时时间
        if self.default_timeout.as_millis() == 0 {
            return Err(ConfigError::InvalidTimeout(
                "Timeout cannot be zero".to_string()));
        }
        
        if self.default_timeout.as_secs() > 300 {
            return Err(ConfigError::InvalidTimeout(
                "Timeout cannot exceed 300 seconds".to_string()));
        }
        
        // 验证重试次数
        if self.retry_count == 0 {
            return Err(ConfigError::InvalidRetryCount(
                "Retry count cannot be zero".to_string()));
        }
        
        if self.retry_count > 10 {
            return Err(ConfigError::InvalidRetryCount(
                "Retry count cannot exceed 10".to_string()));
        }
        
        // 验证端口
        if self.port == 0 {
            return Err(ConfigError::InvalidPort(
                "Port cannot be zero".to_string()));
        }
        
        // 验证并发查询数
        if self.concurrent_queries == 0 {
            return Err(ConfigError::InvalidValue(
                "Concurrent queries cannot be zero".to_string()));
        }
        
        if self.concurrent_queries > 1000 {
            return Err(ConfigError::InvalidValue(
                "Concurrent queries cannot exceed 1000".to_string()));
        }
        
        // 验证缓冲区大小
        if self.buffer_size < 512 {
            return Err(ConfigError::InvalidBufferSize(
                "Buffer size must be at least 512 bytes".to_string()));
        }
        
        if self.buffer_size > 65536 {
            return Err(ConfigError::InvalidBufferSize(
                "Buffer size cannot exceed 65536 bytes".to_string()));
        }
        
        // 验证上游服务器
        if self.upstreams.is_empty() {
            return Err(ConfigError::NoUpstreams);
        }
        
        // 验证每个上游服务器规格
        for (i, upstream) in self.upstreams.iter().enumerate() {
            if upstream.address.is_empty() {
                return Err(ConfigError::InvalidValue(
                    format!("Upstream {} address cannot be empty", i)));
            }
            
            // 严格要求地址包含端口
            if !upstream.address.contains(':') {
                return Err(ConfigError::InvalidValue(
                    format!("Upstream {} address must include port (e.g., '8.8.8.8:53')", i)));
            }
            
            if upstream.protocol.is_empty() {
                return Err(ConfigError::InvalidValue(
                    format!("Upstream {} protocol cannot be empty", i)));
            }
            
            if upstream.weight == 0 {
                return Err(ConfigError::InvalidValue(
                    format!("Upstream {} weight cannot be zero", i)));
            }
        }
        
        // 验证应急阈值
        if self.emergency_threshold < 0.0 || self.emergency_threshold > 1.0 {
            return Err(ConfigError::InvalidValue(
                "Emergency threshold must be between 0.0 and 1.0".to_string()));
        }
        
        // 验证缓存TTL
        if self.enable_cache && self.max_cache_ttl.as_secs() == 0 {
            return Err(ConfigError::InvalidValue(
                "Cache TTL cannot be zero when cache is enabled".to_string()));
        }
        
        // 验证上游监控间隔
        if self.enable_upstream_monitoring && self.upstream_monitoring_interval.as_secs() == 0 {
            return Err(ConfigError::InvalidValue(
                "Upstream monitoring interval cannot be zero when upstream monitoring is enabled".to_string()));
        }
        
        Ok(())
    }
    
    /// 获取启用的上游服务器列表
    pub fn enabled_upstreams(&self) -> Vec<&UpstreamSpec> {
        self.upstreams.iter().filter(|u| u.enabled).collect()
    }
    
    /// 检查配置是否启用了智能功能
    pub fn is_smart_mode(&self) -> bool {
        matches!(self.strategy, QueryStrategy::Smart)
    }
}

impl UpstreamSpec {
    /// 创建新的上游服务器规格
    /// 
    /// 注意：不会自动添加端口或进行格式修正
    pub fn new(address: String, protocol: String, weight: u32) -> Self {
        Self {
            address,
            protocol,
            weight,
            enabled: true,
        }
    }
    
    /// 创建禁用的上游服务器规格
    pub fn disabled(address: String, protocol: String, weight: u32) -> Self {
        Self {
            address,
            protocol,
            weight,
            enabled: false,
        }
    }
    
    /// 解析地址和端口
    /// 
    /// 如果地址格式无效，返回错误而不是尝试修复
    pub fn parse_address(&self) -> Result<(String, u16), ConfigError> {
        let parts: Vec<&str> = self.address.split(':').collect();
        if parts.len() != 2 {
            return Err(ConfigError::InvalidValue(
                format!("Invalid address format: '{}'. Expected 'host:port'", self.address)));
        }
        
        let host = parts[0].to_string();
        let port = parts[1].parse::<u16>()
            .map_err(|_| ConfigError::InvalidValue(
                format!("Invalid port in address: '{}'", self.address)))?;
        
        Ok((host, port))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[test]
    fn test_strict_config_builder_missing_required() {
        let result = StrictDnsConfig::builder().build();
        assert!(result.is_err());
        
        if let Err(ConfigError::MissingRequired(field)) = result {
            assert_eq!(field, "strategy");
        } else {
            panic!("Expected MissingRequired error");
        }
    }
    
    #[test]
    fn test_strict_config_builder_success() {
        let upstream = UpstreamSpec::new(
            "8.8.8.8:53".to_string(),
            "udp".to_string(),
            1
        );
        
        let config = StrictDnsConfig::builder()
            .strategy(QueryStrategy::Smart)
            .timeout(Duration::from_secs(5))
            .retry_count(3)
            .enable_cache(true)
            .cache_ttl(Duration::from_secs(3600))
            .enable_upstream_monitoring(true)
            .upstream_monitoring_interval(Duration::from_secs(30))
            .port(53)
            .concurrent_queries(10)
            .buffer_size(4096)
            .enable_stats(true)
            .emergency_threshold(0.3)
            .add_upstream(upstream)
            .build();
        
        assert!(config.is_ok());
    }
    
    #[test]
    fn test_upstream_spec_parse_address() {
        let upstream = UpstreamSpec::new(
            "8.8.8.8:53".to_string(),
            "udp".to_string(),
            1
        );
        
        let (host, port) = upstream.parse_address().unwrap();
        assert_eq!(host, "8.8.8.8");
        assert_eq!(port, 53);
    }
    
    #[test]
    fn test_upstream_spec_invalid_address() {
        let upstream = UpstreamSpec::new(
            "8.8.8.8".to_string(), // 缺少端口
            "udp".to_string(),
            1
        );
        
        let result = upstream.parse_address();
        assert!(result.is_err());
    }
}