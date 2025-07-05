//! 严格配置模块 - 移除所有兜底默认值
//! 
//! 这个模块实现了严格的配置验证，强制用户明确指定所有配置参数，
//! 不提供任何"贴心"的默认值或自动修复功能。

pub mod strict;

pub use strict::{
    StrictDnsConfig,
    StrictConfigBuilder,
    ConfigError,
};