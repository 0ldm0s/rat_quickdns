//! rat_quickmem 基础使用示例
//! 
//! 演示如何使用 rat_quickmem 进行数据编码和解码

use rat_quickmem::{encode, decode, QuickMemConfig};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== rat_quickmem Rust 基础示例 ===");
    
    // 1. 基础数据类型编码/解码
    basic_types_example()?;
    
    // 2. 复杂数据结构编码/解码
    complex_data_example()?;
    
    // 3. 自定义配置示例
    custom_config_example()?;
    
    println!("\n所有示例执行完成！");
    Ok(())
}

/// 基础数据类型示例
fn basic_types_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- 基础数据类型示例 ---");
    
    // 字符串
    let original_str = "Hello, rat_quickmem!";
    let encoded = encode(original_str)?;
    let decoded: String = decode(&encoded)?;
    println!("字符串: {} -> {} bytes -> {}", original_str, encoded.len(), decoded);
    
    // 数字
    let original_num = 42i32;
    let encoded = encode(&original_num)?;
    let decoded: i32 = decode(&encoded)?;
    println!("数字: {} -> {} bytes -> {}", original_num, encoded.len(), decoded);
    
    // 向量
    let original_vec = vec![1, 2, 3, 4, 5];
    let encoded = encode(&original_vec)?;
    let decoded: Vec<i32> = decode(&encoded)?;
    println!("向量: {:?} -> {} bytes -> {:?}", original_vec, encoded.len(), decoded);
    
    Ok(())
}

/// 复杂数据结构示例
fn complex_data_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- 复杂数据结构示例 ---");
    
    // HashMap
    let mut map = HashMap::new();
    map.insert("name".to_string(), "Alice".to_string());
    map.insert("age".to_string(), "30".to_string());
    map.insert("city".to_string(), "Beijing".to_string());
    
    let encoded = encode(&map)?;
    let decoded: HashMap<String, String> = decode(&encoded)?;
    println!("HashMap: {:?} -> {} bytes", map, encoded.len());
    println!("解码结果: {:?}", decoded);
    
    // 嵌套结构
    let nested_data = vec![
        vec!["row1_col1", "row1_col2"],
        vec!["row2_col1", "row2_col2"],
        vec!["row3_col1", "row3_col2"],
    ];
    
    let encoded = encode(&nested_data)?;
    let decoded: Vec<Vec<&str>> = decode(&encoded)?;
    println!("嵌套向量: {:?} -> {} bytes", nested_data, encoded.len());
    println!("解码结果: {:?}", decoded);
    
    Ok(())
}

/// 自定义配置示例
fn custom_config_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- 自定义配置示例 ---");
    
    // 创建自定义配置
    let config = QuickMemConfig {
        max_data_size: 1024 * 1024, // 1MB
        max_batch_count: 100,
        enable_compression: true,
    };
    
    println!("自定义配置: {:?}", config);
    
    // 大数据测试
    let large_data: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();
    let encoded = encode(&large_data)?;
    let decoded: Vec<u8> = decode(&encoded)?;
    
    println!("大数据测试: {} bytes -> {} bytes (压缩率: {:.2}%)", 
             large_data.len(), 
             encoded.len(),
             (1.0 - encoded.len() as f64 / large_data.len() as f64) * 100.0);
    
    assert_eq!(large_data, decoded);
    println!("数据完整性验证: ✓");
    
    Ok(())
}