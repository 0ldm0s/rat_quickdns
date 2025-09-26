//! Python日志初始化模块
//!
//! 这个模块提供了Python绑定的日志初始化功能，支持调用者控制模式。

use pyo3::prelude::*;
use rat_logger::{LevelFilter, LoggerBuilder, handler::term::TermConfig, config::FormatConfig};
use crate::logger;
use std::sync::atomic::{AtomicBool, Ordering};

/// 全局日志初始化状态标志
static LOGGING_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// 检查日志系统是否已初始化
#[pyfunction]
fn is_logging_initialized() -> bool {
    LOGGING_INITIALIZED.load(Ordering::SeqCst)
}

/// 基本日志初始化 - 使用默认配置
#[pyfunction]
fn init_logging() -> PyResult<()> {
    // 防止重复初始化
    if LOGGING_INITIALIZED.load(Ordering::SeqCst) {
        return Ok(());
    }

    // 使用rat_logger的默认配置
    LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .add_terminal_with_config(TermConfig::default())
        .init_global_logger()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("日志初始化失败: {}", e)))?;

    LOGGING_INITIALIZED.store(true, Ordering::SeqCst);
    Ok(())
}

/// 指定日志级别的初始化
#[pyfunction]
fn init_logging_with_level(level: &str) -> PyResult<()> {
    // 防止重复初始化
    if LOGGING_INITIALIZED.load(Ordering::SeqCst) {
        return Ok(());
    }

    // 解析日志级别
    let level_filter = match level.to_lowercase().as_str() {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        "warn" | "warning" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        "off" => LevelFilter::Off,
        _ => return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("无效的日志级别: {}. 支持的级别: trace, debug, info, warn, error, off", level)
        )),
    };

    LoggerBuilder::new()
        .with_level(level_filter)
        .add_terminal_with_config(TermConfig::default())
        .init_global_logger()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("日志初始化失败: {}", e)))?;

    LOGGING_INITIALIZED.store(true, Ordering::SeqCst);
    Ok(())
}

/// 高级日志初始化 - 完全自定义配置
#[pyfunction]
fn init_logging_advanced(
    level: Option<&str>,
    enable_color: Option<bool>,
    timestamp_format: Option<&str>,
    custom_format_template: Option<&str>,
) -> PyResult<()> {
    // 防止重复初始化
    if LOGGING_INITIALIZED.load(Ordering::SeqCst) {
        return Ok(());
    }

    let mut builder = LoggerBuilder::new();

    // 设置日志级别
    if let Some(level_str) = level {
        let level_filter = match level_str.to_lowercase().as_str() {
            "trace" => LevelFilter::Trace,
            "debug" => LevelFilter::Debug,
            "info" => LevelFilter::Info,
            "warn" | "warning" => LevelFilter::Warn,
            "error" => LevelFilter::Error,
            "off" => LevelFilter::Off,
            _ => return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("无效的日志级别: {}. 支持的级别: trace, debug, info, warn, error, off", level_str)
            )),
        };
        builder = builder.with_level(level_filter);
    } else {
        builder = builder.with_level(LevelFilter::Info);
    }

    // 设置终端配置
    let mut term_config = TermConfig::default();
    if let Some(enable_color) = enable_color {
        term_config.enable_color = enable_color;
    }

    // 创建格式配置
    let mut format_config = FormatConfig::default();
    if let Some(format) = timestamp_format {
        format_config.timestamp_format = format.to_string();
    }
    if let Some(template) = custom_format_template {
        format_config.format_template = template.to_string();
    }

    term_config.format = Some(format_config);

    builder = builder.add_terminal_with_config(term_config);

    builder.init_global_logger()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("日志初始化失败: {}", e)))?;

    LOGGING_INITIALIZED.store(true, Ordering::SeqCst);
    Ok(())
}

/// DNS专用的日志初始化函数
#[pyfunction]
fn init_dns_logging(level: &str) -> PyResult<()> {
    // 防止重复初始化
    if LOGGING_INITIALIZED.load(Ordering::SeqCst) {
        return Ok(());
    }

    // 解析日志级别
    let level_filter = match level.to_lowercase().as_str() {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        "warn" | "warning" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        "off" => LevelFilter::Off,
        _ => return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("无效的日志级别: {}. 支持的级别: trace, debug, info, warn, error, off", level)
        )),
    };

    // 首先初始化基础日志系统
    LoggerBuilder::new()
        .with_level(level_filter)
        .add_terminal_with_config(TermConfig::default())
        .init_global_logger()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("日志初始化失败: {}", e)))?;

    // 然后初始化DNS专用日志格式
    logger::init_dns_logger(level_filter)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("DNS日志初始化失败: {}", e)))?;

    LOGGING_INITIALIZED.store(true, Ordering::SeqCst);
    Ok(())
}

/// DNS专用的高级日志初始化
#[pyfunction]
fn init_dns_logging_advanced(
    level: &str,
    enable_dns_format: Option<bool>,
    enable_color: Option<bool>,
    timestamp_format: Option<&str>,
) -> PyResult<()> {
    // 防止重复初始化
    if LOGGING_INITIALIZED.load(Ordering::SeqCst) {
        return Ok(());
    }

    // 解析日志级别
    let level_filter = match level.to_lowercase().as_str() {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        "warn" | "warning" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        "off" => LevelFilter::Off,
        _ => return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("无效的日志级别: {}. 支持的级别: trace, debug, info, warn, error, off", level)
        )),
    };

    // 设置终端配置
    let mut term_config = TermConfig::default();
    if let Some(enable_color) = enable_color {
        term_config.enable_color = enable_color;
    }

    // 创建格式配置
    let mut format_config = FormatConfig::default();
    if let Some(format) = timestamp_format {
        format_config.timestamp_format = format.to_string();
    }

    term_config.format = Some(format_config);

    // 首先初始化基础日志系统
    LoggerBuilder::new()
        .with_level(level_filter)
        .add_terminal_with_config(term_config)
        .init_global_logger()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("日志初始化失败: {}", e)))?;

    // 然后初始化DNS专用日志格式
    if enable_dns_format.unwrap_or(true) {
        logger::init_dns_logger(level_filter)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("DNS日志初始化失败: {}", e)))?;
    }

    LOGGING_INITIALIZED.store(true, Ordering::SeqCst);
    Ok(())
}

/// DNS日志宏 - 信息级别
#[pyfunction]
fn dns_info(message: &str) {
    if LOGGING_INITIALIZED.load(Ordering::SeqCst) {
        rat_logger::info!("DNS_INFO: {}", message);
    }
}

/// DNS日志宏 - 警告级别
#[pyfunction]
fn dns_warn(message: &str) {
    if LOGGING_INITIALIZED.load(Ordering::SeqCst) {
        rat_logger::warn!("DNS_WARN: {}", message);
    }
}

/// DNS日志宏 - 错误级别
#[pyfunction]
fn dns_error(message: &str) {
    if LOGGING_INITIALIZED.load(Ordering::SeqCst) {
        rat_logger::error!("DNS_ERROR: {}", message);
    }
}

/// DNS日志宏 - 调试级别
#[pyfunction]
fn dns_debug(message: &str) {
    if LOGGING_INITIALIZED.load(Ordering::SeqCst) {
        rat_logger::debug!("DNS_DEBUG: {}", message);
    }
}

/// DNS日志宏 - 跟踪级别
#[pyfunction]
fn dns_trace(message: &str) {
    if LOGGING_INITIALIZED.load(Ordering::SeqCst) {
        rat_logger::trace!("DNS_TRACE: {}", message);
    }
}

/// 通用日志宏 - 信息级别
#[pyfunction]
fn log_info(message: &str) {
    if LOGGING_INITIALIZED.load(Ordering::SeqCst) {
        rat_logger::info!("{}", message);
    }
}

/// 通用日志宏 - 警告级别
#[pyfunction]
fn log_warn(message: &str) {
    if LOGGING_INITIALIZED.load(Ordering::SeqCst) {
        rat_logger::warn!("{}", message);
    }
}

/// 通用日志宏 - 错误级别
#[pyfunction]
fn log_error(message: &str) {
    if LOGGING_INITIALIZED.load(Ordering::SeqCst) {
        rat_logger::error!("{}", message);
    }
}

/// 通用日志宏 - 调试级别
#[pyfunction]
fn log_debug(message: &str) {
    if LOGGING_INITIALIZED.load(Ordering::SeqCst) {
        rat_logger::debug!("{}", message);
    }
}

/// 通用日志宏 - 跟踪级别
#[pyfunction]
fn log_trace(message: &str) {
    if LOGGING_INITIALIZED.load(Ordering::SeqCst) {
        rat_logger::trace!("{}", message);
    }
}

pub fn init_logging_module(m: &PyModule) -> PyResult<()> {
    // 添加日志初始化函数
    m.add_function(wrap_pyfunction!(is_logging_initialized, m)?)?;
    m.add_function(wrap_pyfunction!(init_logging, m)?)?;
    m.add_function(wrap_pyfunction!(init_logging_with_level, m)?)?;
    m.add_function(wrap_pyfunction!(init_logging_advanced, m)?)?;
    m.add_function(wrap_pyfunction!(init_dns_logging, m)?)?;
    m.add_function(wrap_pyfunction!(init_dns_logging_advanced, m)?)?;

    // 添加DNS日志函数
    m.add_function(wrap_pyfunction!(dns_info, m)?)?;
    m.add_function(wrap_pyfunction!(dns_warn, m)?)?;
    m.add_function(wrap_pyfunction!(dns_error, m)?)?;
    m.add_function(wrap_pyfunction!(dns_debug, m)?)?;
    m.add_function(wrap_pyfunction!(dns_trace, m)?)?;

    // 添加通用日志函数
    m.add_function(wrap_pyfunction!(log_info, m)?)?;
    m.add_function(wrap_pyfunction!(log_warn, m)?)?;
    m.add_function(wrap_pyfunction!(log_error, m)?)?;
    m.add_function(wrap_pyfunction!(log_debug, m)?)?;
    m.add_function(wrap_pyfunction!(log_trace, m)?)?;

    Ok(())
}