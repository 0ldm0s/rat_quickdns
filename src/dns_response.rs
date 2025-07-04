//! DNS响应包装器
//!
//! 提供将数据包装成传统DNS响应的功能，支持所有DNS记录类型

use crate::types::*;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::time::{SystemTime, UNIX_EPOCH};

/// DNS响应构建器
#[derive(Debug, Clone)]
pub struct DnsResponseBuilder {
    /// 事务ID
    id: u16,
    /// 标志位
    flags: Flags,
    /// 查询问题
    queries: Vec<Query>,
    /// 回答记录
    answers: Vec<Record>,
    /// 权威记录
    authorities: Vec<Record>,
    /// 附加记录
    additionals: Vec<Record>,
}

impl Default for DnsResponseBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl DnsResponseBuilder {
    /// 创建新的DNS响应构建器
    pub fn new() -> Self {
        Self {
            id: 0,
            flags: Flags {
                qr: true,  // 响应
                opcode: 0, // 标准查询
                aa: false, // 非权威回答
                tc: false, // 未截断
                rd: true,  // 期望递归
                ra: true,  // 递归可用
                z: 0,      // 保留位
                rcode: 0,  // 无错误
            },
            queries: Vec::new(),
            answers: Vec::new(),
            authorities: Vec::new(),
            additionals: Vec::new(),
        }
    }

    /// 设置事务ID
    pub fn with_id(mut self, id: u16) -> Self {
        self.id = id;
        self
    }

    /// 设置响应码
    pub fn with_response_code(mut self, rcode: u8) -> Self {
        self.flags.rcode = rcode;
        self
    }

    /// 设置权威回答标志
    pub fn with_authoritative(mut self, aa: bool) -> Self {
        self.flags.aa = aa;
        self
    }

    /// 设置截断标志
    pub fn with_truncated(mut self, tc: bool) -> Self {
        self.flags.tc = tc;
        self
    }

    /// 添加查询问题
    pub fn add_query(mut self, name: String, qtype: RecordType, qclass: QClass) -> Self {
        self.queries.push(Query {
            name,
            qtype,
            qclass,
        });
        self
    }

    /// 添加A记录到回答部分
    pub fn add_a_answer(mut self, name: String, ttl: u32, ip: Ipv4Addr) -> Self {
        self.answers.push(Record {
            name,
            rtype: RecordType::A,
            class: QClass::IN,
            ttl,
            data: RecordData::A(ip),
        });
        self
    }

    /// 添加AAAA记录到回答部分
    pub fn add_aaaa_answer(mut self, name: String, ttl: u32, ip: Ipv6Addr) -> Self {
        self.answers.push(Record {
            name,
            rtype: RecordType::AAAA,
            class: QClass::IN,
            ttl,
            data: RecordData::AAAA(ip),
        });
        self
    }

    /// 添加CNAME记录到回答部分
    pub fn add_cname_answer(mut self, name: String, ttl: u32, target: String) -> Self {
        self.answers.push(Record {
            name,
            rtype: RecordType::CNAME,
            class: QClass::IN,
            ttl,
            data: RecordData::CNAME(target),
        });
        self
    }

    /// 添加MX记录到回答部分
    pub fn add_mx_answer(mut self, name: String, ttl: u32, priority: u16, exchange: String) -> Self {
        self.answers.push(Record {
            name,
            rtype: RecordType::MX,
            class: QClass::IN,
            ttl,
            data: RecordData::MX { priority, exchange },
        });
        self
    }

    /// 添加NS记录到回答部分
    pub fn add_ns_answer(mut self, name: String, ttl: u32, nameserver: String) -> Self {
        self.answers.push(Record {
            name,
            rtype: RecordType::NS,
            class: QClass::IN,
            ttl,
            data: RecordData::NS(nameserver),
        });
        self
    }

    /// 添加PTR记录到回答部分
    pub fn add_ptr_answer(mut self, name: String, ttl: u32, target: String) -> Self {
        self.answers.push(Record {
            name,
            rtype: RecordType::PTR,
            class: QClass::IN,
            ttl,
            data: RecordData::PTR(target),
        });
        self
    }

    /// 添加SOA记录到回答部分
    pub fn add_soa_answer(
        mut self,
        name: String,
        ttl: u32,
        mname: String,
        rname: String,
        serial: u32,
        refresh: u32,
        retry: u32,
        expire: u32,
        minimum: u32,
    ) -> Self {
        self.answers.push(Record {
            name,
            rtype: RecordType::SOA,
            class: QClass::IN,
            ttl,
            data: RecordData::SOA {
                mname,
                rname,
                serial,
                refresh,
                retry,
                expire,
                minimum,
            },
        });
        self
    }

    /// 添加TXT记录到回答部分
    pub fn add_txt_answer(mut self, name: String, ttl: u32, texts: Vec<String>) -> Self {
        self.answers.push(Record {
            name,
            rtype: RecordType::TXT,
            class: QClass::IN,
            ttl,
            data: RecordData::TXT(texts),
        });
        self
    }

    /// 添加SRV记录到回答部分
    pub fn add_srv_answer(
        mut self,
        name: String,
        ttl: u32,
        priority: u16,
        weight: u16,
        port: u16,
        target: String,
    ) -> Self {
        self.answers.push(Record {
            name,
            rtype: RecordType::SRV,
            class: QClass::IN,
            ttl,
            data: RecordData::SRV {
                priority,
                weight,
                port,
                target,
            },
        });
        self
    }

    /// 添加权威记录
    pub fn add_authority(mut self, record: Record) -> Self {
        self.authorities.push(record);
        self
    }

    /// 添加附加记录
    pub fn add_additional(mut self, record: Record) -> Self {
        self.additionals.push(record);
        self
    }

    /// 构建DNS响应
    pub fn build(self) -> Response {
        Response {
            id: self.id,
            flags: self.flags,
            queries: self.queries,
            answers: self.answers,
            authorities: self.authorities,
            additionals: self.additionals,
        }
    }
}

/// DNS响应包装器
pub struct DnsResponseWrapper;

impl DnsResponseWrapper {
    /// 创建成功的A记录响应
    pub fn create_a_response(query_id: u16, domain: &str, ips: &[Ipv4Addr], ttl: u32) -> Response {
        let mut builder = DnsResponseBuilder::new()
            .with_id(query_id)
            .add_query(domain.to_string(), RecordType::A, QClass::IN);

        for ip in ips {
            builder = builder.add_a_answer(domain.to_string(), ttl, *ip);
        }

        builder.build()
    }

    /// 创建成功的AAAA记录响应
    pub fn create_aaaa_response(query_id: u16, domain: &str, ips: &[Ipv6Addr], ttl: u32) -> Response {
        let mut builder = DnsResponseBuilder::new()
            .with_id(query_id)
            .add_query(domain.to_string(), RecordType::AAAA, QClass::IN);

        for ip in ips {
            builder = builder.add_aaaa_answer(domain.to_string(), ttl, *ip);
        }

        builder.build()
    }

    /// 创建NXDOMAIN响应
    pub fn create_nxdomain_response(query_id: u16, domain: &str, qtype: RecordType) -> Response {
        DnsResponseBuilder::new()
            .with_id(query_id)
            .with_response_code(3) // NXDOMAIN
            .add_query(domain.to_string(), qtype, QClass::IN)
            .build()
    }

    /// 创建服务器错误响应
    pub fn create_server_failure_response(query_id: u16, domain: &str, qtype: RecordType) -> Response {
        DnsResponseBuilder::new()
            .with_id(query_id)
            .with_response_code(2) // SERVFAIL
            .add_query(domain.to_string(), qtype, QClass::IN)
            .build()
    }

    /// 创建CNAME响应
    pub fn create_cname_response(query_id: u16, domain: &str, target: &str, ttl: u32) -> Response {
        DnsResponseBuilder::new()
            .with_id(query_id)
            .add_query(domain.to_string(), RecordType::CNAME, QClass::IN)
            .add_cname_answer(domain.to_string(), ttl, target.to_string())
            .build()
    }

    /// 创建MX响应
    pub fn create_mx_response(query_id: u16, domain: &str, mx_records: &[(u16, String)], ttl: u32) -> Response {
        let mut builder = DnsResponseBuilder::new()
            .with_id(query_id)
            .add_query(domain.to_string(), RecordType::MX, QClass::IN);

        for (priority, exchange) in mx_records {
            builder = builder.add_mx_answer(domain.to_string(), ttl, *priority, exchange.clone());
        }

        builder.build()
    }

    /// 创建TXT响应
    pub fn create_txt_response(query_id: u16, domain: &str, texts: &[String], ttl: u32) -> Response {
        DnsResponseBuilder::new()
            .with_id(query_id)
            .add_query(domain.to_string(), RecordType::TXT, QClass::IN)
            .add_txt_answer(domain.to_string(), ttl, texts.to_vec())
            .build()
    }

    /// 创建当前时间戳的SOA记录
    pub fn create_soa_response(
        query_id: u16,
        domain: &str,
        mname: &str,
        rname: &str,
        ttl: u32,
    ) -> Response {
        let serial = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32;

        DnsResponseBuilder::new()
            .with_id(query_id)
            .add_query(domain.to_string(), RecordType::SOA, QClass::IN)
            .add_soa_answer(
                domain.to_string(),
                ttl,
                mname.to_string(),
                rname.to_string(),
                serial,
                3600,  // refresh
                1800,  // retry
                604800, // expire
                86400, // minimum
            )
            .build()
    }

    /// 创建SRV响应
    pub fn create_srv_response(
        query_id: u16,
        domain: &str,
        srv_records: &[(u16, u16, u16, String)], // priority, weight, port, target
        ttl: u32,
    ) -> Response {
        let mut builder = DnsResponseBuilder::new()
            .with_id(query_id)
            .add_query(domain.to_string(), RecordType::SRV, QClass::IN);

        for (priority, weight, port, target) in srv_records {
            builder = builder.add_srv_answer(
                domain.to_string(),
                ttl,
                *priority,
                *weight,
                *port,
                target.clone(),
            );
        }

        builder.build()
    }

    /// 创建PTR响应（用于反向DNS查询）
    pub fn create_ptr_response(query_id: u16, ptr_name: &str, target: &str, ttl: u32) -> Response {
        DnsResponseBuilder::new()
            .with_id(query_id)
            .add_query(ptr_name.to_string(), RecordType::PTR, QClass::IN)
            .add_ptr_answer(ptr_name.to_string(), ttl, target.to_string())
            .build()
    }

    /// 创建NS响应
    pub fn create_ns_response(query_id: u16, domain: &str, nameservers: &[String], ttl: u32) -> Response {
        let mut builder = DnsResponseBuilder::new()
            .with_id(query_id)
            .add_query(domain.to_string(), RecordType::NS, QClass::IN);

        for ns in nameservers {
            builder = builder.add_ns_answer(domain.to_string(), ttl, ns.clone());
        }

        builder.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_dns_response_builder() {
        let response = DnsResponseBuilder::new()
            .with_id(12345)
            .with_authoritative(true)
            .add_query("example.com".to_string(), RecordType::A, QClass::IN)
            .add_a_answer("example.com".to_string(), 300, Ipv4Addr::new(192, 168, 1, 1))
            .build();

        assert_eq!(response.id, 12345);
        assert!(response.flags.aa);
        assert_eq!(response.queries.len(), 1);
        assert_eq!(response.answers.len(), 1);
        assert_eq!(response.queries[0].name, "example.com");
        
        if let RecordData::A(ip) = &response.answers[0].data {
            assert_eq!(*ip, Ipv4Addr::new(192, 168, 1, 1));
        } else {
            panic!("Expected A record data");
        }
    }

    #[test]
    fn test_create_a_response() {
        let ips = vec![Ipv4Addr::new(1, 2, 3, 4), Ipv4Addr::new(5, 6, 7, 8)];
        let response = DnsResponseWrapper::create_a_response(123, "test.com", &ips, 300);
        
        assert_eq!(response.id, 123);
        assert_eq!(response.answers.len(), 2);
        assert_eq!(response.queries[0].name, "test.com");
    }

    #[test]
    fn test_create_nxdomain_response() {
        let response = DnsResponseWrapper::create_nxdomain_response(456, "notfound.com", RecordType::A);
        
        assert_eq!(response.id, 456);
        assert_eq!(response.flags.rcode, 3); // NXDOMAIN
        assert_eq!(response.answers.len(), 0);
        assert_eq!(response.queries[0].name, "notfound.com");
    }
}