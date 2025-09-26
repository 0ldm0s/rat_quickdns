//! 传输健康检查器

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, Instant, Duration};

/// 基础传输统计
#[derive(Debug, Clone, Default)]
pub struct BasicStats {
    /// 成功计数
    pub success_count: u64,
    /// 失败计数
    pub failure_count: u64,
    /// 最后成功时间
    pub last_success: Option<SystemTime>,
    /// 最后失败时间
    pub last_failure: Option<SystemTime>,
    /// 平均响应时间（毫秒）
    pub avg_response_time: u64,
}

type TransportStats = BasicStats;

/// 上游监控器
#[derive(Debug)]
pub struct UpstreamMonitor {
    /// 传输统计信息
    stats: Arc<RwLock<HashMap<String, DetailedStats>>>,
    /// 检查间隔
    check_interval: Duration,
    /// 健康阈值配置
    config: UpstreamConfig,
}

/// 上游监控配置
#[derive(Debug, Clone)]
pub struct UpstreamConfig {
    /// 最小成功率阈值
    pub min_success_rate: f64,
    /// 最大平均响应时间
    pub max_avg_response_time: Duration,
    /// 连续失败阈值
    pub max_consecutive_failures: u32,
    /// 状态恢复所需的连续成功次数
    pub recovery_success_count: u32,
    /// 统计窗口大小
    pub stats_window_size: usize,
    /// 最大不可用持续时间
    pub max_unavailable_duration: Duration,
}

// 注意：移除了 Default 实现，因为它包含兜底行为
// 硬编码的监控参数（如 0.3 成功率、30秒响应时间等）是兜底代码
// 这些"宽松"的默认值可能掩盖真实的性能问题
// 用户现在必须根据实际需求明确配置监控参数

/// 上游状态
#[derive(Debug, Clone, PartialEq)]
pub enum UpstreamStatus {
    /// 可用
    Available,
    /// 不可用
    Unavailable,
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
    /// 上游状态
    pub upstream_status: UpstreamStatus,
    /// 状态变更时间
    pub status_changed_at: SystemTime,
}

// 注意：保留 DetailedStats 的 Default 实现，因为这是功能性需求
// 统计数据的初始化不是兜底行为，而是正常的数据结构初始化
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
            upstream_status: UpstreamStatus::Unknown,
            status_changed_at: SystemTime::now(),
        }
    }
}

impl UpstreamMonitor {
    // 注意：移除了使用默认配置的构造函数
    // 用户现在必须明确提供 UpstreamConfig，不能依赖兜底配置
    // 
    // 迁移示例：
    // 旧代码: HealthChecker::new(Duration::from_secs(30))
    // 新代码: UpstreamMonitor::with_config(Duration::from_secs(30), your_health_config)
    
    /// 使用自定义配置创建上游监控器
    pub fn with_config(check_interval: Duration, config: UpstreamConfig) -> Self {
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
            
            // 检查上游状态
            self.update_upstream_status(detailed_stats);
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
            
            // 检查上游状态
            self.update_upstream_status(detailed_stats);
        }
    }
    
    /// 更新上游状态
    fn update_upstream_status(&self, stats: &mut DetailedStats) {
        let old_status = stats.upstream_status.clone();
        let mut new_status = UpstreamStatus::Unknown;  // 默认为未知状态
        
        let total = stats.success_count + stats.failure_count;
        
        // 如果样本数不足，保持未知状态
        if total < 3 {
            new_status = UpstreamStatus::Unknown;
        } else {
            // 有足够样本时才进行状态判断
            new_status = UpstreamStatus::Available;
            
            // 检查连续失败
            if stats.consecutive_failures >= self.config.max_consecutive_failures {
                new_status = UpstreamStatus::Unavailable;
            }
            
            // 检查成功率（只有足够样本时才检查）
            if total >= 5 && (stats.success_count as f64 / total as f64) < self.config.min_success_rate {
                new_status = UpstreamStatus::Unavailable;
            }
            
            // 检查平均响应时间
            if stats.avg_response_time > 0 && Duration::from_millis(stats.avg_response_time) > self.config.max_avg_response_time {
                new_status = UpstreamStatus::Unavailable;
            }
        }
        
        // 检查恢复条件
        if stats.upstream_status == UpstreamStatus::Unavailable {
            if stats.consecutive_successes >= self.config.recovery_success_count {
                new_status = UpstreamStatus::Available;
            } else {
                new_status = UpstreamStatus::Unavailable;
            }
        }
        
        // 检查长期不可用状态
        if stats.upstream_status == UpstreamStatus::Unavailable {
            if let Ok(elapsed) = stats.status_changed_at.elapsed() {
                if elapsed > self.config.max_unavailable_duration {
                    // 长期不可用，给一次恢复机会
                    if stats.consecutive_successes > 0 {
                        new_status = UpstreamStatus::Available;
                    }
                }
            }
        }
        
        // 更新状态
        if new_status != old_status {
            stats.upstream_status = new_status;
            stats.status_changed_at = SystemTime::now();
        }
    }
    
    /// 检查传输是否可用
    pub fn is_available(&self, transport_type: &str) -> bool {
        if let Ok(stats) = self.stats.read() {
            if let Some(stats) = stats.get(transport_type) {
                return stats.upstream_status == UpstreamStatus::Available ||
                       stats.upstream_status == UpstreamStatus::Unknown;
            }
        }
        
        // 默认认为是可用的（新传输）
        true
    }
    
    /// 获取传输上游状态
    pub fn get_upstream_status(&self, transport_type: &str) -> UpstreamStatus {
        if let Ok(stats) = self.stats.read() {
            if let Some(detailed_stats) = stats.get(transport_type) {
                return detailed_stats.upstream_status.clone();
            }
        }
        
        UpstreamStatus::Unknown
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
    
    /// 获取可用的传输列表
    pub fn get_available_transports(&self) -> Vec<String> {
        let mut available = Vec::new();
        
        if let Ok(stats) = self.stats.read() {
            for (transport_type, detailed_stats) in stats.iter() {
                if detailed_stats.upstream_status == UpstreamStatus::Available ||
               detailed_stats.upstream_status == UpstreamStatus::Unknown {
                    available.push(transport_type.clone());
                }
            }
        }
        
        available
    }
    
    /// 检查传输是否可用（包括新传输）
    pub fn is_transport_available(&self, transport_type: &str) -> bool {
        if let Ok(stats) = self.stats.read() {
            if let Some(detailed_stats) = stats.get(transport_type) {
                // 有统计记录的传输，检查其健康状态
                detailed_stats.upstream_status == UpstreamStatus::Available ||
                detailed_stats.upstream_status == UpstreamStatus::Unknown
            } else {
                // 新传输默认认为是健康的
                true
            }
        } else {
            // 无法读取统计信息时，默认认为健康
            true
        }
    }
    
    /// 获取不可用的传输列表
    pub fn get_unavailable_transports(&self) -> Vec<String> {
        let mut unavailable = Vec::new();
        
        if let Ok(stats) = self.stats.read() {
            for (transport_type, detailed_stats) in stats.iter() {
                if detailed_stats.upstream_status == UpstreamStatus::Unavailable {
                    unavailable.push(transport_type.clone());
                }
            }
        }
        
        unavailable
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
    
    /// 设置传输上游状态（用于测试或手动干预）
    pub fn set_upstream_status(&self, transport_type: &str, status: UpstreamStatus) {
        if let Ok(mut stats) = self.stats.write() {
            let detailed_stats = stats.entry(transport_type.to_string())
                .or_insert_with(DetailedStats::default);
            
            if detailed_stats.upstream_status != status {
                detailed_stats.upstream_status = status;
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
                
                // 上游状态分数
                match detailed_stats.upstream_status {
                    UpstreamStatus::Available => score += 100.0,
            UpstreamStatus::Unknown => score += 80.0,
            UpstreamStatus::Unavailable => score += 0.0,
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
    
    /// 获取配置引用
    pub fn config(&self) -> &UpstreamConfig {
        &self.config
    }
    
    /// 更新配置
    pub fn update_config(&mut self, config: UpstreamConfig) {
        self.config = config;
    }
}

/// 上游监控任务
pub struct UpstreamMonitorTask {
    monitor: Arc<UpstreamMonitor>,
}

impl UpstreamMonitorTask {
    /// 创建新的上游监控任务
    pub fn new(monitor: Arc<UpstreamMonitor>) -> Self {
        Self { monitor }
    }
    
    /// 启动上游监控任务
    pub async fn start(self) {
        let mut interval = tokio::time::interval(self.monitor.check_interval());
        
        loop {
            interval.tick().await;
            
            // 执行上游监控逻辑
            // 这里可以添加主动健康检查，比如发送ping请求
            self.perform_upstream_monitoring().await;
        }
    }
    
    /// 执行上游监控
    async fn perform_upstream_monitoring(&self) {
        // 获取所有传输的统计信息
        let stats = self.monitor.get_detailed_stats();
        
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
            if detailed_stats.upstream_status == UpstreamStatus::Unavailable {
                if let Ok(elapsed) = detailed_stats.status_changed_at.elapsed() {
                    if elapsed > self.monitor.config().max_unavailable_duration {
                        // 给予恢复机会
                        self.monitor.set_upstream_status(&transport_type, UpstreamStatus::Unknown);
                    }
                }
            }
        }
    }
}