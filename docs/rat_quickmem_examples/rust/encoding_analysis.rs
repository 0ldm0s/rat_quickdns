//! 分析 bincode 编码开销的测试程序

use rat_quickmem::{QuickEncoder, DataValue};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, bincode::Encode, bincode::Decode)]
struct SimpleData {
    data: Vec<u8>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let encoder = QuickEncoder::new();
    
    // 测试不同大小的原始数据
    let test_sizes = vec![100, 1000, 4000, 10000];
    
    println!("=== Bincode 编码开销分析 ===");
    println!("{:<12} {:<12} {:<12} {:<8}", "原始大小", "编码大小", "开销字节", "开销%");
    println!("{}", "-".repeat(50));
    
    for size in test_sizes {
        // 创建指定大小的数据
        let raw_data = vec![0u8; size];
        let test_data = SimpleData { data: raw_data };
        
        // 编码
        let encoded = encoder.encode(&test_data)?;
        let encoded_size = encoded.len();
        let overhead = encoded_size - size;
        let overhead_percent = (overhead as f64 / size as f64) * 100.0;
        
        println!("{:<12} {:<12} {:<12} {:<8.2}%", 
                size, encoded_size, overhead, overhead_percent);
    }
    
    println!("\n=== DataValue 编码分析 ===");
    
    // 测试 DataValue 的编码开销
    let data_4000 = vec![0u8; 4000];
    let data_value = DataValue::bytes(data_4000.clone());
    
    let encoded_dv = encoder.encode(&data_value)?;
    let dv_overhead = encoded_dv.len() - 4000;
    
    println!("DataValue(bytes) 原始: 4000 字节");
    println!("DataValue(bytes) 编码: {} 字节", encoded_dv.len());
    println!("DataValue(bytes) 开销: {} 字节", dv_overhead);
    
    // 分析开销来源
    println!("\n=== 开销来源分析 ===");
    println!("1. Vec<u8> 长度字段: 8 字节 (u64)");
    println!("2. 结构体字段标识: ~1-4 字节");
    println!("3. DataValue 枚举标识: ~1-8 字节");
    println!("4. 对齐填充: 0-7 字节");
    println!("\n总开销通常在 12-20 字节之间，这是 bincode 序列化的正常开销。");
    
    Ok(())
}