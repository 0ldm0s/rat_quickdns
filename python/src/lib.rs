//! rat-quickdns-py: Python bindings for rat_quickdns
//! 
//! 这个crate提供了rat_quickdns库的Python绑定，使用PyO3实现。
//! 支持异步DNS解析、多种查询策略、EDNS功能等。

use pyo3::prelude::*;

// 重新导出rat_quickdns的Python API模块
use rat_quickdns::python_api::*;

/// Python模块定义
#[pymodule]
fn rat_quickdns_py(py: Python, m: &PyModule) -> PyResult<()> {
    // 调用主模块的初始化函数
    rat_quickdns::python_api::init_python_module(py, m)?;
    
    // 便捷函数已被移除，请使用构建器模式创建解析器
    
    // 添加工具函数
    m.add_function(wrap_pyfunction!(utils::is_valid_ip, m)?)?;
    m.add_function(wrap_pyfunction!(utils::is_valid_ipv4, m)?)?;
    m.add_function(wrap_pyfunction!(utils::is_valid_ipv6, m)?)?;
    m.add_function(wrap_pyfunction!(utils::is_valid_domain, m)?)?;
    m.add_function(wrap_pyfunction!(utils::is_valid_socket_addr, m)?)?;
    m.add_function(wrap_pyfunction!(utils::parse_socket_addr, m)?)?;
    m.add_function(wrap_pyfunction!(utils::format_duration, m)?)?;
    m.add_function(wrap_pyfunction!(utils::get_default_dns_servers, m)?)?;
    m.add_function(wrap_pyfunction!(utils::get_default_doh_servers, m)?)?;
    m.add_function(wrap_pyfunction!(utils::get_default_dot_servers, m)?)?;
    m.add_function(wrap_pyfunction!(utils::create_preset_builder, m)?)?;
    
    // 添加模块属性
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
        
    Ok(())
}