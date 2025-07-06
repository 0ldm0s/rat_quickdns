//! 智能决策引擎模块
//! 
//! 本模块实现基于性能指标的智能上游服务器选择算法

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::upstream_handler::UpstreamSpec;
use crate::error::{DnsError, Result};
use super::metrics::PerformanceMetrics;

/// 失败服务器信息
#[derive(Debug, Clone)]
pub struct FailedServerInfo {
    /// 服务器名称
    pub name: String,
    /// 服务器地址
    pub server: String,
    /// 连续失败次数
    pub consecutive_failures: u32,
    /// 最后失败时间
    pub last_failure_time: Option<Instant>,
    /// 失败原因
    pub failure_reason: String,
}

/// 应急响应信息
#[derive(Debug, Clone)]
pub struct EmergencyResponseInfo {
    /// 是否所有服务器都失败
    pub all_servers_failed: bool,
    /// 失败的服务器列表
    pub failed_servers: Vec<FailedServerInfo>,
    /// 最后一个工作的服务器
    pub last_working_server: Option<String>,
    /// 总失败次数
    pub total_failures: u32,
    /// 应急消息
    pub emergency_message: String,
}

/// 智能决策引擎
#[derive(Debug)]
pub struct SmartDecisionEngine {
    /// 上游服务器规格列表
    upstreams: Arc<RwLock<Vec<UpstreamSpec>>>,
    
    /// 性能指标映射
    metrics: Arc<RwLock<HashMap<String, PerformanceMetrics>>>,
    
    /// 轮询索引（用于轮询策略）
    round_robin_index: Arc<RwLock<usize>>,
    
    /// 当前区域
    current_region: String,
}

impl SmartDecisionEngine {
    /// 创建新的智能决策引擎
    pub fn new(region: impl Into<String>) -> Self {
        Self {
            upstreams: Arc::new(RwLock::new(Vec::new())),
            metrics: Arc::new(RwLock::new(HashMap::new())),
            round_robin_index: Arc::new(RwLock::new(0)),
            current_region: region.into(),
        }
    }
    
    /// 添加上游服务器
    pub async fn add_upstream(&mut self, spec: UpstreamSpec) -> Result<()> {
        let mut upstreams = self.upstreams.write().await;
        let mut metrics = self.metrics.write().await;
        
        // 检查是否已存在同名服务器
        if upstreams.iter().any(|s| s.name == spec.name) {
            return Err(DnsError::InvalidConfig(
                format!("Upstream '{}' already exists", spec.name)
            ));
        }
        
        // 初始化性能指标
        metrics.insert(spec.name.clone(), PerformanceMetrics::new());
        upstreams.push(spec);
        
        Ok(())
    }
    
    /// 移除上游服务器
    pub async fn remove_upstream(&mut self, name: &str) -> Result<()> {
        let mut upstreams = self.upstreams.write().await;
        let mut metrics = self.metrics.write().await;
        
        let index = upstreams.iter().position(|s| s.name == name)
            .ok_or_else(|| DnsError::InvalidConfig(
                format!("Upstream '{}' not found", name)
            ))?;
        
        upstreams.remove(index);
        metrics.remove(name);
        
        Ok(())
    }
    
    /// FIFO策略选择上游服务器
    /// 按照配置顺序依次选择第一个健康的服务器
    pub async fn select_fifo_upstream(&self) -> Option<UpstreamSpec> {
        let upstreams = self.upstreams.read().await;
        let metrics = self.metrics.read().await;
        
        if upstreams.is_empty() {
            return None;
        }
        
        // 按配置顺序查找第一个可用的服务器
        for spec in upstreams.iter() {
            if metrics.get(&spec.name)
                .map(|m| m.is_available())
                .unwrap_or(true) {
                return Some(spec.clone());
            }
        }
        
        // 如果没有可用的服务器，返回None让调用者处理错误
        None
    }
    
    /// 智能策略选择上游服务器（别名方法）
    /// 与select_best_upstream功能相同，为了API一致性
    pub async fn select_smart_upstream(&self) -> Option<UpstreamSpec> {
        self.select_best_upstream().await
    }
    
    /// 选择最佳上游服务器（智能策略）
    pub async fn select_best_upstream(&self) -> Option<UpstreamSpec> {
        let upstreams = self.upstreams.read().await;
        let metrics = self.metrics.read().await;
        
        if upstreams.is_empty() {
            return None;
        }
        
        // 过滤可用的上游服务器
        let available_upstreams: Vec<_> = upstreams
            .iter()
            .filter(|spec| {
                metrics.get(&spec.name)
                    .map(|m| m.is_available())
                    .unwrap_or(true)
            })
            .collect();
        
        if available_upstreams.is_empty() {
            // 没有可用的上游服务器，返回None让调用者处理错误
            return None;
        }
        
        // 计算每个可用服务器的综合评分
        let mut scored_upstreams: Vec<_> = available_upstreams
            .into_iter()
            .map(|spec| {
                let score = self.calculate_upstream_score(spec, &metrics);
                (spec, score)
            })
            .collect();
        
        // 按评分降序排序
        scored_upstreams.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // 选择评分最高的服务器
        scored_upstreams.first().map(|(spec, _)| (*spec).clone())
    }
    
    /// 轮询选择上游服务器（优化版本，集成健康检查和应急策略）
    pub async fn select_round_robin_upstream(&self) -> Option<UpstreamSpec> {
        let upstreams = self.upstreams.read().await;
        let metrics = self.metrics.read().await;
        let mut index = self.round_robin_index.write().await;
        
        if upstreams.is_empty() {
            return None;
        }
        
        // 过滤可用的上游服务器
        let available_upstreams: Vec<(usize, &UpstreamSpec)> = upstreams
            .iter()
            .enumerate()
            .filter(|(_, spec)| {
                metrics.get(&spec.name)
                    .map(|m| m.is_available())
                    .unwrap_or(true)
            })
            .collect();
        
        if available_upstreams.is_empty() {
            // 没有可用的上游服务器，返回None让调用者处理错误
            return None;
        }
        
        // 在可用服务器中进行轮询
        let selected_index = *index % available_upstreams.len();
        *index = (*index + 1) % available_upstreams.len();
        
        available_upstreams.get(selected_index)
            .map(|(_, spec)| (*spec).clone())
    }
    
    /// 检查是否所有上游服务器都不可用
    pub async fn all_upstreams_failed(&self) -> bool {
        let upstreams = self.upstreams.read().await;
        let metrics = self.metrics.read().await;
        
        if upstreams.is_empty() {
            return true;
        }
        
        // 检查是否所有服务器都处于不可用状态
        upstreams.iter().all(|spec| {
            metrics.get(&spec.name)
                .map(|m| !m.is_available())
                .unwrap_or(false)
        })
    }
    
    /// 获取应急响应信息
    pub async fn get_emergency_response_info(&self) -> EmergencyResponseInfo {
        let upstreams = self.upstreams.read().await;
        let metrics = self.metrics.read().await;
        
        let mut failed_servers = Vec::new();
        let mut last_working_server = None;
        let mut total_failures = 0;
        
        for upstream in upstreams.iter() {
            if let Some(metric) = metrics.get(&upstream.name) {
                if !metric.is_available() {
                    failed_servers.push(FailedServerInfo {
                        name: upstream.name.clone(),
                        server: upstream.server.clone(),
                        consecutive_failures: metric.consecutive_failures,
                        last_failure_time: metric.last_failure_time,
                        failure_reason: "Health check failed".to_string(),
                    });
                    total_failures += metric.consecutive_failures;
                } else if metric.last_success_time.is_some() {
                    last_working_server = Some(upstream.name.clone());
                }
            }
        }
        
        EmergencyResponseInfo {
            all_servers_failed: failed_servers.len() == upstreams.len(),
            failed_servers,
            last_working_server,
            total_failures,
            emergency_message: self.generate_emergency_message(&upstreams, &metrics).await,
        }
    }
    
    /// 生成应急消息
    async fn generate_emergency_message(&self, upstreams: &[UpstreamSpec], metrics: &HashMap<String, PerformanceMetrics>) -> String {
        if upstreams.is_empty() {
            return "DNS解析服务不可用：未配置任何上游服务器".to_string();
        }
        
        let available_count = upstreams.iter()
            .filter(|spec| {
                metrics.get(&spec.name)
                    .map(|m| m.is_available())
                    .unwrap_or(true)
            })
            .count();
        
        if available_count == 0 {
            format!(
                "DNS解析服务暂时不可用：所有{}个上游服务器均无响应。请检查网络连接或稍后重试。",
                upstreams.len()
            )
        } else {
            format!(
                "DNS解析服务部分可用：{}/{}个上游服务器正常工作",
                available_count,
                upstreams.len()
            )
        }
    }
    
    /// 快速轮询选择（用于高频查询场景）
    /// 跳过健康检查以提高性能，但会记录失败次数
    pub async fn select_fast_round_robin_upstream(&self) -> Option<UpstreamSpec> {
        let upstreams = self.upstreams.read().await;
        let mut index = self.round_robin_index.write().await;
        
        if upstreams.is_empty() {
            return None;
        }
        
        let selected = upstreams.get(*index).cloned();
        *index = (*index + 1) % upstreams.len();
        selected
    }
    
    /// 计算上游服务器综合评分
    fn calculate_upstream_score(&self, spec: &UpstreamSpec, metrics: &HashMap<String, PerformanceMetrics>) -> f64 {
        let base_score = spec.weight as f64;
        
        let default_metric = PerformanceMetrics::default();
        let metric = metrics.get(&spec.name).unwrap_or(&default_metric);
        
        // 成功率权重 (40%)
        let success_rate = if metric.total_queries > 0 {
            metric.successful_queries as f64 / metric.total_queries as f64
        } else {
            0.8 // 新服务器初始评分
        };
        let success_component = base_score * 0.4 * success_rate;
        
        // 延迟权重 (30%)
        let latency_score = if metric.avg_latency.as_millis() > 0 {
            1000.0 / (metric.avg_latency.as_millis() as f64 + 100.0)
        } else {
            1.0
        };
        let latency_component = base_score * 0.3 * latency_score;
        
        // CDN准确性权重 (20%)
        let cdn_component = base_score * 0.2 * metric.cdn_accuracy_score.max(0.7);
        
        // 连续失败惩罚 (10%)
        let failure_penalty = if metric.consecutive_failures > 3 {
            0.1
        } else if metric.consecutive_failures > 0 {
            1.0 - (metric.consecutive_failures as f64 * 0.2)
        } else {
            1.0
        };
        let penalty_component = base_score * 0.1 * failure_penalty;
        
        let mut total_score = success_component + latency_component + cdn_component + penalty_component;
        
        // 最近成功时间加成
        if let Some(last_success) = metric.last_success_time {
            let time_since_success = Instant::now().duration_since(last_success);
            if time_since_success < Duration::from_secs(60) {
                total_score *= 1.1;
            }
        }
        
        // 区域匹配加成
        if spec.region.as_ref().map(|r| r == &self.current_region).unwrap_or(false) {
            total_score *= 1.2;
        }
        
        total_score
    }
    
    /// 更新性能指标
    pub async fn update_metrics(&self, upstream_name: &str, latency: Duration, success: bool, cdn_accurate: bool) {
        let mut metrics = self.metrics.write().await;
        if let Some(metric) = metrics.get_mut(upstream_name) {
            if success {
                metric.record_success(latency, cdn_accurate);
            } else {
                metric.record_failure();
            }
        }
    }
    
    /// 获取所有上游服务器的性能指标
    pub async fn get_all_metrics(&self) -> HashMap<String, PerformanceMetrics> {
        self.metrics.read().await.clone()
    }
    
    /// 获取指定上游服务器的性能指标
    pub async fn get_metrics(&self, upstream_name: &str) -> Option<PerformanceMetrics> {
        self.metrics.read().await.get(upstream_name).cloned()
    }
    
    /// 重置所有性能指标
    pub async fn reset_metrics(&self) {
        let mut metrics = self.metrics.write().await;
        for metric in metrics.values_mut() {
            metric.reset();
        }
    }
    
    /// 获取上游服务器列表
    pub async fn get_upstreams(&self) -> Vec<UpstreamSpec> {
        self.upstreams.read().await.clone()
    }
    
    /// 获取可用的上游服务器数量
    pub async fn available_upstream_count(&self) -> usize {
        let upstreams = self.upstreams.read().await;
        let metrics = self.metrics.read().await;
        
        upstreams
            .iter()
            .filter(|spec| {
                metrics.get(&spec.name)
                    .map(|m| m.is_available())
                    .unwrap_or(true)
            })
            .count()
    }
    
    /// 设置当前区域
    pub fn set_region(&mut self, region: impl Into<String>) {
        self.current_region = region.into();
    }
    
    /// 获取当前区域
    pub fn current_region(&self) -> &str {
        &self.current_region
    }
}