//! 传输健康检查器

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, Instant, Duration};

/// 基础传输统计
#[derive(Debug, Clone, Default)]
pub struct BasicStats {
    pub success_count: u64,
    pub failure_count: u64,
    pub last_success: Option<SystemTime>,
    pub last_failure: Option<SystemTime>,
    pub avg_response_time: u64,
}

type TransportStats = BasicStats;

/// 健康检查器
#[derive(Debug)]
pub struct HealthChecker {
    /// 传输统计信息
    stats: Arc<RwLock<HashMap<String, DetailedStats>>>,
    /// 检查间隔
    check_interval: Duration,
    /// 健康阈值配置
    config: HealthConfig,
}

/// 健康检查配置
#[derive(Debug, Clone)]
pub struct HealthConfig {
    /// 最小成功率阈值
    pub min_success_rate: f64,
    /// 最大平均响应时间
    pub max_avg_response_time: Duration,
    /// 连续失败阈值
    pub max_consecutive_failures: u32,
    /// 健康恢复所需的连续成功次数
    pub recovery_success_count: u32,
    /// 统计窗口大小
    pub stats_window_size: usize,
    /// 不健康状态的最大持续时间
    pub max_unhealthy_duration: Duration,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            min_success_rate: 0.3,  // 降低成功率要求
            max_avg_response_time: Duration::from_secs(30),  // 增加响应时间容忍度
            max_consecutive_failures: 10,  // 增加连续失败容忍度
            recovery_success_count: 1,  // 降低恢复要求
            stats_window_size: 100,
            max_unhealthy_duration: Duration::from_secs(300),
        }
    }
}

/// 传输健康状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    /// 健康
    Healthy,
    /// 不健康
    Unhealthy,
    /// 未知（刚启动）
    Unknown,
}

/// 详细的传输统计
#[derive(Debug, Clone)]
pub struct DetailedStats {
    /// 成功次数
    pub success_count: u64,
    /// 失败次数
    pub failure_count: u64,
    /// 最后成功时间
    pub last_success: Option<SystemTime>,
    /// 最后失败时间
    pub last_failure: Option<SystemTime>,
    /// 平均响应时间(毫秒)
    pub avg_response_time: u64,
    /// 连续失败次数
    pub consecutive_failures: u32,
    /// 连续成功次数
    pub consecutive_successes: u32,
    /// 健康状态
    pub health_status: HealthStatus,
    /// 状态变更时间
    pub status_changed_at: SystemTime,
}

impl Default for DetailedStats {
    fn default() -> Self {
        Self {
            success_count: 0,
            failure_count: 0,
            last_success: None,
            last_failure: None,
            avg_response_time: 0,
            consecutive_failures: 0,
            consecutive_successes: 0,
            health_status: HealthStatus::Unknown,
            status_changed_at: SystemTime::now(),
        }
    }
}

impl HealthChecker {
    /// 创建新的健康检查器
    pub fn new(check_interval: Duration) -> Self {
        Self {
            stats: Arc::new(RwLock::new(HashMap::new())),
            check_interval,
            config: HealthConfig::default(),
        }
    }
    
    /// 使用自定义配置创建健康检查器
    pub fn with_config(check_interval: Duration, config: HealthConfig) -> Self {
        Self {
            stats: Arc::new(RwLock::new(HashMap::new())),
            check_interval,
            config,
        }
    }
    
    /// 记录成功
    pub fn record_success(&self, transport_type: &str, duration: Duration) {
        if let Ok(mut stats) = self.stats.write() {
            let detailed_stats = stats.entry(transport_type.to_string())
                .or_insert_with(DetailedStats::default);
            
            // 更新基础统计
            detailed_stats.success_count += 1;
            detailed_stats.last_success = Some(SystemTime::now());
            
            // 更新平均响应时间
            if detailed_stats.avg_response_time == 0 {
                detailed_stats.avg_response_time = duration.as_millis() as u64;
            } else {
                detailed_stats.avg_response_time = 
                    (detailed_stats.avg_response_time * 9 + duration.as_millis() as u64) / 10;
            }
            
            // 重置连续失败计数
            detailed_stats.consecutive_failures = 0;
            detailed_stats.consecutive_successes += 1;
            
            // 检查健康状态
            self.update_health_status(detailed_stats);
        }
    }
    
    /// 记录失败
    pub fn record_failure(&self, transport_type: &str) {
        if let Ok(mut stats) = self.stats.write() {
            let detailed_stats = stats.entry(transport_type.to_string())
                .or_insert_with(DetailedStats::default);
            
            // 更新基础统计
            detailed_stats.failure_count += 1;
            detailed_stats.last_failure = Some(SystemTime::now());
            
            // 重置连续成功计数
            detailed_stats.consecutive_successes = 0;
            detailed_stats.consecutive_failures += 1;
            
            // 检查健康状态
            self.update_health_status(detailed_stats);
        }
    }
    
    /// 更新健康状态
    fn update_health_status(&self, stats: &mut DetailedStats) {
        let old_status = stats.health_status.clone();
        let mut new_status = HealthStatus::Unknown;  // 默认为未知状态
        
        let total = stats.success_count + stats.failure_count;
        
        // 如果样本数不足，保持未知状态
        if total < 3 {
            new_status = HealthStatus::Unknown;
        } else {
            // 有足够样本时才进行健康判断
            new_status = HealthStatus::Healthy;
            
            // 检查连续失败
            if stats.consecutive_failures >= self.config.max_consecutive_failures {
                new_status = HealthStatus::Unhealthy;
            }
            
            // 检查成功率（只有足够样本时才检查）
            if total >= 5 && (stats.success_count as f64 / total as f64) < self.config.min_success_rate {
                new_status = HealthStatus::Unhealthy;
            }
            
            // 检查平均响应时间
            if stats.avg_response_time > 0 && Duration::from_millis(stats.avg_response_time) > self.config.max_avg_response_time {
                new_status = HealthStatus::Unhealthy;
            }
        }
        
        // 检查恢复条件
        if stats.health_status == HealthStatus::Unhealthy {
            if stats.consecutive_successes >= self.config.recovery_success_count {
                new_status = HealthStatus::Healthy;
            } else {
                new_status = HealthStatus::Unhealthy;
            }
        }
        
        // 检查长期不健康状态
        if stats.health_status == HealthStatus::Unhealthy {
            if let Ok(elapsed) = stats.status_changed_at.elapsed() {
                if elapsed > self.config.max_unhealthy_duration {
                    // 长期不健康，给一次恢复机会
                    if stats.consecutive_successes > 0 {
                        new_status = HealthStatus::Healthy;
                    }
                }
            }
        }
        
        // 更新状态
        if new_status != old_status {
            stats.health_status = new_status;
            stats.status_changed_at = SystemTime::now();
        }
    }
    
    /// 检查传输是否健康
    pub fn is_healthy(&self, transport_type: &str) -> bool {
        if let Ok(stats) = self.stats.read() {
            if let Some(stats) = stats.get(transport_type) {
                return stats.health_status == HealthStatus::Healthy ||
                       stats.health_status == HealthStatus::Unknown;
            }
        }
        
        // 默认认为是健康的（新传输）
        true
    }
    
    /// 获取传输健康状态
    pub fn get_health_status(&self, transport_type: &str) -> HealthStatus {
        if let Ok(stats) = self.stats.read() {
            if let Some(detailed_stats) = stats.get(transport_type) {
                return detailed_stats.health_status.clone();
            }
        }
        
        HealthStatus::Unknown
    }
    
    /// 获取所有传输的统计信息
    pub fn get_stats(&self) -> HashMap<String, (u64, u64, Duration)> {
        let mut result = HashMap::new();
        
        if let Ok(stats) = self.stats.read() {
            for (transport_type, detailed_stats) in stats.iter() {
                result.insert(
                    transport_type.clone(),
                    (
                        detailed_stats.success_count,
                        detailed_stats.failure_count,
                        Duration::from_millis(detailed_stats.avg_response_time),
                    ),
                );
            }
        }
        
        result
    }
    
    /// 获取详细统计信息
    pub fn get_detailed_stats(&self) -> HashMap<String, DetailedStats> {
        if let Ok(stats) = self.stats.read() {
            stats.clone()
        } else {
            HashMap::new()
        }
    }
    
    /// 获取健康的传输列表
    pub fn get_healthy_transports(&self) -> Vec<String> {
        let mut healthy = Vec::new();
        
        if let Ok(stats) = self.stats.read() {
            for (transport_type, detailed_stats) in stats.iter() {
                if detailed_stats.health_status == HealthStatus::Healthy ||
                   detailed_stats.health_status == HealthStatus::Unknown {
                    healthy.push(transport_type.clone());
                }
            }
        }
        
        healthy
    }
    
    /// 检查传输是否健康（包括新传输）
    pub fn is_transport_healthy(&self, transport_type: &str) -> bool {
        if let Ok(stats) = self.stats.read() {
            if let Some(detailed_stats) = stats.get(transport_type) {
                // 有统计记录的传输，检查其健康状态
                detailed_stats.health_status == HealthStatus::Healthy ||
                detailed_stats.health_status == HealthStatus::Unknown
            } else {
                // 新传输默认认为是健康的
                true
            }
        } else {
            // 无法读取统计信息时，默认认为健康
            true
        }
    }
    
    /// 获取不健康的传输列表
    pub fn get_unhealthy_transports(&self) -> Vec<String> {
        let mut unhealthy = Vec::new();
        
        if let Ok(stats) = self.stats.read() {
            for (transport_type, detailed_stats) in stats.iter() {
                if detailed_stats.health_status == HealthStatus::Unhealthy {
                    unhealthy.push(transport_type.clone());
                }
            }
        }
        
        unhealthy
    }
    
    /// 重置传输统计
    pub fn reset_stats(&self, transport_type: &str) {
        if let Ok(mut stats) = self.stats.write() {
            if let Some(detailed_stats) = stats.get_mut(transport_type) {
                *detailed_stats = DetailedStats::default();
            }
        }
    }
    
    /// 重置所有统计
    pub fn reset_all_stats(&self) {
        if let Ok(mut stats) = self.stats.write() {
            stats.clear();
        }
    }
    
    /// 强制设置传输健康状态
    pub fn set_health_status(&self, transport_type: &str, status: HealthStatus) {
        if let Ok(mut stats) = self.stats.write() {
            let detailed_stats = stats.entry(transport_type.to_string())
                .or_insert_with(DetailedStats::default);
            
            if detailed_stats.health_status != status {
                detailed_stats.health_status = status;
                detailed_stats.status_changed_at = SystemTime::now();
            }
        }
    }
    
    /// 获取传输排名（按健康程度和性能）
    pub fn get_transport_ranking(&self) -> Vec<(String, f64)> {
        let mut rankings = Vec::new();
        
        if let Ok(stats) = self.stats.read() {
            for (transport_type, detailed_stats) in stats.iter() {
                let mut score = 0.0;
                
                // 健康状态分数
                match detailed_stats.health_status {
                    HealthStatus::Healthy => score += 100.0,
                    HealthStatus::Unknown => score += 80.0,
                    HealthStatus::Unhealthy => score += 0.0,
                }
                
                // 成功率分数
                let total = detailed_stats.success_count + detailed_stats.failure_count;
                let success_rate = if total > 0 {
                    detailed_stats.success_count as f64 / total as f64
                } else {
                    0.0
                };
                score += success_rate * 50.0;
                
                // 响应时间分数（越快越好）
                let avg_ms = detailed_stats.avg_response_time as f64;
                if avg_ms > 0.0 {
                    score += (1000.0 / avg_ms.max(1.0)).min(50.0).max(0.0);
                }
                
                // 连续成功分数
                score += (detailed_stats.consecutive_successes as f64).min(20.0).max(0.0);
                
                // 连续失败惩罚
                score -= (detailed_stats.consecutive_failures as f64) * 5.0;
                
                rankings.push((transport_type.clone(), score.max(0.0)));
            }
        }
        
        // 按分数降序排序
        rankings.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        rankings
    }
    
    /// 获取检查间隔
    pub fn check_interval(&self) -> Duration {
        self.check_interval
    }
    
    /// 设置检查间隔
    pub fn set_check_interval(&mut self, interval: Duration) {
        self.check_interval = interval;
    }
    
    /// 获取配置
    pub fn config(&self) -> &HealthConfig {
        &self.config
    }
    
    /// 更新配置
    pub fn update_config(&mut self, config: HealthConfig) {
        self.config = config;
    }
}

/// 健康检查任务
pub struct HealthCheckTask {
    checker: Arc<HealthChecker>,
}

impl HealthCheckTask {
    /// 创建新的健康检查任务
    pub fn new(checker: Arc<HealthChecker>) -> Self {
        Self { checker }
    }
    
    /// 启动健康检查任务
    pub async fn start(self) {
        let mut interval = tokio::time::interval(self.checker.check_interval());
        
        loop {
            interval.tick().await;
            
            // 执行健康检查逻辑
            // 这里可以添加主动健康检查，比如发送ping请求
            self.perform_health_check().await;
        }
    }
    
    /// 执行健康检查
    async fn perform_health_check(&self) {
        // 获取所有传输的统计信息
        let stats = self.checker.get_detailed_stats();
        
        for (transport_type, detailed_stats) in stats {
            // 检查长时间无活动的传输
            if let Some(last_success) = detailed_stats.last_success {
                if let Ok(elapsed) = last_success.elapsed() {
                    if elapsed > Duration::from_secs(300) {
                        // 5分钟无成功请求，可能需要主动检查
                        // 这里可以添加主动ping逻辑
                    }
                }
            }
            
            // 检查长时间不健康的传输
            if detailed_stats.health_status == HealthStatus::Unhealthy {
                if let Ok(elapsed) = detailed_stats.status_changed_at.elapsed() {
                    if elapsed > self.checker.config().max_unhealthy_duration {
                        // 给予恢复机会
                        self.checker.set_health_status(&transport_type, HealthStatus::Unknown);
                    }
                }
            }
        }
    }
}