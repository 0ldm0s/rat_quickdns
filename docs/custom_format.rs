use rat_logger::{error, warn, info, debug, trace};
use rat_logger::{Level, LevelFilter};
use rat_logger::LoggerBuilder;
use std::io::Write;
use chrono::Local;

fn themed_format(
    buf: &mut dyn Write,
    record: &rat_logger::config::Record
) -> std::io::Result<()> {
    let level = record.metadata.level;
    
    // 主题配色方案
    let (level_color, level_bg, level_icon) = match level {
        Level::Error => ("\x1b[97m", "\x1b[41m", "❌"), // 白字红底
        Level::Warn => ("\x1b[30m", "\x1b[43m", "⚠️ "), // 黑字黄底
        Level::Info => ("\x1b[97m", "\x1b[44m", "ℹ️ "), // 白字蓝底
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

fn main() {
    let _ = LoggerBuilder::new()
        .with_level(LevelFilter::Trace) // 显示所有级别日志
        .add_terminal_with_config(rat_logger::handler::term::TermConfig::default())
        .init_global_logger();

    // 使用宏输出不同级别的日志
    error!("这是一个错误消息");
    warn!("这是一个警告消息");
    info!("这是一个信息消息");
    debug!("这是一个调试消息");
    trace!("这是一个追踪消息");

    // 多线程测试
    std::thread::spawn(|| {
        info!("来自另一个线程的日志");
    }).join().unwrap();
}