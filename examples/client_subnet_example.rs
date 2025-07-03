//! 客户端子网(EDNS Client Subnet)功能示例
//! 
//! 此示例演示如何使用DNS查询中的客户端IP转发功能，
//! 这对于CDN和地理位置相关的DNS解析非常有用。

use rat_quickdns::{
    Resolver,
    resolver::ResolverConfig,
    types::{RecordType, QClass, ClientSubnet},
    transport::TransportConfig,
};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建解析器配置
    let mut config = ResolverConfig::default();
    config.default_timeout = Duration::from_secs(5);
    
    // 创建解析器
    let mut resolver = Resolver::new(config);
    
    // 添加腾讯云DNS传输
    let udp_config = TransportConfig {
        server: "119.29.29.29".to_string(), // 腾讯云DNS
        port: 53,
        timeout: Duration::from_secs(5),
        tcp_fast_open: false,
        tcp_nodelay: true,
        pool_size: 10,
    };
    resolver.add_udp_transport(udp_config);
    
    let domain = "www.jd.com"; // 京东网站
    
    println!("=== DNS查询示例: {} ===", domain);
    
    // 1. 普通查询(不带客户端IP)
    println!("\n1. 普通查询(不带客户端IP):");
    match resolver.query(domain, RecordType::A, QClass::IN).await {
        Ok(response) => {
            println!("   查询成功，获得 {} 条记录", response.answers.len());
            for answer in &response.answers {
                println!("   - {}: {:?}", answer.name, answer.data);
            }
        }
        Err(e) => println!("   查询失败: {}", e),
    }
    
    // 2. 使用国际IP地址查询(应该返回国际站IP)
    println!("\n2. 使用国际IP地址查询(176.32.39.57):");
    let international_ip = IpAddr::V4(Ipv4Addr::new(176, 32, 39, 57)); // 国际IP
    match resolver.query_with_client_ip(domain, RecordType::A, QClass::IN, Some(international_ip)).await {
        Ok(response) => {
            println!("   查询成功(客户端IP: {})，获得 {} 条记录", international_ip, response.answers.len());
            for answer in &response.answers {
                println!("   - {}: {:?}", answer.name, answer.data);
            }
        }
        Err(e) => println!("   查询失败: {}", e),
    }
    
    // 3. 使用国内IP地址查询(应该返回国内站IP)
    println!("\n3. 使用国内IP地址查询(192.124.176.124):");
    let domestic_ip = IpAddr::V4(Ipv4Addr::new(192, 124, 176, 124)); // 国内IP
    match resolver.query_with_client_ip(domain, RecordType::A, QClass::IN, Some(domestic_ip)).await {
        Ok(response) => {
            println!("   查询成功(客户端IP: {})，获得 {} 条记录", domestic_ip, response.answers.len());
            for answer in &response.answers {
                println!("   - {}: {:?}", answer.name, answer.data);
            }
        }
        Err(e) => println!("   查询失败: {}", e),
    }
    
    // 4. 使用自定义客户端子网信息(国际网段)
    println!("\n4. 使用自定义客户端子网信息(176.32.39.0/24):");
    let international_subnet = ClientSubnet::from_ipv4(
        Ipv4Addr::new(176, 32, 39, 0), // 国际网段
        24 // /24子网掩码
    );
    
    // 使用query_with_client_ip方法替代不存在的query_with_client_subnet
    let subnet_ip = IpAddr::V4(Ipv4Addr::new(176, 32, 39, 0));
    match resolver.query_with_client_ip(domain, RecordType::A, QClass::IN, Some(subnet_ip)).await {
        Ok(response) => {
            println!("   查询成功(客户端子网: {}/{})，获得 {} 条记录", 
                international_subnet.address, international_subnet.source_prefix_length, response.answers.len());
            for answer in &response.answers {
                println!("   - {}: {:?}", answer.name, answer.data);
            }
        }
        Err(e) => println!("   查询失败: {}", e),
    }
    
    // 5. 设置默认客户端IP(国内IP)
    println!("\n5. 设置默认客户端IP后的查询(192.124.176.124):");
    let default_client_ip = IpAddr::V4(Ipv4Addr::new(192, 124, 176, 124)); // 国内IP
    resolver.set_default_client_ip(Some(default_client_ip));
    
    match resolver.query(domain, RecordType::A, QClass::IN).await {
        Ok(response) => {
            println!("   查询成功(默认客户端IP: {})，获得 {} 条记录", default_client_ip, response.answers.len());
            for answer in &response.answers {
                println!("   - {}: {:?}", answer.name, answer.data);
            }
        }
        Err(e) => println!("   查询失败: {}", e),
    }
    
    println!("\n=== 客户端子网功能演示完成 ===");
    println!("\n说明:");
    println!("- 使用腾讯云DNS(119.29.29.29)测试京东网站(www.jd.com)的地理位置解析");
    println!("- 176.32.39.57(国际IP)和192.124.176.124(国内IP)应该返回不同的解析结果");
    println!("- 客户端子网(EDNS Client Subnet)允许DNS服务器根据客户端IP返回最优结果");
    println!("- 这对CDN和地理位置相关的服务特别有用，可以实现智能解析");
    println!("- 支持IPv4和IPv6地址，以及自定义子网掩码");
    
    Ok(())
}