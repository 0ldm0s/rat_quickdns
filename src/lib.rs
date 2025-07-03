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

#[cfg(feature = "python-bindings")]
pub mod python_api;

pub use types::*;
pub use transport::Transport;
pub use resolver::Resolver;
pub use error::{DnsError, Result};
pub use builder::{
    DnsResolverBuilder, EasyDnsResolver, DnsQueryRequest, DnsQueryResponse, DnsRecord,
    QueryStrategy, PerformanceMetrics, SmartDecisionEngine
};
pub use builder::resolver::{ResolverStats, UpstreamHealth};

// 重新导出rat_quickmem的核心功能
pub use rat_quickmem::{encode, decode, QuickMemConfig};

/// 便捷宏：快速创建DNS解析器
#[macro_export]
macro_rules! quick_dns {
    () => {
        $crate::EasyDnsResolver::quick_setup()
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