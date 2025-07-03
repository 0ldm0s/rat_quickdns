//! SIMD 与编解码器集成示例
//! 展示如何在 rat_quickmem 的编解码流程中使用 SIMD 优化

use rat_quickmem::{
    simd::{SimdMemcpy, SimdConfig, SimdCapabilities},
};
use serde::{Deserialize, Serialize};
use bincode::{Encode, Decode};
use std::time::Instant;

#[derive(Serialize, Deserialize, Encode, Decode, Debug, Clone, PartialEq)]
struct LargeImageData {
    width: u32,
    height: u32,
    channels: u8,
    pixels: Vec<u8>,
}

impl LargeImageData {
    fn new(width: u32, height: u32, channels: u8) -> Self {
        let size = (width * height * channels as u32) as usize;
        let pixels = (0..size).map(|i| (i % 256) as u8).collect();
        
        Self {
            width,
            height,
            channels,
            pixels,
        }
    }
    
    fn size_mb(&self) -> f64 {
        self.pixels.len() as f64 / 1024.0 / 1024.0
    }
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, Clone)]
struct NetworkPacket {
    id: u64,
    timestamp: u64,
    payload: Vec<u8>,
}

impl NetworkPacket {
    fn new(id: u64, payload_size: usize) -> Self {
        let payload = (0..payload_size).map(|i| ((i + id as usize) % 256) as u8).collect();
        
        Self {
            id,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            payload,
        }
    }
}

fn main() {
    println!("=== SIMD 与编解码器集成示例 ===");
    
    // 检测平台能力
    let caps = SimdCapabilities::detect();
    println!("\n平台 SIMD 能力: {:?}", caps);
    
    // 1. 大图像数据处理示例
    large_image_processing_example();
    
    // 2. 网络数据批处理示例
    network_batch_processing_example();
    
    // 3. 内存池与 SIMD 结合示例
    memory_pool_simd_example();
    
    // 4. 性能对比测试
    performance_comparison_example();
}

/// 大图像数据处理示例
fn large_image_processing_example() {
    println!("\n=== 1. 大图像数据处理示例 ===");
    
    // 创建一个 4K 图像数据
    let image = LargeImageData::new(3840, 2160, 3); // 4K RGB
    println!("图像大小: {}x{} ({:.1} MB)", image.width, image.height, image.size_mb());
    
    // 配置 SIMD 优化的编解码器
    let simd_config = SimdConfig {
        enable_avx2: true,
        enable_sse2: true,
        enable_neon: true,
        min_chunk_size: 64, // 大图像使用较大的块
    };
    
    let simd = SimdMemcpy::new(simd_config);
    println!("使用 SIMD: {}", simd.get_active_simd());
    
    // 模拟编码过程（序列化）
    let start = Instant::now();
    let encoded = bincode::encode_to_vec(&image, bincode::config::standard()).expect("编码失败");
    let encode_time = start.elapsed();
    
    println!("编码时间: {:?}", encode_time);
    println!("编码后大小: {:.2} MB", encoded.len() as f64 / 1024.0 / 1024.0);
    
    // 模拟解码过程（反序列化）
    let start = Instant::now();
    let (decoded, _): (LargeImageData, usize) = bincode::decode_from_slice(&encoded, bincode::config::standard()).expect("解码失败");
    let decode_time = start.elapsed();
    
    println!("解码时间: {:?}", decode_time);
    println!("数据完整性: {}", if image == decoded { "✓" } else { "✗" });
    
    // 模拟 SIMD 优化的像素数据拷贝
    let mut processed_pixels = vec![0u8; image.pixels.len()];
    let start = Instant::now();
    simd.copy(&image.pixels, &mut processed_pixels).expect("SIMD 拷贝失败");
    let simd_copy_time = start.elapsed();
    
    println!("SIMD 像素拷贝时间: {:?}", simd_copy_time);
    
    // 对比标准拷贝
    let mut std_pixels = vec![0u8; image.pixels.len()];
    let start = Instant::now();
    std_pixels.copy_from_slice(&image.pixels);
    let std_copy_time = start.elapsed();
    
    println!("标准拷贝时间: {:?}", std_copy_time);
    
    let speedup = std_copy_time.as_nanos() as f64 / simd_copy_time.as_nanos() as f64;
    println!("SIMD 加速比: {:.2}x", speedup);
}

/// 网络数据批处理示例
fn network_batch_processing_example() {
    println!("\n=== 2. 网络数据批处理示例 ===");
    
    // 生成网络包批次
    let packet_sizes = [64, 256, 1024, 1500]; // 常见网络包大小
    let packets_per_size = 1000;
    
    let simd_config = SimdConfig {
        min_chunk_size: 32, // 网络包较小，降低阈值
        ..Default::default()
    };
    
    let simd = SimdMemcpy::new(simd_config);
    
    for &packet_size in &packet_sizes {
        println!("\n处理 {} 字节网络包 (批次: {} 个)", packet_size, packets_per_size);
        
        // 生成测试包
        let packets: Vec<NetworkPacket> = (0..packets_per_size)
            .map(|i| NetworkPacket::new(i as u64, packet_size))
            .collect();
        
        // 批量编码
        let start = Instant::now();
        let encoded_packets: Vec<_> = packets
            .iter()
            .map(|packet| bincode::encode_to_vec(packet, bincode::config::standard()).expect("编码失败"))
            .collect();
        let encode_time = start.elapsed();
        
        // 批量解码
        let start = Instant::now();
        let decoded_packets: Vec<NetworkPacket> = encoded_packets
            .iter()
            .map(|data| {
                let (packet, _): (NetworkPacket, usize) = bincode::decode_from_slice(data, bincode::config::standard()).expect("解码失败");
                packet
            })
            .collect();
        let decode_time = start.elapsed();
        
        // 模拟 SIMD 优化的负载拷贝
        let start = Instant::now();
        for (original, decoded) in packets.iter().zip(decoded_packets.iter()) {
            let mut buffer = vec![0u8; decoded.payload.len()];
            simd.copy(&decoded.payload, &mut buffer).expect("SIMD 拷贝失败");
        }
        let simd_copy_time = start.elapsed();
        
        let total_data = packet_size * packets_per_size;
        let encode_throughput = total_data as f64 / encode_time.as_secs_f64() / 1024.0 / 1024.0;
        let decode_throughput = total_data as f64 / decode_time.as_secs_f64() / 1024.0 / 1024.0;
        let copy_throughput = total_data as f64 / simd_copy_time.as_secs_f64() / 1024.0 / 1024.0;
        
        println!("  编码吞吐量: {:.2} MB/s", encode_throughput);
        println!("  解码吞吐量: {:.2} MB/s", decode_throughput);
        println!("  SIMD 拷贝吞吐量: {:.2} MB/s", copy_throughput);
        
        // 验证数据完整性
        let integrity_ok = packets.iter().zip(decoded_packets.iter())
            .all(|(orig, decoded)| orig.payload == decoded.payload);
        println!("  数据完整性: {}", if integrity_ok { "✓" } else { "✗" });
    }
}

/// 内存池与 SIMD 结合示例
fn memory_pool_simd_example() {
    println!("\n=== 3. 内存池与 SIMD 结合示例 ===");
    
    // 模拟内存池功能（简化版）
    let mut buffers: Vec<Vec<u8>> = Vec::new();
    let simd = SimdMemcpy::new(SimdConfig::default());
    
    // 模拟高频数据处理场景
    let data_sizes = [1024, 4096, 16384]; // 1KB, 4KB, 16KB
    let iterations = 10000;
    
    for &size in &data_sizes {
        println!("\n处理 {} KB 数据 ({} 次迭代)", size / 1024, iterations);
        
        // 生成测试数据
        let test_data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        
        let start = Instant::now();
        
        for _ in 0..iterations {
            // 从内存池获取缓冲区
            let mut buffer = vec![0u8; size];
            
            // 使用 SIMD 优化拷贝
            simd.copy(&test_data, &mut buffer).expect("SIMD 拷贝失败");
            
            // 模拟一些处理
            std::hint::black_box(&buffer);
            
            // 将缓冲区添加到池中
            buffers.push(buffer);
        }
        
        let duration = start.elapsed();
        let throughput = (size * iterations) as f64 / duration.as_secs_f64() / 1024.0 / 1024.0;
        
        println!("  总时间: {:?}", duration);
        println!("  吞吐量: {:.2} MB/s", throughput);
        println!("  平均每次: {:?}", duration / iterations as u32);
        
        // 显示缓冲区统计
        println!("  缓冲区数量: {}", buffers.len());
    }
    
    // 模拟大量小块内存分配和拷贝
    let block_count = 1000;
    let block_size = 4096;
    
    let start = Instant::now();
    for i in 0..block_count {
        let mut buffer = vec![0u8; block_size];
        let data: Vec<u8> = (0..block_size).map(|j| ((i + j) % 256) as u8).collect();
        
        // 使用 SIMD 优化的内存拷贝
        simd.copy(&data, &mut buffer).expect("SIMD 拷贝失败");
        buffers.push(buffer);
    }
    let total_time = start.elapsed();
    
    println!("处理 {} 个 {} 字节块", block_count, block_size);
    println!("总时间: {:?}", total_time);
    println!("平均每块: {:?}", total_time / block_count as u32);
    println!("吞吐量: {:.2} MB/s", 
        (block_count * block_size) as f64 / 1024.0 / 1024.0 / total_time.as_secs_f64());
    
    println!("分配的缓冲区数量: {}", buffers.len());
}

/// 性能对比测试
fn performance_comparison_example() {
    println!("\n=== 4. 性能对比测试 ===");
    
    let test_sizes = [1024, 8192, 32768, 131072]; // 1KB, 8KB, 32KB, 128KB
    let iterations = 1000;
    
    // 不同配置的 SIMD
    let configs = vec![
        ("标准拷贝", None),
        ("SIMD 默认", Some(SimdConfig::default())),
        ("SIMD 大块", Some(SimdConfig {
            min_chunk_size: 64,
            ..Default::default()
        })),
        ("SIMD 小块", Some(SimdConfig {
            min_chunk_size: 16,
            ..Default::default()
        })),
    ];
    
    println!("\n{:<12} {:<8} {:<12} {:<12} {:<10}", "配置", "大小", "时间", "吞吐量", "加速比");
    println!("{}", "-".repeat(60));
    
    for &size in &test_sizes {
        let test_data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        let mut baseline_time: Option<std::time::Duration> = None;
        
        for (config_name, simd_config) in &configs {
            let start = Instant::now();
            
            for _ in 0..iterations {
                let mut dst = vec![0u8; size];
                
                match simd_config {
                    Some(config) => {
                        let simd = SimdMemcpy::new(*config);
                        simd.copy(&test_data, &mut dst).expect("SIMD 拷贝失败");
                    }
                    None => {
                        dst.copy_from_slice(&test_data);
                    }
                }
                
                std::hint::black_box(dst);
            }
            
            let duration = start.elapsed();
            let throughput = (size * iterations) as f64 / duration.as_secs_f64() / 1024.0 / 1024.0;
            
            let speedup = if let Some(baseline) = baseline_time {
                baseline.as_nanos() as f64 / duration.as_nanos() as f64
            } else {
                baseline_time = Some(duration);
                1.0
            };
            
            println!("{:<12} {:<8} {:<12?} {:<12.2} {:<10.2}x", 
                    config_name, 
                    format!("{}KB", size / 1024),
                    duration,
                    throughput,
                    speedup);
        }
        
        println!();
    }
}