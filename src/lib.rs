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

#[cfg(feature = "python-bindings")]
pub mod python_api;

pub use types::*;
pub use transport::Transport;
pub use resolver::CoreResolver;
pub use builder::resolver::CoreResolverStats;
pub use error::{DnsError, Result};
pub use builder::{
    DnsResolverBuilder, SmartDnsResolver, DnsQueryRequest, DnsQueryResponse, DnsRecord,
    QueryStrategy, PerformanceMetrics, SmartDecisionEngine
};
pub use builder::resolver::UpstreamHealth;
pub use dns_response::{DnsResponseBuilder, DnsResponseWrapper};
pub use logger::{init_dns_logger, init_dns_logger_silent, dns_format};

// 重新导出zerg_creep基础日志宏到crate根部，供DNS宏使用
pub use zerg_creep::{error, warn, info, debug, trace};

// 重新导出rat_quickmem的核心功能
pub use rat_quickmem::{encode, decode, QuickMemConfig};

/// 便捷宏：快速创建DNS解析器
#[macro_export]
macro_rules! quick_dns {
    () => {
        $crate::SmartDnsResolver::quick_setup()
    };
    (public) => {
        $crate::DnsResolverBuilder::new().with_public_dns().build()
    };
    (timeout = $timeout:expr) => {
        $crate::DnsResolverBuilder::new()
            .with_public_dns()
            .with_timeout(std::time::Duration::from_secs($timeout))
            .build()
    };
    (servers = [$($server:expr),*]) => {
        {
            let mut builder = $crate::DnsResolverBuilder::new();
            $(
                builder = builder.add_udp_server($server, 53);
            )*
            builder.build()
        }
    };
}