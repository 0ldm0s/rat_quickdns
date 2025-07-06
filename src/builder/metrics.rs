//! DNS解析器性能指标模块
//! 
//! 本模块提供DNS上游服务器的性能指标收集、统计和分析功能

use std::time::{Duration, Instant};

/// 上游服务器性能指标
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// 总查询次数
    pub total_queries: u64,
    
    /// 成功查询次数
    pub successful_queries: u64,
    
    /// 失败查询次数
    pub failed_queries: u64,
    
    /// 连续失败次数
    pub consecutive_failures: u32,
    
    /// 平均延迟
    pub avg_latency: Duration,
    
    /// 最后成功时间
    pub last_success_time: Option<Instant>,
    
    /// 最后失败时间
    pub last_failure_time: Option<Instant>,
    
    /// CDN准确性评分 (0.0-1.0)
    pub cdn_accuracy_score: f64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            total_queries: 0,
            successful_queries: 0,
            failed_queries: 0,
            consecutive_failures: 0,
            avg_latency: Duration::from_millis(100), // 默认100ms
            last_success_time: None,
            last_failure_time: None,
            cdn_accuracy_score: 0.8, // 默认80%准确率
        }
    }
}

impl PerformanceMetrics {
    /// 创建新的性能指标实例
    pub fn new() -> Self {
        Self::default()
    }
    
    /// 计算成功率
    pub fn success_rate(&self) -> f64 {
        if self.total_queries == 0 {
            0.0
        } else {
            self.successful_queries as f64 / self.total_queries as f64
        }
    }
    
    /// 计算失败率
    pub fn failure_rate(&self) -> f64 {
        1.0 - self.success_rate()
    }
    
    /// 是否可用（基于连续失败次数）- 修正术语，更准确描述服务器可用性
    pub fn is_available(&self) -> bool {
        self.consecutive_failures < 5
    }
    
    /// 获取延迟等级描述
    pub fn latency_grade(&self) -> &'static str {
        let ms = self.avg_latency.as_millis();
        match ms {
            0..=50 => "优秀",
            51..=100 => "良好",
            101..=200 => "一般",
            201..=500 => "较差",
            _ => "很差",
        }
    }
    
    /// 更新成功查询指标
    pub fn record_success(&mut self, latency: Duration, cdn_accurate: bool) {
        self.total_queries += 1;
        self.successful_queries += 1;
        self.consecutive_failures = 0;
        self.last_success_time = Some(Instant::now());
        
        // 更新平均延迟（指数移动平均）
        if self.total_queries == 1 {
            self.avg_latency = latency;
        } else {
            let alpha = 0.1; // 平滑因子
            let old_latency_ms = self.avg_latency.as_millis() as f64;
            let new_latency_ms = latency.as_millis() as f64;
            let smoothed_latency_ms = old_latency_ms * (1.0 - alpha) + new_latency_ms * alpha;
            self.avg_latency = Duration::from_millis(smoothed_latency_ms as u64);
        }
        
        // 更新CDN准确性评分
        let current_score = self.cdn_accuracy_score * (self.successful_queries - 1) as f64;
        let new_score = if cdn_accurate { 1.0 } else { 0.0 };
        self.cdn_accuracy_score = (current_score + new_score) / self.successful_queries as f64;
    }
    
    /// 更新失败查询指标
    pub fn record_failure(&mut self) {
        self.total_queries += 1;
        self.failed_queries += 1;
        self.consecutive_failures += 1;
        self.last_failure_time = Some(Instant::now());
    }
    
    /// 重置指标
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    
    /// 获取综合评分 (0.0-1.0)
    pub fn overall_score(&self) -> f64 {
        let success_weight = 0.4;
        let latency_weight = 0.3;
        let cdn_weight = 0.2;
        let availability_weight = 0.1;
        
        let success_score = self.success_rate();
        
        let latency_score = if self.avg_latency.as_millis() > 0 {
            (1000.0 / (self.avg_latency.as_millis() as f64 + 100.0)).min(1.0)
        } else {
            1.0
        };
        
        let cdn_score = self.cdn_accuracy_score;
        
        let availability_score = if self.is_available() { 1.0 } else { 0.0 };
        
        success_score * success_weight
            + latency_score * latency_weight
            + cdn_score * cdn_weight
            + availability_score * availability_weight
    }
}