//! RatQuickDNS - 高性能DNS查询库
//!
//! 提供UDP/TCP/DOH/DOT多协议支持、智能决策、缓存、健康检查和客户端IP转发(EDNS Client Subnet)功能

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(unsafe_code)]

pub mod types;
pub mod transport;
pub mod resolver;
pub mod error;

pub use types::*;
pub use transport::Transport;
pub use resolver::Resolver;
pub use error::{DnsError, Result};