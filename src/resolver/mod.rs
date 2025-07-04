//! 智能DNS解析器

use crate::{Request, Response, Result, DnsError};
use crate::types::{Query, RecordType, QClass, Flags, ClientSubnet};
use crate::transport::{Transport, UdpTransport, TcpTransport, TlsTransport, HttpsTransport};
use crate::transport::{TransportConfig, TlsConfig, HttpsConfig};
use std::fmt::Debug;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::net::IpAddr;
use tokio::time::timeout;
use std::collections::HashMap;
use crate::{dns_debug, dns_info, dns_error, dns_transport};

pub mod strategy;
pub mod cache;
pub mod health;

use strategy::QueryStrategy;
use strategy::QueryResult;
use cache::DnsCache;
use health::HealthChecker;

/// 智能DNS解析器
#[derive(Debug, Clone)]
pub struct Resolver {
    /// 传输层实例
    transports: Vec<Arc<dyn Transport + Send + Sync + 'static>>,
    /// 查询策略
    strategy: QueryStrategy,
    /// DNS缓存
    cache: Option<Arc<DnsCache>>,
    /// 健康检查器
    health_checker: Option<Arc<HealthChecker>>,
    /// 默认超时时间
    default_timeout: Duration,
    /// 重试次数
    retry_count: usize,
    /// 默认客户端子网信息
    default_client_subnet: Option<ClientSubnet>,
}

/// 解析器配置
#[derive(Debug, Clone)]
pub struct ResolverConfig {
    /// 查询策略
    pub strategy: QueryStrategy,
    /// 默认超时时间
    pub default_timeout: Duration,
    /// 重试次数
    pub retry_count: usize,
    /// 是否启用缓存
    pub enable_cache: bool,
    /// 缓存TTL上限
    pub max_cache_ttl: Duration,
    /// 是否启用健康检查
    pub enable_health_check: bool,
    /// 健康检查间隔
    pub health_check_interval: Duration,
    /// 默认客户端子网信息
    pub default_client_subnet: Option<ClientSubnet>,
    /// DNS服务器端口
    pub port: u16,
    /// 并发查询数量
    pub concurrent_queries: usize,
    /// 是否启用递归查询
    pub recursion_desired: bool,
    /// 查询缓冲区大小
    pub buffer_size: usize,
    /// 日志级别
    pub log_level: zerg_creep::logger::LevelFilter,
    /// 是否启用DNS专用日志格式
    pub enable_dns_log_format: bool,
}

impl Default for ResolverConfig {
    fn default() -> Self {
        Self {
            strategy: QueryStrategy::FastestFirst,
            default_timeout: Duration::from_secs(5),
            retry_count: 2,
            enable_cache: true,
            max_cache_ttl: Duration::from_secs(3600),
            enable_health_check: true,
            health_check_interval: Duration::from_secs(30),
            default_client_subnet: None,
            port: 53,
            concurrent_queries: 10,
            recursion_desired: true,
            buffer_size: 4096,
            log_level: zerg_creep::logger::LevelFilter::Off,
            enable_dns_log_format: true,
        }
    }
}

impl Resolver {
    /// 创建新的解析器
    pub fn new(config: ResolverConfig) -> Self {
        let cache = if config.enable_cache {
            Some(Arc::new(DnsCache::new(config.max_cache_ttl)))
        } else {
            None
        };
        
        let health_checker = if config.enable_health_check {
            Some(Arc::new(HealthChecker::new(config.health_check_interval)))
        } else {
            None
        };
        
        Self {
            transports: Vec::new(),
            strategy: config.strategy,
            cache,
            health_checker,
            default_timeout: config.default_timeout,
            retry_count: config.retry_count,
            default_client_subnet: config.default_client_subnet,
        }
    }
    
    /// 使用默认配置创建解析器
    pub fn default() -> Self {
        Self::new(ResolverConfig::default())
    }
    
    /// 添加UDP传输
    pub fn add_udp_transport(&mut self, config: TransportConfig) {
        dns_transport!("添加UDP传输: {}:{}", config.server, config.port);
        let transport = Arc::new(UdpTransport::new(config));
        self.transports.push(transport.clone());
        dns_info!("UDP传输已添加，当前传输总数: {}", self.transports.len());
        dns_debug!("新添加的传输类型: {}", transport.transport_type());
    }
    
    /// 添加TCP传输
    pub fn add_tcp_transport(&mut self, config: TransportConfig) {
        let transport = Arc::new(TcpTransport::new(config));
        self.transports.push(transport);
    }
    
    /// 添加TLS传输
    pub fn add_tls_transport(&mut self, config: TlsConfig) -> Result<()> {
        let transport = Arc::new(TlsTransport::new(config)?);
        self.transports.push(transport);
        Ok(())
    }
    
    /// 添加HTTPS传输
    pub fn add_https_transport(&mut self, config: HttpsConfig) -> Result<()> {
        let transport = Arc::new(HttpsTransport::new(config)?);
        self.transports.push(transport);
        Ok(())
    }
    
    /// 添加自定义传输
    pub fn add_transport(&mut self, transport: Arc<dyn Transport>) {
        self.transports.push(transport);
    }
    
    /// 查询DNS记录
    pub async fn query(
        &self,
        name: &str,
        record_type: RecordType,
        class: QClass,
    ) -> Result<Response> {
        self.query_with_client_ip(name, record_type, class, None).await
    }
    
    /// 查询DNS记录并指定客户端IP
    pub async fn query_with_client_ip(
        &self,
        name: &str,
        record_type: RecordType,
        class: QClass,
        client_ip: Option<IpAddr>,
    ) -> Result<Response> {
        let client_subnet = client_ip.map(|ip| match ip {
            IpAddr::V4(addr) => ClientSubnet::from_ipv4(addr, 24),
            IpAddr::V6(addr) => ClientSubnet::from_ipv6(addr, 56),
        });
        
        let query = Query {
            name: name.to_string(),
            qtype: record_type,
            qclass: class,
        };
        
        // 检查缓存
        if let Some(cache) = &self.cache {
            if let Some(cached_response) = cache.get(&query) {
                return Ok(cached_response);
            }
        }
        
        // 创建DNS请求
        let request = Request {
            id: rand::random(),
            flags: Flags::default(),
            query: query.clone(),
            client_subnet: client_subnet.or_else(|| self.default_client_subnet.clone()),
        };
        
        // 执行查询策略
        let response = self.execute_query_strategy(&request).await?;
        
        // 缓存结果
        if let Some(cache) = &self.cache {
            cache.insert(query, response.clone());
        }
        
        Ok(response)
    }
    
    /// 设置默认客户端子网
    pub fn set_default_client_subnet(&mut self, client_subnet: Option<ClientSubnet>) {
        self.default_client_subnet = client_subnet;
    }
    
    /// 设置默认客户端IP（便捷方法）
    pub fn set_default_client_ip(&mut self, client_ip: Option<IpAddr>) {
        self.default_client_subnet = client_ip.map(|ip| match ip {
            IpAddr::V4(addr) => ClientSubnet::from_ipv4(addr, 24),
            IpAddr::V6(addr) => ClientSubnet::from_ipv6(addr, 56),
        });
    }
    
    /// 执行查询策略
    async fn execute_query_strategy(&self, request: &Request) -> Result<Response> {
        if self.transports.is_empty() {
            return Err(DnsError::Config("No transports configured".to_string()));
        }
        
        match self.strategy {
            QueryStrategy::FastestFirst => self.query_fastest_first(request).await,
            QueryStrategy::Parallel => self.query_parallel(request).await,
            QueryStrategy::Sequential => self.query_sequential(request).await,
            QueryStrategy::SmartDecision => self.query_smart_decision(request).await,
        }
    }
    
    /// 最快优先策略（优化版：支持早期取消）
    async fn query_fastest_first(&self, request: &Request) -> Result<Response> {
        use tokio::sync::{oneshot, broadcast};
        
        // 获取健康的传输实例
        let healthy_transports = self.get_healthy_transports();
        
        if healthy_transports.is_empty() {
            return Err(DnsError::Server("No healthy transports available".to_string()));
        }
        
        // 创建取消通道，用于在获得第一个成功响应后取消其他任务
        let (cancel_tx, _) = broadcast::channel::<()>(1);
        let cancel_tx = Arc::new(cancel_tx);
        let (success_tx, mut success_rx) = oneshot::channel();
        let success_tx = Arc::new(tokio::sync::Mutex::new(Some(success_tx)));
        
        // 并发查询所有传输
        let mut tasks = Vec::new();
        
        for transport in healthy_transports {
            let transport_clone = Arc::clone(&transport);
            let request_clone = request.clone();
            let mut cancel_rx = cancel_tx.subscribe();
            let success_tx_clone = success_tx.clone();
            let cancel_tx_clone = cancel_tx.clone();
            let health_checker = self.health_checker.clone();
            
            let task = tokio::spawn(async move {
                let start = Instant::now();
                
                // 使用select!来同时监听取消信号和DNS查询
                tokio::select! {
                    // DNS查询结果
                    result = transport_clone.send(&request_clone) => {
                        let duration = start.elapsed();
                        let transport_type = transport_clone.transport_type();
                        
                        match result {
                            Ok(response) => {
                                // 记录成功统计
                                if let Some(health_checker) = &health_checker {
                                    health_checker.record_success(transport_type, duration);
                                }
                                
                                // 尝试发送成功结果（只有第一个成功的会被接收）
                                if let Ok(mut sender) = success_tx_clone.try_lock() {
                                    if let Some(tx) = sender.take() {
                                        let _ = tx.send(Ok(response));
                                        // 通知其他任务取消
                                        let _ = cancel_tx_clone.send(());
                                    }
                                }
                            }
                            Err(e) => {
                                // 记录失败统计
                                if let Some(health_checker) = &health_checker {
                                    health_checker.record_failure(transport_type);
                                }
                                // 失败不取消其他任务，继续等待
                            }
                        }
                    }
                    // 取消信号
                    _ = cancel_rx.recv() => {
                        // 任务被取消，直接退出
                        dns_debug!("传输 {} 的查询任务被取消", transport_clone.transport_type());
                    }
                }
            });
            
            tasks.push(task);
        }
        
        // 使用oneshot通道来处理任务完成情况
        let (all_done_tx, all_done_rx) = oneshot::channel::<Result<Response>>();
        let all_done_tx = Arc::new(tokio::sync::Mutex::new(Some(all_done_tx)));
        
        // 创建一个单独的任务来等待所有查询完成
        let all_tasks_handle = tokio::spawn({
            let all_done_tx = all_done_tx.clone();
            async move {
                // 等待所有任务完成
                let _ = futures::future::join_all(tasks).await;
                
                // 如果还没有成功结果，则发送失败信息
                if let Ok(mut sender) = all_done_tx.try_lock() {
                    if let Some(tx) = sender.take() {
                        let _ = tx.send(Err(DnsError::Server("All transports failed".to_string())));
                    }
                }
            }
        });
        
        // 等待第一个成功的结果或所有任务完成
        let result = tokio::select! {
            // 收到成功响应
            result = &mut success_rx => {
                // 取消所有剩余任务
                let _ = cancel_tx.send(());
                match result {
                    Ok(response) => response,
                    Err(_) => Err(DnsError::Server("Internal communication error".to_string()))
                }
            }
            // 所有任务都完成了但没有成功响应
            result = all_done_rx => {
                result.unwrap_or(Err(DnsError::Server("Internal communication error".to_string())))
            }
        };
        
        // 等待清理任务完成（设置短超时避免长时间等待）
        let _ = tokio::time::timeout(Duration::from_millis(100), all_tasks_handle).await;
        
        result
    }
    
    /// 并行查询策略
    async fn query_parallel(&self, request: &Request) -> Result<Response> {
        let healthy_transports = self.get_healthy_transports();
        
        if healthy_transports.is_empty() {
            return Err(DnsError::Server("No healthy transports available".to_string()));
        }
        
        let mut tasks = Vec::new();
        
        for transport in healthy_transports {
            let transport_clone = Arc::clone(&transport);
            let request_clone = request.clone();
            
            let task = tokio::spawn(async move {
                transport_clone.send(&request_clone).await
            });
            
            tasks.push(task);
        }
        
        // 等待所有任务完成
        let results = futures::future::join_all(tasks).await;
        
        // 返回第一个成功的结果
        for result in results {
            if let Ok(Ok(response)) = result {
                return Ok(response);
            }
        }
        
        Err(DnsError::Server("All parallel queries failed".to_string()))
    }
    
    /// 顺序查询策略
    async fn query_sequential(&self, request: &Request) -> Result<Response> {
        let healthy_transports = self.get_healthy_transports();
        
        if healthy_transports.is_empty() {
            return Err(DnsError::Server("No healthy transports available".to_string()));
        }
        
        let mut last_error = DnsError::Server("No transports tried".to_string());
        
        for transport in healthy_transports {
            for attempt in 0..=self.retry_count {
                match transport.send(request).await {
                    Ok(response) => return Ok(response),
                    Err(e) => {
                        last_error = e;
                        if attempt < self.retry_count {
                            tokio::time::sleep(Duration::from_millis(100 * (attempt + 1) as u64)).await;
                        }
                    }
                }
            }
        }
        
        Err(last_error)
    }
    
    /// 智能决策策略
    async fn query_smart_decision(&self, request: &Request) -> Result<Response> {
        // 智能决策：结合速度、可靠性和结果完整性
        let healthy_transports = self.get_healthy_transports();
        
        if healthy_transports.is_empty() {
            return Err(DnsError::Server("No healthy transports available".to_string()));
        }
        
        let mut tasks = Vec::new();
        
        for transport in healthy_transports {
            let transport_clone = Arc::clone(&transport);
            let request_clone = request.clone();
            
            let task = tokio::spawn(async move {
                let start = Instant::now();
                let result = transport_clone.send(&request_clone).await;
                let duration = start.elapsed();
                QueryResult {
                    response: result,
                    duration,
                    transport_type: transport_clone.transport_type().to_string(),
                }
            });
            
            tasks.push(task);
        }
        
        // 收集所有结果
        let mut results = Vec::new();
        let mut fastest_response: Option<Response> = None;
        let mut fastest_time = Duration::from_secs(u64::MAX);
        
        // 等待所有结果或超时
        let timeout_duration = self.default_timeout;
        let deadline = Instant::now() + timeout_duration;
        
        while !tasks.is_empty() && Instant::now() < deadline {
            let remaining_time = deadline.duration_since(Instant::now());
            
            match timeout(remaining_time, futures::future::select_all(tasks)).await {
                Ok((task_result, _index, remaining_tasks)) => {
                    tasks = remaining_tasks;
                    
                    if let Ok(query_result) = task_result {
                        if let Ok(response) = &query_result.response {
                            // 记录最快的响应
                            if query_result.duration < fastest_time {
                                fastest_time = query_result.duration;
                                fastest_response = Some(response.clone());
                            }
                        }
                        results.push(query_result);
                    }
                }
                Err(_) => break, // 超时
            }
        }
        
        // 智能选择最佳结果
        self.select_best_result(results, fastest_response)
    }
    
    /// 选择最佳查询结果
    fn select_best_result(
        &self,
        results: Vec<QueryResult>,
        fastest_response: Option<Response>,
    ) -> Result<Response> {
        if results.is_empty() {
            return Err(DnsError::Timeout);
        }
        
        // 按优先级选择结果：
        // 1. 结果最完整的（答案记录最多）
        // 2. 如果完整性相同，选择最快的
        // 3. 如果都失败，返回最快的错误
        
        let mut best_response: Option<Response> = None;
        let mut best_score = -1i32;
        let mut best_duration = Duration::from_secs(u64::MAX);
        
        for result in &results {
            if let Ok(response) = &result.response {
                let score = response.answers.len() as i32;
                
                if score > best_score || 
                   (score == best_score && result.duration < best_duration) {
                    best_score = score;
                    best_duration = result.duration;
                    best_response = Some(response.clone());
                }
            }
        }
        
        // 如果有完整结果，返回最佳结果
        if let Some(response) = best_response {
            return Ok(response);
        }
        
        // 如果没有成功结果，但有最快的响应，返回它
        if let Some(response) = fastest_response {
            return Ok(response);
        }
        
        // 返回第一个错误
        for result in results {
            if let Err(e) = result.response {
                return Err(e);
            }
        }
        
        Err(DnsError::Server("No valid results".to_string()))
    }
    
    /// 获取健康的传输实例
    fn get_healthy_transports(&self) -> Vec<Arc<dyn Transport + Send + Sync + 'static>> {
        dns_debug!("开始获取健康的传输实例");
        dns_debug!("总传输数量: {}", self.transports.len());
        
        for (i, transport) in self.transports.iter().enumerate() {
            dns_debug!("传输 {}: 类型={}", i, transport.transport_type());
        }
        
        if let Some(health_checker) = &self.health_checker {
            dns_debug!("使用健康检查器过滤传输");
            let healthy_transports: Vec<_> = self.transports
                .iter()
                .enumerate()
                .filter(|(i, t)| {
                    let is_healthy = health_checker.is_transport_healthy(t.transport_type());
                    dns_debug!("传输 {} ({}): 健康状态={}", i, t.transport_type(), is_healthy);
                    is_healthy
                })
                .map(|(_, t)| t.clone())
                .collect();
            
            dns_debug!("健康传输数量: {}", healthy_transports.len());
            healthy_transports
        } else {
            dns_debug!("未启用健康检查，返回所有传输");
            self.transports.clone()
        }
    }
    
    /// 获取传输统计信息
    pub fn get_transport_stats(&self) -> HashMap<String, (u64, u64, Duration)> {
        if let Some(health_checker) = &self.health_checker {
            health_checker.get_stats()
        } else {
            HashMap::new()
        }
    }
    
    /// 清空缓存
    pub fn clear_cache(&self) {
        if let Some(cache) = &self.cache {
            cache.clear();
        }
    }
}