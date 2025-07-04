//! DNS响应包装器的完整测试套件
//!
//! 测试所有DNS记录类型的响应创建和验证功能

use rat_quickdns::{
    DnsResponseBuilder, DnsResponseWrapper, RecordType, QClass, RecordData, ResponseCode
};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

/// 测试DNS响应构建器的基本功能
#[test]
fn test_dns_response_builder_basic() {
    let response = DnsResponseBuilder::new()
        .with_id(12345)
        .with_authoritative(true)
        .with_response_code(0)
        .add_query("example.com".to_string(), RecordType::A, QClass::IN)
        .build();

    assert_eq!(response.id, 12345);
    assert!(response.flags.qr); // 应该是响应
    assert!(response.flags.aa); // 权威回答
    assert_eq!(response.flags.rcode, 0); // 无错误
    assert_eq!(response.queries.len(), 1);
    assert_eq!(response.queries[0].name, "example.com");
    assert_eq!(response.queries[0].qtype, RecordType::A);
    assert_eq!(response.queries[0].qclass, QClass::IN);
}

/// 测试A记录响应创建
#[test]
fn test_create_a_response() {
    let ips = vec![
        Ipv4Addr::new(192, 168, 1, 1),
        Ipv4Addr::new(10, 0, 0, 1),
        Ipv4Addr::new(172, 16, 0, 1),
    ];
    
    let response = DnsResponseWrapper::create_a_response(123, "test.com", &ips, 300);
    
    assert_eq!(response.id, 123);
    assert_eq!(response.answers.len(), 3);
    assert_eq!(response.queries.len(), 1);
    assert_eq!(response.queries[0].name, "test.com");
    assert_eq!(response.queries[0].qtype, RecordType::A);
    
    // 验证所有A记录
    for (i, answer) in response.answers.iter().enumerate() {
        assert_eq!(answer.name, "test.com");
        assert_eq!(answer.rtype, RecordType::A);
        assert_eq!(answer.ttl, 300);
        
        if let RecordData::A(ip) = &answer.data {
            assert_eq!(*ip, ips[i]);
        } else {
            panic!("Expected A record data at index {}", i);
        }
    }
}

/// 测试AAAA记录响应创建
#[test]
fn test_create_aaaa_response() {
    let ips = vec![
        Ipv6Addr::from_str("2001:db8::1").unwrap(),
        Ipv6Addr::from_str("fe80::1").unwrap(),
    ];
    
    let response = DnsResponseWrapper::create_aaaa_response(456, "ipv6.test.com", &ips, 600);
    
    assert_eq!(response.id, 456);
    assert_eq!(response.answers.len(), 2);
    assert_eq!(response.queries[0].qtype, RecordType::AAAA);
    
    for (i, answer) in response.answers.iter().enumerate() {
        assert_eq!(answer.rtype, RecordType::AAAA);
        assert_eq!(answer.ttl, 600);
        
        if let RecordData::AAAA(ip) = &answer.data {
            assert_eq!(*ip, ips[i]);
        } else {
            panic!("Expected AAAA record data at index {}", i);
        }
    }
}

/// 测试CNAME记录响应创建
#[test]
fn test_create_cname_response() {
    let response = DnsResponseWrapper::create_cname_response(
        789, 
        "alias.example.com", 
        "canonical.example.com", 
        1800
    );
    
    assert_eq!(response.id, 789);
    assert_eq!(response.answers.len(), 1);
    assert_eq!(response.queries[0].qtype, RecordType::CNAME);
    
    let answer = &response.answers[0];
    assert_eq!(answer.rtype, RecordType::CNAME);
    assert_eq!(answer.ttl, 1800);
    
    if let RecordData::CNAME(target) = &answer.data {
        assert_eq!(target, "canonical.example.com");
    } else {
        panic!("Expected CNAME record data");
    }
}

/// 测试MX记录响应创建
#[test]
fn test_create_mx_response() {
    let mx_records = vec![
        (10, "mail1.example.com".to_string()),
        (20, "mail2.example.com".to_string()),
        (30, "mail3.example.com".to_string()),
    ];
    
    let response = DnsResponseWrapper::create_mx_response(
        101, 
        "example.com", 
        &mx_records, 
        3600
    );
    
    assert_eq!(response.id, 101);
    assert_eq!(response.answers.len(), 3);
    assert_eq!(response.queries[0].qtype, RecordType::MX);
    
    for (i, answer) in response.answers.iter().enumerate() {
        assert_eq!(answer.rtype, RecordType::MX);
        assert_eq!(answer.ttl, 3600);
        
        if let RecordData::MX { priority, exchange } = &answer.data {
            assert_eq!(*priority, mx_records[i].0);
            assert_eq!(exchange, &mx_records[i].1);
        } else {
            panic!("Expected MX record data at index {}", i);
        }
    }
}

/// 测试TXT记录响应创建
#[test]
fn test_create_txt_response() {
    let texts = vec![
        "v=spf1 include:_spf.google.com ~all".to_string(),
        "google-site-verification=abc123".to_string(),
    ];
    
    let response = DnsResponseWrapper::create_txt_response(
        202, 
        "example.com", 
        &texts, 
        300
    );
    
    assert_eq!(response.id, 202);
    assert_eq!(response.answers.len(), 1);
    assert_eq!(response.queries[0].qtype, RecordType::TXT);
    
    let answer = &response.answers[0];
    assert_eq!(answer.rtype, RecordType::TXT);
    assert_eq!(answer.ttl, 300);
    
    if let RecordData::TXT(record_texts) = &answer.data {
        assert_eq!(record_texts.len(), 2);
        assert_eq!(record_texts[0], texts[0]);
        assert_eq!(record_texts[1], texts[1]);
    } else {
        panic!("Expected TXT record data");
    }
}

/// 测试SOA记录响应创建
#[test]
fn test_create_soa_response() {
    let response = DnsResponseWrapper::create_soa_response(
        303, 
        "example.com", 
        "ns1.example.com", 
        "admin.example.com", 
        7200
    );
    
    assert_eq!(response.id, 303);
    assert_eq!(response.answers.len(), 1);
    assert_eq!(response.queries[0].qtype, RecordType::SOA);
    
    let answer = &response.answers[0];
    assert_eq!(answer.rtype, RecordType::SOA);
    assert_eq!(answer.ttl, 7200);
    
    if let RecordData::SOA { mname, rname, serial, refresh, retry, expire, minimum } = &answer.data {
        assert_eq!(mname, "ns1.example.com");
        assert_eq!(rname, "admin.example.com");
        assert!(*serial > 0); // 应该是当前时间戳
        assert_eq!(*refresh, 3600);
        assert_eq!(*retry, 1800);
        assert_eq!(*expire, 604800);
        assert_eq!(*minimum, 86400);
    } else {
        panic!("Expected SOA record data");
    }
}

/// 测试SRV记录响应创建
#[test]
fn test_create_srv_response() {
    let srv_records = vec![
        (10, 60, 443, "server1.example.com".to_string()),
        (10, 40, 443, "server2.example.com".to_string()),
        (20, 100, 443, "backup.example.com".to_string()),
    ];
    
    let response = DnsResponseWrapper::create_srv_response(
        404, 
        "_https._tcp.example.com", 
        &srv_records, 
        1800
    );
    
    assert_eq!(response.id, 404);
    assert_eq!(response.answers.len(), 3);
    assert_eq!(response.queries[0].qtype, RecordType::SRV);
    
    for (i, answer) in response.answers.iter().enumerate() {
        assert_eq!(answer.rtype, RecordType::SRV);
        assert_eq!(answer.ttl, 1800);
        
        if let RecordData::SRV { priority, weight, port, target } = &answer.data {
            assert_eq!(*priority, srv_records[i].0);
            assert_eq!(*weight, srv_records[i].1);
            assert_eq!(*port, srv_records[i].2);
            assert_eq!(target, &srv_records[i].3);
        } else {
            panic!("Expected SRV record data at index {}", i);
        }
    }
}

/// 测试PTR记录响应创建（反向DNS）
#[test]
fn test_create_ptr_response() {
    let response = DnsResponseWrapper::create_ptr_response(
        505, 
        "1.1.168.192.in-addr.arpa", 
        "host.example.com", 
        3600
    );
    
    assert_eq!(response.id, 505);
    assert_eq!(response.answers.len(), 1);
    assert_eq!(response.queries[0].qtype, RecordType::PTR);
    
    let answer = &response.answers[0];
    assert_eq!(answer.rtype, RecordType::PTR);
    assert_eq!(answer.ttl, 3600);
    
    if let RecordData::PTR(target) = &answer.data {
        assert_eq!(target, "host.example.com");
    } else {
        panic!("Expected PTR record data");
    }
}

/// 测试NS记录响应创建
#[test]
fn test_create_ns_response() {
    let nameservers = vec![
        "ns1.example.com".to_string(),
        "ns2.example.com".to_string(),
        "ns3.example.com".to_string(),
    ];
    
    let response = DnsResponseWrapper::create_ns_response(
        606, 
        "example.com", 
        &nameservers, 
        86400
    );
    
    assert_eq!(response.id, 606);
    assert_eq!(response.answers.len(), 3);
    assert_eq!(response.queries[0].qtype, RecordType::NS);
    
    for (i, answer) in response.answers.iter().enumerate() {
        assert_eq!(answer.rtype, RecordType::NS);
        assert_eq!(answer.ttl, 86400);
        
        if let RecordData::NS(nameserver) = &answer.data {
            assert_eq!(nameserver, &nameservers[i]);
        } else {
            panic!("Expected NS record data at index {}", i);
        }
    }
}

/// 测试NXDOMAIN响应创建
#[test]
fn test_create_nxdomain_response() {
    let response = DnsResponseWrapper::create_nxdomain_response(
        707, 
        "nonexistent.example.com", 
        RecordType::A
    );
    
    assert_eq!(response.id, 707);
    assert_eq!(response.flags.rcode, 3); // NXDOMAIN
    assert_eq!(response.answers.len(), 0); // 没有回答记录
    assert_eq!(response.queries.len(), 1);
    assert_eq!(response.queries[0].name, "nonexistent.example.com");
    assert_eq!(response.queries[0].qtype, RecordType::A);
}

/// 测试服务器错误响应创建
#[test]
fn test_create_server_failure_response() {
    let response = DnsResponseWrapper::create_server_failure_response(
        808, 
        "error.example.com", 
        RecordType::AAAA
    );
    
    assert_eq!(response.id, 808);
    assert_eq!(response.flags.rcode, 2); // SERVFAIL
    assert_eq!(response.answers.len(), 0); // 没有回答记录
    assert_eq!(response.queries.len(), 1);
    assert_eq!(response.queries[0].name, "error.example.com");
    assert_eq!(response.queries[0].qtype, RecordType::AAAA);
}

/// 测试复杂的DNS响应构建（包含多种记录类型）
#[test]
fn test_complex_dns_response() {
    let response = DnsResponseBuilder::new()
        .with_id(999)
        .with_authoritative(true)
        .add_query("example.com".to_string(), RecordType::A, QClass::IN)
        // 添加A记录回答
        .add_a_answer("example.com".to_string(), 300, Ipv4Addr::new(192, 168, 1, 1))
        .add_a_answer("example.com".to_string(), 300, Ipv4Addr::new(192, 168, 1, 2))
        // 添加NS权威记录
        .add_authority(rat_quickdns::Record {
            name: "example.com".to_string(),
            rtype: RecordType::NS,
            class: QClass::IN,
            ttl: 86400,
            data: RecordData::NS("ns1.example.com".to_string()),
        })
        // 添加A记录附加信息
        .add_additional(rat_quickdns::Record {
            name: "ns1.example.com".to_string(),
            rtype: RecordType::A,
            class: QClass::IN,
            ttl: 86400,
            data: RecordData::A(Ipv4Addr::new(192, 168, 2, 1)),
        })
        .build();
    
    assert_eq!(response.id, 999);
    assert!(response.flags.aa); // 权威回答
    assert_eq!(response.queries.len(), 1);
    assert_eq!(response.answers.len(), 2); // 两个A记录
    assert_eq!(response.authorities.len(), 1); // 一个NS记录
    assert_eq!(response.additionals.len(), 1); // 一个附加A记录
    
    // 验证回答记录
    for answer in &response.answers {
        assert_eq!(answer.rtype, RecordType::A);
        assert_eq!(answer.name, "example.com");
    }
    
    // 验证权威记录
    let authority = &response.authorities[0];
    assert_eq!(authority.rtype, RecordType::NS);
    assert_eq!(authority.name, "example.com");
    
    // 验证附加记录
    let additional = &response.additionals[0];
    assert_eq!(additional.rtype, RecordType::A);
    assert_eq!(additional.name, "ns1.example.com");
}

/// 测试截断标志的设置
#[test]
fn test_truncated_response() {
    let response = DnsResponseBuilder::new()
        .with_id(1111)
        .with_truncated(true)
        .add_query("large.example.com".to_string(), RecordType::A, QClass::IN)
        .build();
    
    assert_eq!(response.id, 1111);
    assert!(response.flags.tc); // 截断标志应该被设置
}

/// 测试不同响应码的设置
#[test]
fn test_various_response_codes() {
    // 测试各种响应码
    let test_cases = vec![
        (0, "NoError"),
        (1, "FormatError"),
        (2, "ServerFailure"),
        (3, "NxDomain"),
        (4, "NotImplemented"),
        (5, "Refused"),
    ];
    
    for (rcode, description) in test_cases {
        let response = DnsResponseBuilder::new()
            .with_id(2000 + rcode as u16)
            .with_response_code(rcode)
            .add_query(format!("test-{}.example.com", description.to_lowercase()), RecordType::A, QClass::IN)
            .build();
        
        assert_eq!(response.flags.rcode, rcode, "Failed for {}", description);
    }
}

/// 性能测试：创建大量DNS响应
#[test]
fn test_performance_bulk_response_creation() {
    use std::time::Instant;
    
    let start = Instant::now();
    let mut responses = Vec::new();
    
    // 创建1000个DNS响应
    for i in 0..1000 {
        let response = DnsResponseWrapper::create_a_response(
            i as u16,
            &format!("test{}.example.com", i),
            &[Ipv4Addr::new(192, 168, (i / 256) as u8, (i % 256) as u8)],
            300,
        );
        responses.push(response);
    }
    
    let duration = start.elapsed();
    println!("创建1000个DNS响应耗时: {:?}", duration);
    
    // 验证所有响应都正确创建
    assert_eq!(responses.len(), 1000);
    for (i, response) in responses.iter().enumerate() {
        assert_eq!(response.id, i as u16);
        assert_eq!(response.answers.len(), 1);
    }
    
    // 性能要求：应该在100ms内完成
    assert!(duration.as_millis() < 100, "性能测试失败：耗时 {:?}", duration);
}

/// 边界条件测试
#[test]
fn test_edge_cases() {
    // 测试空域名
    let response = DnsResponseWrapper::create_a_response(
        1234,
        "",
        &[Ipv4Addr::new(127, 0, 0, 1)],
        0,
    );
    assert_eq!(response.queries[0].name, "");
    assert_eq!(response.answers[0].ttl, 0);
    
    // 测试最大TTL值
    let response = DnsResponseWrapper::create_a_response(
        5678,
        "max-ttl.example.com",
        &[Ipv4Addr::new(127, 0, 0, 1)],
        u32::MAX,
    );
    assert_eq!(response.answers[0].ttl, u32::MAX);
    
    // 测试空IP列表
    let response = DnsResponseWrapper::create_a_response(
        9999,
        "empty.example.com",
        &[],
        300,
    );
    assert_eq!(response.answers.len(), 0);
}

/// 集成测试：模拟真实DNS服务器场景
#[test]
fn test_dns_server_simulation() {
    // 模拟处理不同类型的DNS查询
    struct MockDnsServer;
    
    impl MockDnsServer {
        fn handle_query(&self, query_id: u16, domain: &str, qtype: RecordType) -> rat_quickdns::Response {
            match (domain, qtype) {
                ("example.com", RecordType::A) => {
                    DnsResponseWrapper::create_a_response(
                        query_id,
                        domain,
                        &[Ipv4Addr::new(93, 184, 216, 34)],
                        300,
                    )
                }
                ("example.com", RecordType::AAAA) => {
                    DnsResponseWrapper::create_aaaa_response(
                        query_id,
                        domain,
                        &[Ipv6Addr::from_str("2606:2800:220:1:248:1893:25c8:1946").unwrap()],
                        300,
                    )
                }
                ("example.com", RecordType::MX) => {
                    DnsResponseWrapper::create_mx_response(
                        query_id,
                        domain,
                        &[(10, "mail.example.com".to_string())],
                        3600,
                    )
                }
                ("www.example.com", RecordType::CNAME) => {
                    DnsResponseWrapper::create_cname_response(
                        query_id,
                        domain,
                        "example.com",
                        1800,
                    )
                }
                (_, _) => {
                    DnsResponseWrapper::create_nxdomain_response(query_id, domain, qtype)
                }
            }
        }
    }
    
    let server = MockDnsServer;
    
    // 测试各种查询
    let test_queries = vec![
        (1, "example.com", RecordType::A),
        (2, "example.com", RecordType::AAAA),
        (3, "example.com", RecordType::MX),
        (4, "www.example.com", RecordType::CNAME),
        (5, "nonexistent.com", RecordType::A),
    ];
    
    for (query_id, domain, qtype) in test_queries {
        let response = server.handle_query(query_id, domain, qtype);
        
        assert_eq!(response.id, query_id);
        assert_eq!(response.queries[0].name, domain);
        assert_eq!(response.queries[0].qtype, qtype);
        
        match domain {
            "nonexistent.com" => {
                assert_eq!(response.flags.rcode, 3); // NXDOMAIN
                assert_eq!(response.answers.len(), 0);
            }
            _ => {
                assert_eq!(response.flags.rcode, 0); // NoError
                assert!(response.answers.len() > 0);
            }
        }
    }
}

/// 测试DNS响应的序列化兼容性（为将来的网络传输做准备）
#[test]
fn test_response_structure_completeness() {
    let response = DnsResponseBuilder::new()
        .with_id(42)
        .with_authoritative(true)
        .with_response_code(0)
        .add_query("test.com".to_string(), RecordType::A, QClass::IN)
        .add_a_answer("test.com".to_string(), 300, Ipv4Addr::new(1, 2, 3, 4))
        .build();
    
    // 验证响应结构的完整性
    assert!(response.flags.qr); // 必须是响应
    assert_eq!(response.flags.opcode, 0); // 标准查询
    assert!(response.flags.rd); // 期望递归
    assert!(response.flags.ra); // 递归可用
    assert_eq!(response.flags.z, 0); // 保留位必须为0
    
    // 验证记录结构
    let answer = &response.answers[0];
    assert_eq!(answer.class, QClass::IN); // 应该是Internet类
    assert!(answer.ttl > 0); // TTL应该大于0
    
    // 验证数据完整性
    if let RecordData::A(ip) = &answer.data {
        assert_eq!(*ip, Ipv4Addr::new(1, 2, 3, 4));
    } else {
        panic!("Expected A record data");
    }
}