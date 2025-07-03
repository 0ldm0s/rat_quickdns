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
    
    /// 选择最佳上游服务器（智能策略）
    pub async fn select_best_upstream(&self) -> Option<UpstreamSpec> {
        let upstreams = self.upstreams.read().await;
        let metrics = self.metrics.read().await;
        
        if upstreams.is_empty() {
            return None;
        }
        
        // 过滤健康的上游服务器
        let healthy_upstreams: Vec<_> = upstreams
            .iter()
            .filter(|spec| {
                metrics.get(&spec.name)
                    .map(|m| m.is_healthy())
                    .unwrap_or(true)
            })
            .collect();
        
        if healthy_upstreams.is_empty() {
            // 如果没有健康的服务器，选择连续失败次数最少的
            return upstreams
                .iter()
                .min_by_key(|spec| {
                    metrics.get(&spec.name)
                        .map(|m| m.consecutive_failures)
                        .unwrap_or(0)
                })
                .cloned();
        }
        
        // 计算每个健康服务器的综合评分
        let mut scored_upstreams: Vec<_> = healthy_upstreams
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
    
    /// 轮询选择上游服务器
    pub async fn select_round_robin_upstream(&self) -> Option<UpstreamSpec> {
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
    
    /// 获取健康的上游服务器数量
    pub async fn healthy_upstream_count(&self) -> usize {
        let upstreams = self.upstreams.read().await;
        let metrics = self.metrics.read().await;
        
        upstreams
            .iter()
            .filter(|spec| {
                metrics.get(&spec.name)
                    .map(|m| m.is_healthy())
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