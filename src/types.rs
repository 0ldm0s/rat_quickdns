//! DNS核心类型定义

use std::fmt;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// DNS查询请求
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    /// 事务ID
    pub id: u16,
    /// 标志位
    pub flags: Flags,
    /// 查询问题
    pub query: Query,
    /// 客户端地址信息 (EDNS Client Subnet)
    pub client_address: Option<ClientAddress>,
}

/// DNS响应
#[derive(Debug, Clone, PartialEq)]
pub struct Response {
    /// 事务ID
    pub id: u16,
    /// 标志位
    pub flags: Flags,
    /// 查询问题
    pub queries: Vec<Query>,
    /// 回答记录
    pub answers: Vec<Record>,
    /// 权威记录
    pub authorities: Vec<Record>,
    /// 附加记录
    pub additionals: Vec<Record>,
}

/// DNS查询问题
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Query {
    /// 查询名称
    pub name: String,
    /// 查询类型
    pub qtype: RecordType,
    /// 查询类别
    pub qclass: QClass,
}

/// DNS资源记录
#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    /// 记录名称
    pub name: String,
    /// 记录类型
    pub rtype: RecordType,
    /// 记录类别
    pub class: QClass,
    /// 生存时间(秒)
    pub ttl: u32,
    /// 记录数据
    pub data: RecordData,
}

/// DNS记录数据
#[derive(Debug, Clone, PartialEq)]
pub enum RecordData {
    /// A记录 - IPv4地址
    A(Ipv4Addr),
    /// AAAA记录 - IPv6地址
    AAAA(Ipv6Addr),
    /// CNAME记录 - 别名
    CNAME(String),
    /// MX记录 - 邮件交换
    ///
    /// # 字段
    /// - `priority`: 优先级，数值越小优先级越高
    /// - `exchange`: 邮件服务器域名
    MX {
        /// 优先级，数值越小优先级越高
        priority: u16,
        /// 邮件服务器域名
        exchange: String
    },
    /// NS记录 - 名称服务器
    NS(String),
    /// PTR记录 - 指针
    PTR(String),
    /// SOA记录 - 授权开始
    ///
    /// # 字段
    /// - `mname`: 主名称服务器
    /// - `rname`: 管理员邮箱
    /// - `serial`: 序列号
    /// - `refresh`: 刷新间隔（秒）
    /// - `retry`: 重试间隔（秒）
    /// - `expire`: 过期时间（秒）
    /// - `minimum`: 最小TTL（秒）
    SOA {
        /// 主名称服务器
        mname: String,
        /// 管理员邮箱
        rname: String,
        /// 序列号
        serial: u32,
        /// 刷新间隔（秒）
        refresh: u32,
        /// 重试间隔（秒）
        retry: u32,
        /// 过期时间（秒）
        expire: u32,
        /// 最小TTL（秒）
        minimum: u32,
    },
    /// TXT记录 - 文本
    TXT(Vec<String>),
    /// SRV记录 - 服务
    ///
    /// # 字段
    /// - `priority`: 优先级，数值越小优先级越高
    /// - `weight`: 权重，用于负载均衡
    /// - `port`: 服务端口号
    /// - `target`: 目标主机名
    SRV {
        /// 优先级，数值越小优先级越高
        priority: u16,
        /// 权重，用于负载均衡
        weight: u16,
        /// 服务端口号
        port: u16,
        /// 目标主机名
        target: String,
    },
    /// 未知记录类型
    Unknown(Vec<u8>),
}

/// DNS记录类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum RecordType {
    /// A记录
    A = 1,
    /// NS记录
    NS = 2,
    /// CNAME记录
    CNAME = 5,
    /// SOA记录
    SOA = 6,
    /// PTR记录
    PTR = 12,
    /// MX记录
    MX = 15,
    /// TXT记录
    TXT = 16,
    /// AAAA记录
    AAAA = 28,
    /// SRV记录
    SRV = 33,
    /// 未知类型
    Unknown(u16),
}

/// DNS查询类别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum QClass {
    /// Internet类别
    IN = 1,
    /// Chaos类别
    CH = 3,
    /// Hesiod类别
    HS = 4,
    /// 任意类别
    ANY = 255,
    /// 未知类别
    Unknown(u16),
}

/// DNS标志位
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Flags {
    /// 查询/响应标志
    pub qr: bool,
    /// 操作码
    pub opcode: u8,
    /// 权威回答
    pub aa: bool,
    /// 截断标志
    pub tc: bool,
    /// 期望递归
    pub rd: bool,
    /// 递归可用
    pub ra: bool,
    /// 保留位
    pub z: u8,
    /// 响应码
    pub rcode: u8,
}

/// DNS响应码
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ResponseCode {
    /// 无错误
    NoError = 0,
    /// 格式错误
    FormatError = 1,
    /// 服务器失败
    ServerFailure = 2,
    /// 域名不存在
    NxDomain = 3,
    /// 未实现
    NotImplemented = 4,
    /// 查询被拒绝
    Refused = 5,
    /// 未知响应码
    Unknown(u8),
}

/// EDNS客户端地址信息
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientAddress {
    /// 客户端IP地址
    pub address: IpAddr,
    /// 源前缀长度
    pub source_prefix_length: u8,
    /// 作用域前缀长度
    pub scope_prefix_length: u8,
}

/// EDNS选项
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdnsOption {
    /// 选项代码
    pub code: u16,
    /// 选项数据
    pub data: Vec<u8>,
}

/// EDNS记录
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdnsRecord {
    /// UDP载荷大小
    pub udp_payload_size: u16,
    /// 扩展RCODE
    pub extended_rcode: u8,
    /// EDNS版本
    pub version: u8,
    /// DO位(DNSSEC OK)
    pub dnssec_ok: bool,
    /// EDNS选项
    pub options: Vec<EdnsOption>,
}

/// EDNS选项代码常量
pub mod edns_option_codes {
    /// Client Address选项代码 - 修正命名，原CLIENT_SUBNET容易误导
    /// 这个选项实际上传递的是客户端地址信息，而不是子网信息
    pub const CLIENT_ADDRESS: u16 = 8;
    /// Cookie选项代码
    pub const COOKIE: u16 = 10;
    /// Keepalive选项代码
    pub const KEEPALIVE: u16 = 11;
    /// Padding选项代码
    pub const PADDING: u16 = 12;
}

impl ClientAddress {
    /// 创建新的客户端地址信息
    pub fn new(address: IpAddr, source_prefix_length: u8) -> Self {
        Self {
            address,
            source_prefix_length,
            scope_prefix_length: 0, // 通常由服务器设置
        }
    }
    
    /// 从IPv4地址创建客户端地址信息
    pub fn from_ipv4(address: Ipv4Addr, prefix_length: u8) -> Self {
        Self::new(IpAddr::V4(address), prefix_length)
    }
    
    /// 从IPv6地址创建客户端地址信息
    pub fn from_ipv6(address: Ipv6Addr, prefix_length: u8) -> Self {
        Self::new(IpAddr::V6(address), prefix_length)
    }
    
    /// 获取地址族代码 (1=IPv4, 2=IPv6)
    pub fn family(&self) -> u16 {
        match self.address {
            IpAddr::V4(_) => 1,
            IpAddr::V6(_) => 2,
        }
    }
    
    /// 将客户端地址信息编码为EDNS选项数据
    pub fn encode(&self) -> Vec<u8> {
        let mut data = Vec::new();
        
        // 地址族 (2字节)
        data.extend_from_slice(&self.family().to_be_bytes());
        
        // 源前缀长度 (1字节)
        data.push(self.source_prefix_length);
        
        // 作用域前缀长度 (1字节)
        data.push(self.scope_prefix_length);
        
        // 地址数据
        match self.address {
            IpAddr::V4(addr) => {
                let bytes = addr.octets();
                let byte_count = (self.source_prefix_length + 7) / 8;
                data.extend_from_slice(&bytes[..byte_count as usize]);
            }
            IpAddr::V6(addr) => {
                let bytes = addr.octets();
                let byte_count = (self.source_prefix_length + 7) / 8;
                data.extend_from_slice(&bytes[..byte_count as usize]);
            }
        }
        
        data
    }
    
    /// 从EDNS选项数据解码客户端地址信息
    pub fn decode(data: &[u8]) -> Result<Self, &'static str> {
        if data.len() < 4 {
            return Err("Client subnet data too short");
        }
        
        let family = u16::from_be_bytes([data[0], data[1]]);
        let source_prefix_length = data[2];
        let scope_prefix_length = data[3];
        
        let address = match family {
            1 => {
                // IPv4
                if data.len() < 4 {
                    return Err("IPv4 client subnet data too short");
                }
                let mut addr_bytes = [0u8; 4];
                let available_bytes = data.len() - 4;
                let copy_bytes = std::cmp::min(4, available_bytes);
                addr_bytes[..copy_bytes].copy_from_slice(&data[4..4 + copy_bytes]);
                IpAddr::V4(Ipv4Addr::from(addr_bytes))
            }
            2 => {
                // IPv6
                if data.len() < 4 {
                    return Err("IPv6 client subnet data too short");
                }
                let mut addr_bytes = [0u8; 16];
                let available_bytes = data.len() - 4;
                let copy_bytes = std::cmp::min(16, available_bytes);
                addr_bytes[..copy_bytes].copy_from_slice(&data[4..4 + copy_bytes]);
                IpAddr::V6(Ipv6Addr::from(addr_bytes))
            }
            _ => return Err("Unsupported address family"),
        };
        
        Ok(Self {
            address,
            source_prefix_length,
            scope_prefix_length,
        })
    }
}

// 注意：移除了 Default 实现，因为它包含兜底行为
// 硬编码的默认值（如 4096 UDP载荷大小）是兜底代码
// 用户现在必须明确配置所有EDNS参数
//
// 迁移示例：
// 旧代码: EdnsRecord::default()
// 新代码: EdnsRecord {
//     udp_payload_size: your_payload_size,
//     extended_rcode: 0,
//     version: 0,
//     dnssec_ok: your_dnssec_preference,
//     options: your_options,
// }

impl From<u16> for RecordType {
    fn from(value: u16) -> Self {
        match value {
            1 => RecordType::A,
            2 => RecordType::NS,
            5 => RecordType::CNAME,
            6 => RecordType::SOA,
            12 => RecordType::PTR,
            15 => RecordType::MX,
            16 => RecordType::TXT,
            28 => RecordType::AAAA,
            33 => RecordType::SRV,
            _ => RecordType::Unknown(value),
        }
    }
}

impl From<RecordType> for u16 {
    fn from(rtype: RecordType) -> Self {
        match rtype {
            RecordType::A => 1,
            RecordType::NS => 2,
            RecordType::CNAME => 5,
            RecordType::SOA => 6,
            RecordType::PTR => 12,
            RecordType::MX => 15,
            RecordType::TXT => 16,
            RecordType::AAAA => 28,
            RecordType::SRV => 33,
            RecordType::Unknown(value) => value,
        }
    }
}

impl From<u16> for QClass {
    fn from(value: u16) -> Self {
        match value {
            1 => QClass::IN,
            3 => QClass::CH,
            4 => QClass::HS,
            255 => QClass::ANY,
            _ => QClass::Unknown(value),
        }
    }
}

impl From<QClass> for u16 {
    fn from(qclass: QClass) -> Self {
        match qclass {
            QClass::IN => 1,
            QClass::CH => 3,
            QClass::HS => 4,
            QClass::ANY => 255,
            QClass::Unknown(value) => value,
        }
    }
}

impl From<u8> for ResponseCode {
    fn from(value: u8) -> Self {
        match value {
            0 => ResponseCode::NoError,
            1 => ResponseCode::FormatError,
            2 => ResponseCode::ServerFailure,
            3 => ResponseCode::NxDomain,
            4 => ResponseCode::NotImplemented,
            5 => ResponseCode::Refused,
            _ => ResponseCode::Unknown(value),
        }
    }
}

impl From<ResponseCode> for u8 {
    fn from(rcode: ResponseCode) -> Self {
        match rcode {
            ResponseCode::NoError => 0,
            ResponseCode::FormatError => 1,
            ResponseCode::ServerFailure => 2,
            ResponseCode::NxDomain => 3,
            ResponseCode::NotImplemented => 4,
            ResponseCode::Refused => 5,
            ResponseCode::Unknown(value) => value,
        }
    }
}

// 注意：保留 Flags 的 Default 实现，因为这是功能性需求
// DNS标志位的初始化不是兜底行为，而是协议规范的正常默认值
impl Default for Flags {
    fn default() -> Self {
        Self {
            qr: false,
            opcode: 0,
            aa: false,
            tc: false,
            rd: true,
            ra: false,
            z: 0,
            rcode: 0,
        }
    }
}

impl fmt::Display for RecordType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecordType::A => write!(f, "A"),
            RecordType::NS => write!(f, "NS"),
            RecordType::CNAME => write!(f, "CNAME"),
            RecordType::SOA => write!(f, "SOA"),
            RecordType::PTR => write!(f, "PTR"),
            RecordType::MX => write!(f, "MX"),
            RecordType::TXT => write!(f, "TXT"),
            RecordType::AAAA => write!(f, "AAAA"),
            RecordType::SRV => write!(f, "SRV"),
            RecordType::Unknown(value) => write!(f, "TYPE{}", value),
        }
    }
}