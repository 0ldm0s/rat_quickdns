//! DNS解析器核心类型定义
//! 
//! 本模块定义了DNS查询过程中使用的核心数据结构

use serde::{Deserialize, Serialize};
use std::net::IpAddr;

/// DNS查询请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsQueryRequest {
    /// 查询ID（可选，用于追踪）
    pub query_id: Option<String>,
    
    /// 要查询的域名
    pub domain: String,
    
    /// 记录类型
    pub record_type: DnsRecordType,
    
    /// 是否启用EDNS
    pub enable_edns: bool,
    
    /// 客户端子网信息（用于CDN优化）
    pub client_subnet: Option<String>,
    
    /// 查询超时时间（毫秒）
    pub timeout_ms: Option<u64>,
    
    /// 是否禁用缓存
    pub disable_cache: bool,
    
    /// 是否启用DNSSEC验证
    pub enable_dnssec: bool,
}

impl DnsQueryRequest {
    /// 创建新的DNS查询请求
    pub fn new(domain: impl Into<String>, record_type: DnsRecordType) -> Self {
        Self {
            query_id: None,
            domain: domain.into(),
            record_type,
            enable_edns: true,
            client_subnet: None,
            timeout_ms: None,
            disable_cache: false,
            enable_dnssec: false,
        }
    }
    
    /// 设置查询ID
    pub fn with_query_id(mut self, id: impl Into<String>) -> Self {
        self.query_id = Some(id.into());
        self
    }
    
    /// 设置客户端子网
    pub fn with_client_subnet(mut self, subnet: impl Into<String>) -> Self {
        self.client_subnet = Some(subnet.into());
        self
    }
    
    /// 设置超时时间
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }
    
    /// 禁用缓存
    pub fn disable_cache(mut self) -> Self {
        self.disable_cache = true;
        self
    }
    
    /// 启用DNSSEC验证
    pub fn with_dnssec(mut self, enable: bool) -> Self {
        self.enable_dnssec = enable;
        self
    }
}

/// DNS查询响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsQueryResponse {
    /// 查询ID
    pub query_id: String,
    
    /// 查询的域名
    pub domain: String,
    
    /// 记录类型
    pub record_type: DnsRecordType,
    
    /// 查询是否成功
    pub success: bool,
    
    /// 错误信息（如果失败）
    pub error: Option<String>,
    
    /// DNS记录列表
    pub records: Vec<DnsRecord>,
    
    /// 查询耗时（毫秒）
    pub duration_ms: u64,
    
    /// 使用的上游服务器
    pub server_used: Option<String>,
    
    /// DNSSEC验证状态
    pub dnssec_status: Option<DnssecStatus>,
    
    /// DNSSEC相关记录（RRSIG、DNSKEY等）
    pub dnssec_records: Vec<DnsRecord>,
}

impl DnsQueryResponse {
    /// 提取IP地址列表
    pub fn ip_addresses(&self) -> Vec<IpAddr> {
        self.records
            .iter()
            .filter_map(|record| {
                if let DnsRecordValue::IpAddr(ip) = &record.value {
                    Some(*ip)
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// 提取域名列表（用于CNAME等记录）
    pub fn domains(&self) -> Vec<String> {
        self.records
            .iter()
            .filter_map(|record| {
                if let DnsRecordValue::Domain(domain) = &record.value {
                    Some(domain.clone())
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// 提取文本列表（用于TXT记录）
    pub fn texts(&self) -> Vec<String> {
        self.records
            .iter()
            .filter_map(|record| {
                if let DnsRecordValue::Text(text) = &record.value {
                    Some(text.clone())
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// 提取MX记录
    pub fn mx_records(&self) -> Vec<(u16, String)> {
        self.records
            .iter()
            .filter_map(|record| {
                if let DnsRecordValue::Mx { priority, exchange } = &record.value {
                    Some((*priority, exchange.clone()))
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// 检查是否有DNSSEC记录
    pub fn has_dnssec_records(&self) -> bool {
        !self.dnssec_records.is_empty() || 
        self.records.iter().any(|r| r.record_type.is_dnssec_record())
    }
    
    /// 获取DNSSEC状态描述
    pub fn dnssec_status_description(&self) -> String {
        match &self.dnssec_status {
            Some(DnssecStatus::Secure) => "🔒 DNSSEC验证通过".to_string(),
            Some(DnssecStatus::Insecure) => "🔓 未启用DNSSEC".to_string(),
            Some(DnssecStatus::Bogus) => "⚠️ DNSSEC验证失败".to_string(),
            Some(DnssecStatus::Indeterminate) => "❓ DNSSEC状态不确定".to_string(),
            None => "➖ 无DNSSEC信息".to_string(),
        }
    }
    
    /// 提取DNSSEC相关记录
    pub fn dnssec_record_summary(&self) -> String {
        let dnssec_records: Vec<_> = self.records.iter()
            .filter(|r| r.record_type.is_dnssec_record())
            .collect();
        
        if dnssec_records.is_empty() {
            "无DNSSEC记录".to_string()
        } else {
            let mut summary = Vec::new();
            let mut counts = std::collections::HashMap::new();
            
            for record in &dnssec_records {
                *counts.entry(record.record_type).or_insert(0) += 1;
            }
            
            for (record_type, count) in counts {
                summary.push(format!("{}: {}", record_type.as_str(), count));
            }
            
            summary.join(", ")
        }
    }
}

/// DNSSEC验证状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DnssecStatus {
    /// 安全 - DNSSEC验证通过
    Secure,
    
    /// 不安全 - 域名未启用DNSSEC
    Insecure,
    
    /// 伪造 - DNSSEC验证失败
    Bogus,
    
    /// 不确定 - 无法验证DNSSEC状态
    Indeterminate,
}

/// DNS记录类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DnsRecordType {
    /// A记录 - IPv4地址
    A,
    
    /// AAAA记录 - IPv6地址
    AAAA,
    
    /// CNAME记录 - 别名
    CNAME,
    
    /// MX记录 - 邮件交换
    MX,
    
    /// TXT记录 - 文本记录
    TXT,
    
    /// NS记录 - 名称服务器
    NS,
    
    /// PTR记录 - 反向解析
    PTR,
    
    /// SRV记录 - 服务记录
    SRV,
    
    /// SOA记录 - 授权开始
    SOA,
    
    /// DNSSEC相关记录类型
    /// RRSIG记录 - 资源记录签名
    RRSIG,
    
    /// DNSKEY记录 - DNS公钥
    DNSKEY,
    
    /// DS记录 - 委托签名者
    DS,
    
    /// NSEC记录 - 下一个安全记录
    NSEC,
    
    /// NSEC3记录 - 下一个安全记录版本3
    NSEC3,
}

impl DnsRecordType {
    /// 获取记录类型的字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::A => "A",
            Self::AAAA => "AAAA",
            Self::CNAME => "CNAME",
            Self::MX => "MX",
            Self::TXT => "TXT",
            Self::NS => "NS",
            Self::PTR => "PTR",
            Self::SRV => "SRV",
            Self::SOA => "SOA",
            Self::RRSIG => "RRSIG",
            Self::DNSKEY => "DNSKEY",
            Self::DS => "DS",
            Self::NSEC => "NSEC",
            Self::NSEC3 => "NSEC3",
        }
    }
    
    /// 从字符串解析记录类型
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "A" => Some(Self::A),
            "AAAA" => Some(Self::AAAA),
            "CNAME" => Some(Self::CNAME),
            "MX" => Some(Self::MX),
            "TXT" => Some(Self::TXT),
            "NS" => Some(Self::NS),
            "PTR" => Some(Self::PTR),
            "SRV" => Some(Self::SRV),
            "SOA" => Some(Self::SOA),
            "RRSIG" => Some(Self::RRSIG),
            "DNSKEY" => Some(Self::DNSKEY),
            "DS" => Some(Self::DS),
            "NSEC" => Some(Self::NSEC),
            "NSEC3" => Some(Self::NSEC3),
            _ => None,
        }
    }
    
    /// 检查是否为DNSSEC相关记录类型
    pub fn is_dnssec_record(&self) -> bool {
        matches!(self, Self::RRSIG | Self::DNSKEY | Self::DS | Self::NSEC | Self::NSEC3)
    }
}

/// DNS记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    /// 记录名称
    pub name: String,
    
    /// 记录类型
    pub record_type: DnsRecordType,
    
    /// 记录值
    pub value: DnsRecordValue,
    
    /// TTL（生存时间）
    pub ttl: u32,
}

/// DNS记录值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DnsRecordValue {
    /// IP地址（A/AAAA记录）
    IpAddr(IpAddr),
    
    /// 域名（CNAME/NS/PTR记录）
    Domain(String),
    
    /// 文本（TXT记录）
    Text(String),
    
    /// MX记录
    Mx {
        priority: u16,
        exchange: String,
    },
    
    /// SRV记录
    Srv {
        priority: u16,
        weight: u16,
        port: u16,
        target: String,
    },
    
    /// SOA记录
    Soa {
        mname: String,
        rname: String,
        serial: u32,
        refresh: u32,
        retry: u32,
        expire: u32,
        minimum: u32,
    },
}

impl DnsRecord {
    /// 创建A记录
    pub fn a(name: impl Into<String>, ip: std::net::Ipv4Addr, ttl: u32) -> Self {
        Self {
            name: name.into(),
            record_type: DnsRecordType::A,
            value: DnsRecordValue::IpAddr(IpAddr::V4(ip)),
            ttl,
        }
    }
    
    /// 创建AAAA记录
    pub fn aaaa(name: impl Into<String>, ip: std::net::Ipv6Addr, ttl: u32) -> Self {
        Self {
            name: name.into(),
            record_type: DnsRecordType::AAAA,
            value: DnsRecordValue::IpAddr(IpAddr::V6(ip)),
            ttl,
        }
    }
    
    /// 创建CNAME记录
    pub fn cname(name: impl Into<String>, target: impl Into<String>, ttl: u32) -> Self {
        Self {
            name: name.into(),
            record_type: DnsRecordType::CNAME,
            value: DnsRecordValue::Domain(target.into()),
            ttl,
        }
    }
    
    /// 创建TXT记录
    pub fn txt(name: impl Into<String>, text: impl Into<String>, ttl: u32) -> Self {
        Self {
            name: name.into(),
            record_type: DnsRecordType::TXT,
            value: DnsRecordValue::Text(text.into()),
            ttl,
        }
    }
}