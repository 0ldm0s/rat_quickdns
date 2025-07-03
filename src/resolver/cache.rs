//! DNS缓存实现

use crate::{Query, Response, Record};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// DNS缓存条目
#[derive(Debug, Clone)]
struct CacheEntry {
    /// 缓存的响应
    response: Response,
    /// 插入时间
    inserted_at: Instant,
    /// 过期时间
    expires_at: Instant,
    /// 原始TTL
    original_ttl: Duration,
}

/// DNS缓存
#[derive(Debug)]
pub struct DnsCache {
    /// 缓存存储
    cache: Arc<RwLock<HashMap<CacheKey, CacheEntry>>>,
    /// 最大TTL
    max_ttl: Duration,
    /// 缓存统计
    stats: Arc<RwLock<CacheStats>>,
}

/// 缓存键
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    /// 查询名称
    name: String,
    /// 查询类型
    qtype: u16,
    /// 查询类别
    qclass: u16,
}

/// 缓存统计
#[derive(Debug, Default, Clone)]
pub struct CacheStats {
    /// 命中次数
    pub hits: u64,
    /// 未命中次数
    pub misses: u64,
    /// 插入次数
    pub inserts: u64,
    /// 过期清理次数
    pub evictions: u64,
    /// 当前缓存大小
    pub current_size: usize,
}

impl CacheKey {
    /// 从查询创建缓存键
    fn from_query(query: &Query) -> Self {
        Self {
            name: query.name.to_lowercase(),
            qtype: query.qtype.into(),
            qclass: query.qclass.into(),
        }
    }
}

impl DnsCache {
    /// 创建新的DNS缓存
    pub fn new(max_ttl: Duration) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_ttl,
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }
    
    /// 获取缓存记录
    pub fn get(&self, query: &Query) -> Option<Response> {
        let key = CacheKey::from_query(query);
        let now = Instant::now();
        
        let cache = self.cache.read().ok()?;
        
        if let Some(entry) = cache.get(&key) {
            if now < entry.expires_at {
                // 更新统计
                if let Ok(mut stats) = self.stats.write() {
                    stats.hits += 1;
                }
                
                // 调整TTL
                let mut response = entry.response.clone();
                let remaining_ttl = entry.expires_at.duration_since(now);
                
                // 更新所有记录的TTL
                for record in &mut response.answers {
                    record.ttl = remaining_ttl.as_secs() as u32;
                }
                for record in &mut response.authorities {
                    record.ttl = remaining_ttl.as_secs() as u32;
                }
                for record in &mut response.additionals {
                    record.ttl = remaining_ttl.as_secs() as u32;
                }
                
                return Some(response);
            }
        }
        
        // 更新统计
        if let Ok(mut stats) = self.stats.write() {
            stats.misses += 1;
        }
        
        None
    }
    
    /// 插入缓存记录
    pub fn insert(&self, query: Query, response: Response) {
        let key = CacheKey::from_query(&query);
        let now = Instant::now();
        
        // 计算TTL
        let ttl = self.calculate_ttl(&response);
        if ttl.is_zero() {
            return; // 不缓存TTL为0的记录
        }
        
        let entry = CacheEntry {
            response,
            inserted_at: now,
            expires_at: now + ttl,
            original_ttl: ttl,
        };
        
        if let Ok(mut cache) = self.cache.write() {
            let is_new = !cache.contains_key(&key);
            cache.insert(key, entry);
            
            // 更新统计
            if let Ok(mut stats) = self.stats.write() {
                stats.inserts += 1;
                if is_new {
                    stats.current_size = cache.len();
                }
            }
        }
    }
    
    /// 计算缓存TTL
    fn calculate_ttl(&self, response: &Response) -> Duration {
        let mut min_ttl = self.max_ttl;
        
        // 找到最小的TTL
        for record in &response.answers {
            let record_ttl = Duration::from_secs(record.ttl as u64);
            if record_ttl < min_ttl {
                min_ttl = record_ttl;
            }
        }
        
        for record in &response.authorities {
            let record_ttl = Duration::from_secs(record.ttl as u64);
            if record_ttl < min_ttl {
                min_ttl = record_ttl;
            }
        }
        
        for record in &response.additionals {
            let record_ttl = Duration::from_secs(record.ttl as u64);
            if record_ttl < min_ttl {
                min_ttl = record_ttl;
            }
        }
        
        // 确保不超过最大TTL
        min_ttl.min(self.max_ttl)
    }
    
    /// 清理过期条目
    pub fn cleanup_expired(&self) {
        let now = Instant::now();
        let mut evicted_count = 0;
        
        if let Ok(mut cache) = self.cache.write() {
            let original_size = cache.len();
            cache.retain(|_, entry| {
                let keep = now < entry.expires_at;
                if !keep {
                    evicted_count += 1;
                }
                keep
            });
            
            // 更新统计
            if let Ok(mut stats) = self.stats.write() {
                stats.evictions += evicted_count;
                stats.current_size = cache.len();
            }
        }
    }
    
    /// 清空缓存
    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
            
            // 重置统计
            if let Ok(mut stats) = self.stats.write() {
                stats.current_size = 0;
            }
        }
    }
    
    /// 获取缓存大小
    pub fn size(&self) -> usize {
        self.cache.read().map(|cache| cache.len()).unwrap_or(0)
    }
    
    /// 获取缓存统计
    pub fn stats(&self) -> CacheStats {
        self.stats.read().map(|stats| stats.clone()).unwrap_or_default()
    }
    
    /// 获取命中率
    pub fn hit_rate(&self) -> f64 {
        let stats = self.stats();
        let total = stats.hits + stats.misses;
        if total == 0 {
            0.0
        } else {
            stats.hits as f64 / total as f64
        }
    }
    
    /// 检查是否包含指定查询
    pub fn contains(&self, query: &Query) -> bool {
        let key = CacheKey::from_query(query);
        let now = Instant::now();
        
        if let Ok(cache) = self.cache.read() {
            if let Some(entry) = cache.get(&key) {
                return now < entry.expires_at;
            }
        }
        
        false
    }
    
    /// 移除指定查询的缓存
    pub fn remove(&self, query: &Query) -> bool {
        let key = CacheKey::from_query(query);
        
        if let Ok(mut cache) = self.cache.write() {
            let removed = cache.remove(&key).is_some();
            
            if removed {
                // 更新统计
                if let Ok(mut stats) = self.stats.write() {
                    stats.current_size = cache.len();
                }
            }
            
            return removed;
        }
        
        false
    }
    
    /// 获取所有缓存的查询
    pub fn get_cached_queries(&self) -> Vec<Query> {
        if let Ok(cache) = self.cache.read() {
            cache.keys().map(|key| Query {
                name: key.name.clone(),
                qtype: key.qtype.into(),
                qclass: key.qclass.into(),
            }).collect()
        } else {
            Vec::new()
        }
    }
    
    /// 设置最大TTL
    pub fn set_max_ttl(&mut self, max_ttl: Duration) {
        self.max_ttl = max_ttl;
    }
    
    /// 获取最大TTL
    pub fn max_ttl(&self) -> Duration {
        self.max_ttl
    }
}

/// 缓存清理任务
pub struct CacheCleanupTask {
    cache: Arc<DnsCache>,
    interval: Duration,
}

impl CacheCleanupTask {
    /// 创建新的清理任务
    pub fn new(cache: Arc<DnsCache>, interval: Duration) -> Self {
        Self { cache, interval }
    }
    
    /// 启动清理任务
    pub async fn start(self) {
        let mut interval_timer = tokio::time::interval(self.interval);
        
        loop {
            interval_timer.tick().await;
            self.cache.cleanup_expired();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use std::net::Ipv4Addr;
    
    fn create_test_query() -> Query {
        Query {
            name: "example.com".to_string(),
            qtype: RecordType::A,
            qclass: QClass::IN,
        }
    }
    
    fn create_test_response() -> Response {
        Response {
            id: 12345,
            flags: Flags::default(),
            queries: vec![create_test_query()],
            answers: vec![Record {
                name: "example.com".to_string(),
                rtype: RecordType::A,
                class: QClass::IN,
                ttl: 300,
                data: RecordData::A(Ipv4Addr::new(93, 184, 216, 34)),
            }],
            authorities: vec![],
            additionals: vec![],
        }
    }
    
    #[test]
    fn test_cache_insert_and_get() {
        let cache = DnsCache::new(Duration::from_secs(3600));
        let query = create_test_query();
        let response = create_test_response();
        
        // 插入缓存
        cache.insert(query.clone(), response.clone());
        
        // 获取缓存
        let cached_response = cache.get(&query);
        assert!(cached_response.is_some());
        
        let cached = cached_response.unwrap();
        assert_eq!(cached.answers.len(), 1);
        assert_eq!(cached.answers[0].name, "example.com");
    }
    
    #[test]
    fn test_cache_expiration() {
        let cache = DnsCache::new(Duration::from_millis(100));
        let query = create_test_query();
        let mut response = create_test_response();
        response.answers[0].ttl = 0; // 立即过期
        
        cache.insert(query.clone(), response);
        
        // 应该没有缓存（TTL为0）
        let cached_response = cache.get(&query);
        assert!(cached_response.is_none());
    }
}