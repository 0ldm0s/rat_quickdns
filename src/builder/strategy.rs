//! DNS查询策略定义
//! 
//! 本模块定义了DNS解析器支持的各种查询策略

use serde::{Deserialize, Serialize};

/// DNS查询策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryStrategy {
    /// FIFO策略：按照配置顺序依次查询上游服务器
    Fifo,
    
    /// 智能策略：基于性能指标和网络状况智能选择最优上游服务器
    Smart,
    
    /// 轮询策略：轮流使用不同的上游服务器
    RoundRobin,
}

impl Default for QueryStrategy {
    fn default() -> Self {
        Self::Smart
    }
}

impl QueryStrategy {
    /// 获取策略描述
    pub fn description(&self) -> &'static str {
        match self {
            Self::Fifo => "按配置顺序依次查询，适合有明确优先级的场景",
            Self::Smart => "基于性能指标智能选择，适合追求最优性能的场景",
            Self::RoundRobin => "轮流使用不同服务器，适合负载均衡场景",
        }
    }
    
    /// 是否需要性能指标收集
    pub fn requires_metrics(&self) -> bool {
        matches!(self, Self::Smart)
    }
    
    /// 是否支持并发查询
    pub fn supports_concurrent(&self) -> bool {
        matches!(self, Self::Fifo | Self::Smart)
    }
}