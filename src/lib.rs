//! RatQuickDNS - 高性能DNS查询库
//!
//! 提供UDP/TCP/DOH/DOT多协议支持、智能决策、缓存、健康检查和客户端IP转发(EDNS Client Subnet)功能

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(unsafe_code)]

pub mod types;
pub mod transport;
pub mod resolver;
pub mod error;
pub mod builder;
pub mod upstream_handler;
pub mod dns_response;
pub mod logger;
pub mod config;
pub mod utils;

#[cfg(feature = "python-bindings")]
pub mod python_api;

pub use types::*;
pub use transport::Transport;
pub use resolver::CoreResolver;
pub use builder::resolver::CoreResolverStats;
pub use error::{DnsError, Result};
pub use builder::{
    DnsResolverBuilder, SmartDnsResolver, DnsQueryRequest, DnsQueryResponse, DnsRecord,
    QueryStrategy, PerformanceMetrics, SmartDecisionEngine, LoggerInitStrategy
};
pub use builder::resolver::UpstreamStatus;
pub use dns_response::{DnsResponseBuilder, DnsResponseWrapper};
pub use logger::{init_dns_logger, init_dns_logger_silent, dns_format};
pub use config::{StrictDnsConfig, StrictConfigBuilder, ConfigError};

// 重新导出zerg_creep基础日志宏到crate根部，供DNS宏使用
pub use zerg_creep::{error, warn, info, debug, trace};


// 注意：移除了 quick_dns 宏，因为它包含兜底行为
// 用户现在必须使用 StrictDnsConfig 明确配置所有参数
// 
// 迁移示例：
// 旧代码: quick_dns!()
// 新代码: 
//   let config = StrictDnsConfig::builder()
//       .strategy(QueryStrategy::Smart)
//       .timeout(Duration::from_secs(5))
//       .retry_count(3)
//       .enable_cache(true)
//       .cache_ttl(Duration::from_secs(3600))
//       .enable_upstream_monitoring(true)
//       .upstream_monitoring_interval(Duration::from_secs(30))
//       .port(53)
//       .concurrent_queries(10)
//       .buffer_size(4096)
//       .enable_stats(true)
//       .emergency_threshold(0.3)
//       .add_upstream(UpstreamSpec::new("8.8.8.8:53".to_string(), "udp".to_string(), 1))
//       .build()?;
//   SmartDnsResolver::from_config(config)?