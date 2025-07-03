//! 开箱即用的DNS解析器示例
//! 
//! 演示如何使用构造器模式和bincode2序列化功能

use rat_quickdns::{
    DnsResolverBuilder, EasyDnsResolver, DnsQueryRequest, DnsQueryResponse,
    create_dns_query, encode_dns_query, decode_dns_response, quick_dns,
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 开箱即用的DNS解析器示例 ===");
    
    // 1. 使用构造器模式创建解析器
    builder_example().await?;
    
    // 2. 使用便捷宏创建解析器
    macro_example().await?;
    
    // 3. 使用bincode2序列化功能
    serialization_example().await?;
    
    // 4. 批量查询示例
    batch_query_example().await?;
    
    println!("\n所有示例执行完成！");
    Ok(())
}

/// 构造器模式示例
async fn builder_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- 构造器模式示例 ---");
    
    // 创建自定义配置的解析器
    let resolver = DnsResolverBuilder::new()
        .add_udp_server("223.5.5.5", 53)      // 阿里DNS
        .add_udp_server("119.29.29.29", 53)   // 腾讯DNS
        .add_doh_server("https://dns.alidns.com/dns-query")
        .with_timeout(Duration::from_secs(3))
        .with_retry_count(2)
        .with_cache(true)
        .build()?;
    
    // 解析域名
    let ips = resolver.resolve("example.com").await?;
    println!("example.com 解析结果: {:?}", ips);
    
    // 解析特定记录类型
    let mx_records = resolver.resolve_type("gmail.com", "MX").await?;
    println!("gmail.com MX记录: {:?}", mx_records);
    
    Ok(())
}

/// 便捷宏示例
async fn macro_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- 便捷宏示例 ---");
    
    // 使用默认配置
    let resolver = quick_dns!()?;
    let ips = resolver.resolve("github.com").await?;
    println!("github.com 解析结果: {:?}", ips);
    
    // 自定义超时
    let resolver = quick_dns!(timeout = 5)?;
    let ips = resolver.resolve("microsoft.com").await?;
    println!("microsoft.com 解析结果: {:?}", ips);
    
    // 自定义服务器
    let resolver = quick_dns!(servers = ["8.8.8.8", "8.8.4.4"])?;
    let ips = resolver.resolve("google.com").await?;
    println!("google.com 解析结果: {:?}", ips);
    
    Ok(())
}

/// 序列化示例
async fn serialization_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- 序列化示例 ---");
    
    // 创建解析器
    let resolver = EasyDnsResolver::default()?;
    
    // 创建查询请求
    let request = create_dns_query("baidu.com", "A");
    println!("查询请求: {:?}", request);
    
    // 编码请求
    let encoded_request = encode_dns_query(&request)?;
    println!("编码后大小: {} 字节", encoded_request.len());
    
    // 处理编码请求
    let encoded_response = resolver.process_encoded_query(&encoded_request).await?;
    println!("编码响应大小: {} 字节", encoded_response.len());
    
    // 解码响应
    let response = decode_dns_response(&encoded_response)?;
    println!("解码响应: {}", response_to_string(&response));
    
    Ok(())
}

/// 批量查询示例
async fn batch_query_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- 批量查询示例 ---");
    
    // 创建解析器
    let resolver = EasyDnsResolver::quick_setup()?;
    
    // 创建批量查询请求
    let requests = vec![
        create_dns_query("baidu.com", "A"),
        create_dns_query("qq.com", "A"),
        create_dns_query("github.com", "A"),
        create_dns_query("gmail.com", "MX"),
    ];
    
    println!("批量查询 {} 个域名...", requests.len());
    
    // 执行批量查询
    let responses = resolver.process_batch_queries(requests).await?;
    
    // 输出结果
    for (i, response) in responses.iter().enumerate() {
        println!("\n查询 #{}: {}", i + 1, response_to_string(response));
    }
    
    Ok(())
}

/// 辅助函数：格式化响应
fn response_to_string(response: &DnsQueryResponse) -> String {
    if response.success {
        let records = response.records.iter()
            .map(|r| format!("{} ({})", r.value, r.record_type))
            .collect::<Vec<_>>()
            .join(", ");
        
        format!("成功 - {} 条记录 [{}] ({}ms)", 
                response.records.len(), 
                records,
                response.duration_ms)
    } else {
        format!("失败 - {}", response.error.as_deref().unwrap_or("未知错误"))
    }
}