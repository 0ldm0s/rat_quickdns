//! DNS解析器构建器模块
//! 
//! 本模块提供了构建高性能DNS解析器的完整功能，包括：
//! - 查询策略定义
//! - 性能指标收集
//! - 智能决策引擎
//! - 解析器构建器
//! - DNS解析器实现

pub mod strategy;
pub mod metrics;
pub mod engine;
pub mod resolver_builder;
pub mod resolver;
pub mod types;

// 重新导出主要类型
pub use strategy::QueryStrategy;
pub use metrics::PerformanceMetrics;
pub use engine::SmartDecisionEngine;
pub use resolver_builder::DnsResolverBuilder;
pub use resolver::SmartDnsResolver;
pub use types::*;

// 为了向后兼容，保持原有的导出
pub use resolver_builder::DnsResolverBuilder as Builder;
pub use resolver::SmartDnsResolver as Resolver;