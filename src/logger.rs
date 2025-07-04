//! DNS 查询器专用日志系统
//! 
//! 基于 zerg_creep 高性能日志库，提供适合 DNS 查询场景的彩色日志输出

use zerg_creep::logger::{Level, LevelFilter};
use zerg_creep::logger::builder::LoggerBuilder;
use std::io::Write;
use std::sync::Once;
use chrono::Local;

// 重新导出 zerg_creep 的日志宏
pub use zerg_creep::{error, warn, info, debug, trace};

/// 确保日志器只初始化一次
static INIT: Once = Once::new();

/// DNS 查询专用日志格式化器
pub fn dns_format(
    buf: &mut dyn Write,
    record: &zerg_creep::logger::Record
) -> std::io::Result<()> {
    let level = record.metadata.level;
    
    // DNS 主题配色方案
    let (level_color, level_bg, level_icon) = match level {
        Level::Error => ("\x1b[97m", "\x1b[41m", "🚫"), // 白字红底 - DNS 错误
        Level::Warn => ("\x1b[30m", "\x1b[43m", "⚠️ "), // 黑字黄底 - DNS 警告
        Level::Info => ("\x1b[97m", "\x1b[42m", "🌐"), // 白字绿底 - DNS 查询
        Level::Debug => ("\x1b[30m", "\x1b[46m", "🔍"), // 黑字青底 - DNS 调试
        Level::Trace => ("\x1b[97m", "\x1b[45m", "📡"), // 白字紫底 - DNS 追踪
    };
    
    // 颜色定义
    let timestamp_color = "\x1b[90m"; // 灰色时间戳
    let message_color = "\x1b[37m";   // 亮白色消息
    let reset = "\x1b[0m";
    
    // 获取当前时间
    let now = Local::now();
    let timestamp = now.format("%H:%M:%S%.3f");
    
    writeln!(
        buf,
        "{}{} {}{}{:5}{} {} {}{}{}",
        timestamp_color, timestamp,        // 时间戳
        level_color, level_bg, level, reset, // 彩色级别标签
        level_icon,                        // DNS 相关图标
        message_color, record.args, reset  // 消息内容
    )
}

/// 初始化 DNS 查询器日志系统（线程安全，防止重复初始化）
/// 
/// # Arguments
/// * `level` - 日志级别过滤器，传入 `LevelFilter::Off` 可禁用日志
/// 
/// # Example
/// ```
/// use rat_quickdns::logger::init_dns_logger;
/// use zerg_creep::logger::LevelFilter;
/// 
/// // 启用Info级别日志
/// init_dns_logger(LevelFilter::Info).unwrap();
/// 
/// // 禁用所有日志
/// init_dns_logger(LevelFilter::Off).unwrap();
/// ```
pub fn init_dns_logger(level: LevelFilter) -> Result<(), Box<dyn std::error::Error>> {
    let mut init_result = Ok(());
    
    INIT.call_once(|| {
        // 只在第一次调用时初始化日志器
        LoggerBuilder::new()
            .filter(level)
            .format(dns_format)
            .init();
    });
    
    init_result
}

/// 安全的日志初始化函数，默认禁用日志输出
/// 
/// 这个函数专门用于构造器，确保默认情况下不输出日志
pub fn init_dns_logger_silent() -> Result<(), Box<dyn std::error::Error>> {
    init_dns_logger(LevelFilter::Off)
}

/// DNS 查询相关的便捷日志宏
#[macro_export]
macro_rules! dns_query {
    ($domain:expr, $record_type:expr) => {
        $crate::logger::info!("🔍 查询 {} 记录: {}", $record_type, $domain);
    };
}

#[macro_export]
macro_rules! dns_response {
    ($domain:expr, $count:expr, $duration:expr) => {
        $crate::logger::info!("✅ {} 响应: {} 条记录 ({}ms)", $domain, $count, $duration);
    };
}

#[macro_export]
macro_rules! dns_error {
    ($($arg:tt)*) => {
        $crate::error!($($arg)*);
    };
}

#[macro_export]
macro_rules! dns_debug {
    ($($arg:tt)*) => {
        $crate::debug!($($arg)*);
    };
}

#[macro_export]
macro_rules! dns_info {
    ($($arg:tt)*) => {
        $crate::info!($($arg)*);
    };
}

#[macro_export]
macro_rules! dns_transport {
    ($($arg:tt)*) => {
        $crate::info!($($arg)*);
    };
}

#[macro_export]
macro_rules! dns_timeout {
    ($domain:expr, $timeout:expr) => {
        $crate::logger::warn!("⏰ {} 查询超时: {}ms", $domain, $timeout);
    };
}

#[macro_export]
macro_rules! dns_cache_hit {
    ($domain:expr) => {
        $crate::logger::debug!("💾 缓存命中: {}", $domain);
    };
}

#[macro_export]
macro_rules! dns_cache_miss {
    ($domain:expr) => {
        $crate::logger::debug!("🔄 缓存未命中: {}", $domain);
    };
}

#[macro_export]
macro_rules! dns_upstream {
    ($server:expr, $domain:expr) => {
        $crate::logger::trace!("📡 上游服务器 {} 查询: {}", $server, $domain);
    };
}

#[macro_export]
macro_rules! dns_strategy {
    ($strategy:expr, $domain:expr) => {
        $crate::logger::debug!("🎯 使用策略 {} 查询: {}", $strategy, $domain);
    };
}
