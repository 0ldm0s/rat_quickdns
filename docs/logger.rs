//! 基于 zerg_creep 的日志系统
//! 提供统一的日志接口和配置

use zerg_creep::logger::{Level, LevelFilter};
use zerg_creep::logger::builder::LoggerBuilder;
use zerg_creep::logger::config::{FileConfig, NetworkConfig};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use chrono::Local;

// 重新导出 zerg_creep 的日志宏
pub use zerg_creep::{error, warn, info, debug, trace};

/// 日志级别映射
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => Level::Error,
            LogLevel::Warn => Level::Warn,
            LogLevel::Info => Level::Info,
            LogLevel::Debug => Level::Debug,
            LogLevel::Trace => Level::Trace,
        }
    }
}

impl From<LogLevel> for LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Trace => LevelFilter::Trace,
        }
    }
}

/// 日志输出类型
#[derive(Debug, Clone)]
pub enum LogOutput {
    /// 终端输出
    Terminal,
    /// 文件输出
    File {
        log_dir: PathBuf,
        max_file_size: u64,
        max_compressed_files: u32,
    },
    /// UDP网络输出
    Udp {
        server_addr: String,
        server_port: u16,
        auth_token: String,
        app_id: String,
    },
}

/// 日志配置
#[derive(Debug, Clone)]
pub struct LogConfig {
    pub enabled: bool,
    pub level: LogLevel,
    pub output: LogOutput,
    pub use_colors: bool,
    pub use_emoji: bool,
    pub show_timestamp: bool,
    pub show_module: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        LogConfig {
            enabled: true,
            level: LogLevel::Info,
            output: LogOutput::Terminal,
            use_colors: true,
            use_emoji: true,
            show_timestamp: true,
            show_module: true,
        }
    }
}

impl LogConfig {
    /// 创建禁用日志的配置
    pub fn disabled() -> Self {
        LogConfig {
            enabled: false,
            ..Default::default()
        }
    }
    
    /// 创建文件日志配置
    pub fn file<P: Into<PathBuf>>(log_dir: P) -> Self {
        LogConfig {
            enabled: true,
            level: LogLevel::Info,
            output: LogOutput::File {
                log_dir: log_dir.into(),
                max_file_size: 10 * 1024 * 1024, // 10MB
                max_compressed_files: 5,
            },
            use_colors: false, // 文件日志不使用颜色
            use_emoji: false,  // 文件日志不使用emoji
            show_timestamp: true,
            show_module: true,
        }
    }
    
    /// 创建UDP日志配置
    pub fn udp(server_addr: String, server_port: u16, auth_token: String, app_id: String) -> Self {
        LogConfig {
            enabled: true,
            level: LogLevel::Info,
            output: LogOutput::Udp {
                server_addr,
                server_port,
                auth_token,
                app_id,
            },
            use_colors: false, // UDP日志不使用颜色
            use_emoji: false,  // UDP日志不使用emoji
            show_timestamp: true,
            show_module: true,
        }
    }
}

/// 终端彩色格式化函数
fn rat_engine_format(
    buf: &mut dyn Write,
    record: &zerg_creep::logger::Record
) -> std::io::Result<()> {
    let level = record.metadata.level;
    
    // RAT Engine 主题配色方案
    let (level_color, level_bg, level_icon) = match level {
        Level::Error => ("\x1b[97m", "\x1b[41m", "❌"), // 白字红底
        Level::Warn => ("\x1b[30m", "\x1b[43m", "⚠️ "), // 黑字黄底
        Level::Info => ("\x1b[97m", "\x1b[44m", "🚀"), // 白字蓝底
        Level::Debug => ("\x1b[30m", "\x1b[46m", "🔧"), // 黑字青底
        Level::Trace => ("\x1b[97m", "\x1b[45m", "🔍"), // 白字紫底
    };
    
    // 时间戳颜色
    let timestamp_color = "\x1b[90m"; // 灰色
    let message_color = "\x1b[37m";   // 亮白色
    let reset = "\x1b[0m";
    
    // 获取当前时间
    let now = Local::now();
    let timestamp = now.format("%H:%M:%S%.3f");
    
    writeln!(
        buf,
        "{}{} {}{}{:5}{} {} {}{}{}",
        timestamp_color, timestamp,        // 时间戳
        level_color, level_bg, level, reset, // 彩色级别标签
        level_icon,                        // 级别图标
        message_color, record.args, reset  // 消息内容
    )
}

/// 文件格式化函数（无颜色）
fn file_format(
    buf: &mut dyn Write,
    record: &zerg_creep::logger::Record
) -> std::io::Result<()> {
    let now = Local::now();
    let timestamp = now.format("%Y-%m-%d %H:%M:%S%.3f");
    
    writeln!(
        buf,
        "[{}] [{}] [RAT-Engine] {}",
        timestamp,
        record.metadata.level,
        record.args
    )
}

/// UDP格式化函数（简洁格式）
fn udp_format(
    buf: &mut dyn Write,
    record: &zerg_creep::logger::Record
) -> std::io::Result<()> {
    let now = Local::now();
    let timestamp = now.format("%H:%M:%S%.3f");
    
    writeln!(
        buf,
        "[{}] {} {}",
        timestamp,
        record.metadata.level,
        record.args
    )
}

/// 日志器结构
pub struct Logger;

impl Logger {
    /// 初始化日志系统
    pub fn init(config: LogConfig) -> Result<(), Box<dyn std::error::Error>> {
        // 如果日志被禁用，直接返回
        if !config.enabled {
            return Ok(());
        }
        
        let mut builder = LoggerBuilder::new();
        builder.filter(LevelFilter::from(config.level));

        match &config.output {
            LogOutput::Terminal => {
                builder.format(rat_engine_format);
            }
            LogOutput::File { log_dir, max_file_size, max_compressed_files } => {
                let mut file_config = FileConfig::new();
                file_config.log_dir = log_dir.clone();
                file_config.max_file_size = *max_file_size;
                file_config.max_compressed_files = *max_compressed_files as usize;
                
                builder.format(file_format);
                builder.with_file_logging(file_config);
            }
            LogOutput::Udp { server_addr, server_port, auth_token, app_id } => {
                let network_config = NetworkConfig {
                    server_addr: server_addr.clone(),
                    server_port: *server_port,
                    auth_token: Arc::from(auth_token.as_str()),
                    app_id: app_id.clone(),
                };
                
                builder.format(udp_format);
                builder.with_udp_logging(network_config);
            }
        }
        
        match builder.try_init() {
            Ok(_) => Ok(()),
            Err(e) => {
                // 如果已经初始化过了，这是正常的
                eprintln!("Logger init warning: {}", e);
                Ok(())
            }
        }
    }
    
    /// 使用默认配置初始化
    pub fn init_default() -> Result<(), Box<dyn std::error::Error>> {
        Self::init(LogConfig::default())
    }
}

/// 便捷的初始化函数
pub fn init_logger() -> Result<(), Box<dyn std::error::Error>> {
    Logger::init_default()
}

/// 带配置的初始化函数
pub fn init_logger_with_config(config: LogConfig) -> Result<(), Box<dyn std::error::Error>> {
    Logger::init(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_logger_init() {
        // 测试logger初始化，允许重复初始化失败
        let result = Logger::init_default();
        // 无论成功还是失败都是可接受的，因为可能已经初始化过了
        match result {
            Ok(_) => println!("Logger initialized successfully"),
            Err(_) => println!("Logger already initialized"),
        }
    }
    
    #[test]
    fn test_log_levels() {
        let _ = Logger::init_default();
        
        error!("Test error message");
        warn!("Test warning message");
        info!("Test info message");
        debug!("Test debug message");
        trace!("Test trace message");
    }
}