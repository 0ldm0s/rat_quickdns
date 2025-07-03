//! 查询策略定义

use crate::{Response, Result};
use std::time::Duration;

/// 查询策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryStrategy {
    /// 最快优先：并发查询所有传输，返回最快的结果
    FastestFirst,
    /// 并行查询：同时查询所有传输，等待所有结果后选择最佳
    Parallel,
    /// 顺序查询：按顺序尝试每个传输，直到成功
    Sequential,
    /// 智能决策：综合考虑速度、可靠性和结果完整性
    SmartDecision,
}

/// 查询结果
#[derive(Debug)]
pub struct QueryResult {
    /// 查询响应
    pub response: Result<Response>,
    /// 查询耗时
    pub duration: Duration,
    /// 传输类型
    pub transport_type: String,
}

/// 传输性能统计
#[derive(Debug, Clone)]
pub struct TransportStats {
    /// 成功次数
    pub success_count: u64,
    /// 失败次数
    pub failure_count: u64,
    /// 平均响应时间
    pub avg_response_time: Duration,
    /// 最后成功时间
    pub last_success: Option<std::time::Instant>,
    /// 最后失败时间
    pub last_failure: Option<std::time::Instant>,
    /// 健康状态
    pub is_healthy: bool,
}

impl Default for TransportStats {
    fn default() -> Self {
        Self {
            success_count: 0,
            failure_count: 0,
            avg_response_time: Duration::from_millis(0),
            last_success: None,
            last_failure: None,
            is_healthy: true,
        }
    }
}

impl TransportStats {
    /// 记录成功
    pub fn record_success(&mut self, duration: Duration) {
        self.success_count += 1;
        self.last_success = Some(std::time::Instant::now());
        
        // 更新平均响应时间
        if self.success_count == 1 {
            self.avg_response_time = duration;
        } else {
            let total_time = self.avg_response_time * (self.success_count - 1) as u32 + duration;
            self.avg_response_time = total_time / self.success_count as u32;
        }
        
        // 更新健康状态
        self.update_health_status();
    }
    
    /// 记录失败
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure = Some(std::time::Instant::now());
        
        // 更新健康状态
        self.update_health_status();
    }
    
    /// 更新健康状态
    fn update_health_status(&mut self) {
        let total_requests = self.success_count + self.failure_count;
        
        if total_requests == 0 {
            self.is_healthy = true;
            return;
        }
        
        let success_rate = self.success_count as f64 / total_requests as f64;
        
        // 如果成功率低于50%，标记为不健康
        if success_rate < 0.5 {
            self.is_healthy = false;
        } else if success_rate > 0.8 {
            // 如果成功率高于80%，标记为健康
            self.is_healthy = true;
        }
        
        // 如果最近连续失败太多，也标记为不健康
        if let Some(last_failure) = self.last_failure {
            if let Some(last_success) = self.last_success {
                if last_failure > last_success && 
                   last_failure.elapsed() < Duration::from_secs(30) {
                    // 最近30秒内失败且没有成功，标记为不健康
                    self.is_healthy = false;
                }
            } else if last_failure.elapsed() < Duration::from_secs(30) {
                // 从未成功过且最近失败，标记为不健康
                self.is_healthy = false;
            }
        }
    }
    
    /// 获取成功率
    pub fn success_rate(&self) -> f64 {
        let total = self.success_count + self.failure_count;
        if total == 0 {
            1.0
        } else {
            self.success_count as f64 / total as f64
        }
    }
    
    /// 获取总请求数
    pub fn total_requests(&self) -> u64 {
        self.success_count + self.failure_count
    }
    
    /// 重置统计
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// 查询策略配置
#[derive(Debug, Clone)]
pub struct StrategyConfig {
    /// 并行查询超时时间
    pub parallel_timeout: Duration,
    /// 顺序查询重试间隔
    pub sequential_retry_delay: Duration,
    /// 智能决策等待时间
    pub smart_decision_wait_time: Duration,
    /// 健康检查阈值
    pub health_check_threshold: f64,
    /// 最小成功率
    pub min_success_rate: f64,
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            parallel_timeout: Duration::from_secs(10),
            sequential_retry_delay: Duration::from_millis(100),
            smart_decision_wait_time: Duration::from_millis(500),
            health_check_threshold: 0.8,
            min_success_rate: 0.5,
        }
    }
}

/// 结果评分器
#[derive(Debug)]
pub struct ResultScorer;

impl ResultScorer {
    /// 计算响应质量分数
    pub fn score_response(response: &Response, duration: Duration) -> f64 {
        let mut score = 0.0;
        
        // 基础分数：有答案记录
        if !response.answers.is_empty() {
            score += 100.0;
            
            // 答案数量加分
            score += response.answers.len() as f64 * 10.0;
        }
        
        // 权威记录加分
        if !response.authorities.is_empty() {
            score += 20.0;
        }
        
        // 附加记录加分
        if !response.additionals.is_empty() {
            score += 10.0;
        }
        
        // 响应时间加分（越快分数越高）
        let time_score = if duration.as_millis() == 0 {
            50.0
        } else {
            (1000.0 / duration.as_millis() as f64).min(50.0)
        };
        score += time_score;
        
        // 检查响应码
        match response.flags.rcode {
            0 => score += 0.0,  // NOERROR
            3 => score -= 50.0, // NXDOMAIN
            2 => score -= 100.0, // SERVFAIL
            1 => score -= 80.0,  // FORMERR
            5 => score -= 90.0,  // REFUSED
            _ => score -= 60.0,  // 其他错误
        }
        
        score.max(0.0)
    }
    
    /// 选择最佳响应
    pub fn select_best_response(results: &[QueryResult]) -> Option<&QueryResult> {
        let mut best_result: Option<&QueryResult> = None;
        let mut best_score = -1.0;
        
        for result in results {
            if let Ok(response) = &result.response {
                let score = Self::score_response(response, result.duration);
                
                if score > best_score {
                    best_score = score;
                    best_result = Some(result);
                }
            }
        }
        
        best_result
    }
    
    /// 检查响应是否完整
    pub fn is_response_complete(response: &Response) -> bool {
        // 检查是否有答案记录
        if response.answers.is_empty() {
            return false;
        }
        
        // 检查响应码
        if response.flags.rcode != 0 {
            return false;
        }
        
        // 检查是否被截断
        if response.flags.tc {
            return false;
        }
        
        true
    }
    
    /// 比较两个响应的质量
    pub fn compare_responses(
        resp1: &Response,
        duration1: Duration,
        resp2: &Response,
        duration2: Duration,
    ) -> std::cmp::Ordering {
        let score1 = Self::score_response(resp1, duration1);
        let score2 = Self::score_response(resp2, duration2);
        
        score1.partial_cmp(&score2).unwrap_or(std::cmp::Ordering::Equal)
    }
}