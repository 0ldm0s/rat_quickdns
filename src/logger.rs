//! DNS 查询器专用日志系统
//!
//! 基于 rat_logger 高性能日志库，提供适合 DNS 查询场景的彩色日志输出
//!
//! # 调用者初始化逻辑
//!
//! 这个模块遵循调用者初始化模式，用户必须先初始化rat_logger日志系统，
//! 然后才能使用DNS日志功能。
//!
//! ## 推荐的初始化流程
//!
//! 调用者需要先初始化rat_logger系统，然后才能使用DNS日志功能。

use rat_logger::{Level, LevelFilter};
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use chrono::Local;

/// 确保日志器只初始化一次
static INIT: std::sync::Once = std::sync::Once::new();

/// 日志初始化状态标志（线程安全）
static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// DNS 查询专用日志格式化器
pub fn dns_format(
    buf: &mut dyn std::io::Write,
    record: &rat_logger::config::Record
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
/// **注意**: 这个函数现在使用调用者初始化模式。调用者必须先初始化rat_logger日志系统。
///
/// # Arguments
/// * `level` - 日志级别过滤器，传入 `LevelFilter::Off` 可禁用日志
///
/// # Example
/// 使用方法：
/// 1. 首先初始化rat_logger系统（调用者责任）
/// 2. 然后调用此函数初始化DNS日志格式
pub fn init_dns_logger(level: LevelFilter) -> Result<(), Box<dyn std::error::Error>> {
    // 检查rat_logger是否已经初始化
    // 注意：这里我们只是设置一个标志，实际的日志系统初始化由调用者负责
    INIT.call_once(|| {
        INITIALIZED.store(true, Ordering::SeqCst);
    });

    // 设置rat_logger的全局日志级别
    rat_logger::core::set_max_level(level);

    Ok(())
}

/// 安全的日志初始化函数，默认禁用日志输出
///
/// 这个函数专门用于构造器，确保默认情况下不输出日志
pub fn init_dns_logger_silent() -> Result<(), Box<dyn std::error::Error>> {
    init_dns_logger(LevelFilter::Off)
}

/// 检查DNS日志系统是否已初始化
pub fn is_dns_logger_initialized() -> bool {
    INITIALIZED.load(Ordering::SeqCst)
}

/// DNS 查询相关的便捷日志宏
#[macro_export]
macro_rules! dns_query {
    ($domain:expr, $record_type:expr) => {
        $crate::info!("🔍 查询 {} 记录: {}", $record_type, $domain);
    };
}

#[macro_export]
macro_rules! dns_response {
    ($domain:expr, $count:expr, $duration:expr) => {
        $crate::info!("✅ {} 响应: {} 条记录 ({}ms)", $domain, $count, $duration);
    };
}

#[macro_export]
macro_rules! dns_error {
    ($($arg:tt)*) => {
        $crate::error!($($arg)*)
    };
}

#[macro_export]
macro_rules! dns_debug {
    ($($arg:tt)*) => {
        $crate::debug!($($arg)*)
    };
}

#[macro_export]
macro_rules! dns_info {
    ($($arg:tt)*) => {
        $crate::info!($($arg)*)
    };
}

#[macro_export]
macro_rules! dns_warn {
    ($($arg:tt)*) => {
        $crate::warn!($($arg)*)
    };
}

#[macro_export]
macro_rules! dns_transport {
    ($($arg:tt)*) => {
        $crate::info!($($arg)*)
    };
}

#[macro_export]
macro_rules! dns_timeout {
    ($domain:expr, $timeout:expr) => {
        $crate::warn!("⏰ {} 查询超时: {}ms", $domain, $timeout);
    };
}

#[macro_export]
macro_rules! dns_cache_hit {
    ($domain:expr) => {
        $crate::debug!("💾 缓存命中: {}", $domain);
    };
}

#[macro_export]
macro_rules! dns_cache_miss {
    ($domain:expr) => {
        $crate::debug!("🔄 缓存未命中: {}", $domain);
    };
}

#[macro_export]
macro_rules! dns_upstream {
    ($server:expr, $domain:expr) => {
        $crate::trace!("📡 上游服务器 {} 查询: {}", $server, $domain);
    };
}

#[macro_export]
macro_rules! dns_strategy {
    ($strategy:expr, $domain:expr) => {
        $crate::debug!("🎯 使用策略 {} 查询: {}", $strategy, $domain);
    };
}