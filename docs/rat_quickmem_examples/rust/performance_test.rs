//! rat_quickmem 性能测试示例
//! 
//! 演示 rat_quickmem 的性能特性和基准测试

use rat_quickmem::{encode, decode};
use std::time::{Duration, Instant};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== rat_quickmem 性能测试 ===");
    
    // 1. 编码性能测试
    encoding_performance_test()?;
    
    // 2. 解码性能测试
    decoding_performance_test()?;
    
    // 3. 内存使用效率测试
    memory_efficiency_test()?;
    
    // 4. 批量操作性能测试
    batch_operations_test()?;
    
    println!("\n性能测试完成！");
    Ok(())
}

/// 编码性能测试
fn encoding_performance_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- 编码性能测试 ---");
    
    let test_data = generate_test_data(10000);
    let iterations = 1000;
    
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = encode(&test_data)?;
    }
    let duration = start.elapsed();
    
    let avg_time = duration / iterations;
    let throughput = test_data.len() as f64 / avg_time.as_secs_f64() / 1024.0 / 1024.0;
    
    println!("数据大小: {} bytes", test_data.len());
    println!("迭代次数: {}", iterations);
    println!("平均编码时间: {:?}", avg_time);
    println!("编码吞吐量: {:.2} MB/s", throughput);
    
    Ok(())
}

/// 解码性能测试
fn decoding_performance_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- 解码性能测试 ---");
    
    let test_data = generate_test_data(10000);
    let encoded_data = encode(&test_data)?;
    let iterations = 1000;
    
    let start = Instant::now();
    for _ in 0..iterations {
        let _: Vec<String> = decode(&encoded_data)?;
    }
    let duration = start.elapsed();
    
    let avg_time = duration / iterations;
    let throughput = encoded_data.len() as f64 / avg_time.as_secs_f64() / 1024.0 / 1024.0;
    
    println!("编码数据大小: {} bytes", encoded_data.len());
    println!("迭代次数: {}", iterations);
    println!("平均解码时间: {:?}", avg_time);
    println!("解码吞吐量: {:.2} MB/s", throughput);
    
    Ok(())
}

/// 内存使用效率测试
fn memory_efficiency_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- 内存使用效率测试 ---");
    
    let test_sizes = vec![100, 1000, 10000, 50000];
    
    for size in test_sizes {
        let test_data = generate_test_data(size);
        let original_size = estimate_memory_size(&test_data);
        
        let encoded = encode(&test_data)?;
        let compression_ratio = encoded.len() as f64 / original_size as f64;
        
        println!("数据量: {} 项", size);
        println!("原始大小: {} bytes", original_size);
        println!("编码大小: {} bytes", encoded.len());
        println!("压缩比: {:.3}", compression_ratio);
        println!("节省空间: {:.1}%", (1.0 - compression_ratio) * 100.0);
        println!();
    }
    
    Ok(())
}

/// 批量操作性能测试
fn batch_operations_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- 批量操作性能测试 ---");
    
    let batch_sizes = vec![10, 50, 100, 500];
    
    for batch_size in batch_sizes {
        let mut batch_data = Vec::new();
        for _ in 0..batch_size {
            batch_data.push(generate_test_data(100));
        }
        
        // 批量编码测试
        let start = Instant::now();
        let mut encoded_batch = Vec::new();
        for data in &batch_data {
            encoded_batch.push(encode(data)?);
        }
        let encode_duration = start.elapsed();
        
        // 批量解码测试
        let start = Instant::now();
        for encoded in &encoded_batch {
            let _: Vec<String> = decode(encoded)?;
        }
        let decode_duration = start.elapsed();
        
        println!("批量大小: {} 项", batch_size);
        println!("批量编码时间: {:?}", encode_duration);
        println!("批量解码时间: {:?}", decode_duration);
        println!("单项平均编码时间: {:?}", encode_duration / batch_size);
        println!("单项平均解码时间: {:?}", decode_duration / batch_size);
        println!();
    }
    
    Ok(())
}

/// 生成测试数据
fn generate_test_data(size: usize) -> Vec<String> {
    (0..size)
        .map(|i| format!("test_string_{}_{}", i, "x".repeat(i % 50)))
        .collect()
}

/// 估算内存大小
fn estimate_memory_size(data: &[String]) -> usize {
    data.iter().map(|s| s.len() + std::mem::size_of::<String>()).sum()
}

/// 运行基准测试
#[allow(dead_code)]
fn run_benchmark<F>(name: &str, iterations: u32, mut operation: F) -> Duration
where
    F: FnMut() -> Result<(), Box<dyn std::error::Error>>,
{
    println!("运行基准测试: {}", name);
    
    let start = Instant::now();
    for i in 0..iterations {
        if let Err(e) = operation() {
            eprintln!("基准测试失败 (迭代 {}): {}", i, e);
            break;
        }
    }
    let duration = start.elapsed();
    
    println!("完成 {} 次迭代，总时间: {:?}", iterations, duration);
    println!("平均时间: {:?}", duration / iterations);
    
    duration
}