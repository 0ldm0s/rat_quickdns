//! Python API bindings for rat_quickdns
//! 
//! 这个模块提供了rat_quickdns库的Python绑定，使用PyO3实现。
//! 支持异步DNS解析、多种查询策略、EDNS功能等。

use pyo3::prelude::*;

pub mod resolver;
pub mod builder;
pub mod types;
pub mod utils;

use resolver::PyDnsResolver;
use builder::PyDnsResolverBuilder;
use types::{PyQueryStrategy, PyDnsResult, PyDnsRecordType, PyTransportType};

/// Python模块初始化函数
pub fn init_python_module(_py: Python, m: &PyModule) -> pyo3::PyResult<()> {
    // 添加类
    m.add_class::<PyDnsResolver>()?;
    m.add_class::<PyDnsResolverBuilder>()?;
    m.add_class::<PyQueryStrategy>()?;
    m.add_class::<PyDnsResult>()?;
    m.add_class::<PyTransportType>()?;
    m.add_class::<PyDnsRecordType>()?;
    
    // 保持架构纯净性，只暴露核心构建器类
    
    // 添加版本信息
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("__author__", "0ldm0s <oldmos@gmail.com>")?;
    
    Ok(())
}