//! SIMD 优化使用示例
//! 展示如何在实际应用中集成 SIMD 加速

use rat_quickmem::simd::{SimdMemcpy, SimdConfig, SimdCapabilities};
use std::time::Instant;

fn main() {
    println!("=== SIMD 优化使用示例 ===");
    
    // 1. 检测平台能力
    detect_platform_capabilities();
    
    // 2. 基本使用示例
    basic_usage_example();
    
    // 3. 配置优化示例
    configuration_example();
    
    // 4. 性能对比示例
    performance_comparison();
    
    // 5. 实际应用场景
    real_world_scenarios();
}

/// 检测平台 SIMD 能力
fn detect_platform_capabilities() {
    println!("\n1. 平台 SIMD 能力检测");
    println!("{}", "-".repeat(40));
    
    let caps = SimdCapabilities::detect();
    
    println!("当前平台: {} on {}", std::env::consts::ARCH, std::env::consts::OS);
    println!("SIMD 支持情况:");
    println!("  AVX2: {}", if caps.has_avx2 { "✓" } else { "✗" });
    println!("  SSE2: {}", if caps.has_sse2 { "✓" } else { "✗" });
    println!("  NEON: {}", if caps.has_neon { "✓" } else { "✗" });
    
    // 推荐配置
    let recommended_config = if caps.has_avx2 {
        "建议启用 AVX2 以获得最佳性能"
    } else if caps.has_sse2 {
        "建议启用 SSE2 以获得较好性能"
    } else if caps.has_neon {
        "建议启用 NEON 以获得较好性能"
    } else {
        "当前平台不支持 SIMD，将使用标准实现"
    };
    
    println!("推荐: {}", recommended_config);
}

/// 基本使用示例
fn basic_usage_example() {
    println!("\n2. 基本使用示例");
    println!("{}", "-".repeat(40));
    
    // 创建默认配置的 SIMD 实例
    let config = SimdConfig::default();
    let simd = SimdMemcpy::new(config);
    
    println!("使用的 SIMD 类型: {}", simd.get_active_simd());
    
    // 准备测试数据
    let src_data: Vec<u8> = (0..1024).map(|i| (i % 256) as u8).collect();
    let mut dst_data = vec![0u8; 1024];
    
    // 执行 SIMD 拷贝
    match simd.copy(&src_data, &mut dst_data) {
        Ok(()) => {
            println!("✓ SIMD 拷贝成功完成 (1024 字节)");
            
            // 验证数据正确性
            if src_data == dst_data {
                println!("✓ 数据完整性验证通过");
            } else {
                println!("✗ 数据完整性验证失败");
            }
        }
        Err(e) => {
            println!("✗ SIMD 拷贝失败: {}", e);
        }
    }
}

/// 配置优化示例
fn configuration_example() {
    println!("\n3. 配置优化示例");
    println!("{}", "-".repeat(40));
    
    let test_data: Vec<u8> = (0..2048).map(|i| (i % 256) as u8).collect();
    
    // 测试不同的最小块大小配置
    let chunk_sizes = [16, 32, 64, 128];
    
    for &chunk_size in &chunk_sizes {
        let config = SimdConfig {
            enable_avx2: true,
            enable_sse2: true,
            enable_neon: true,
            min_chunk_size: chunk_size,
        };
        
        let simd = SimdMemcpy::new(config);
        let mut dst = vec![0u8; test_data.len()];
        
        let start = Instant::now();
        for _ in 0..1000 {
            simd.copy(&test_data, &mut dst).unwrap();
        }
        let duration = start.elapsed();
        
        println!("最小块大小 {} 字节: {:?} (使用 {})", 
                chunk_size, duration, simd.get_active_simd());
    }
}

/// 性能对比示例
fn performance_comparison() {
    println!("\n4. 性能对比示例");
    println!("{}", "-".repeat(40));
    
    let sizes = [1024, 4096, 16384, 65536]; // 1KB, 4KB, 16KB, 64KB
    let iterations = 10000;
    
    for &size in &sizes {
        println!("\n测试数据大小: {} KB", size / 1024);
        
        let test_data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        
        // 标准拷贝性能
        let start = Instant::now();
        for _ in 0..iterations {
            let mut dst = vec![0u8; size];
            dst.copy_from_slice(&test_data);
            std::hint::black_box(dst);
        }
        let std_duration = start.elapsed();
        
        // SIMD 拷贝性能
        let config = SimdConfig::default();
        let simd = SimdMemcpy::new(config);
        
        let start = Instant::now();
        for _ in 0..iterations {
            let mut dst = vec![0u8; size];
            simd.copy(&test_data, &mut dst).unwrap();
            std::hint::black_box(dst);
        }
        let simd_duration = start.elapsed();
        
        let speedup = std_duration.as_nanos() as f64 / simd_duration.as_nanos() as f64;
        
        println!("  标准拷贝: {:?}", std_duration);
        println!("  SIMD 拷贝 ({}): {:?}", simd.get_active_simd(), simd_duration);
        println!("  加速比: {:.2}x", speedup);
        
        if speedup > 1.0 {
            println!("  ✓ SIMD 优化有效");
        } else if speedup > 0.8 {
            println!("  ~ SIMD 性能相当");
        } else {
            println!("  ⚠ SIMD 可能有开销");
        }
    }
}

/// 实际应用场景示例
fn real_world_scenarios() {
    println!("\n5. 实际应用场景");
    println!("{}", "-".repeat(40));
    
    // 场景1: 图像数据处理
    image_processing_scenario();
    
    // 场景2: 网络数据传输
    network_transfer_scenario();
    
    // 场景3: 文件批量处理
    batch_processing_scenario();
}

/// 图像数据处理场景
fn image_processing_scenario() {
    println!("\n场景1: 图像数据处理");
    
    // 模拟 1920x1080 RGB 图像数据
    let width = 1920;
    let height = 1080;
    let channels = 3;
    let image_size = width * height * channels;
    
    println!("处理 {}x{} RGB 图像 ({:.1} MB)", width, height, image_size as f64 / 1024.0 / 1024.0);
    
    let image_data: Vec<u8> = (0..image_size).map(|i| (i % 256) as u8).collect();
    let config = SimdConfig {
        enable_avx2: true,
        enable_sse2: true,
        enable_neon: true,
        min_chunk_size: 64, // 图像数据通常较大，使用较大的块
    };
    
    let simd = SimdMemcpy::new(config);
    let mut processed_data = vec![0u8; image_size];
    
    let start = Instant::now();
    simd.copy(&image_data, &mut processed_data).unwrap();
    let duration = start.elapsed();
    
    let throughput = image_size as f64 / duration.as_secs_f64() / 1024.0 / 1024.0 / 1024.0;
    
    println!("  处理时间: {:?}", duration);
    println!("  吞吐量: {:.2} GB/s", throughput);
    println!("  使用 SIMD: {}", simd.get_active_simd());
}

/// 网络数据传输场景
fn network_transfer_scenario() {
    println!("\n场景2: 网络数据传输");
    
    // 模拟网络包数据
    let packet_sizes = [64, 256, 1024, 1500]; // 常见网络包大小
    let packets_per_size = 10000;
    
    let config = SimdConfig {
        enable_avx2: true,
        enable_sse2: true,
        enable_neon: true,
        min_chunk_size: 32, // 网络包较小，使用较小的阈值
    };
    
    let simd = SimdMemcpy::new(config);
    
    for &packet_size in &packet_sizes {
        let packet_data: Vec<u8> = (0..packet_size).map(|i| (i % 256) as u8).collect();
        
        let start = Instant::now();
        for _ in 0..packets_per_size {
            let mut buffer = vec![0u8; packet_size];
            simd.copy(&packet_data, &mut buffer).unwrap();
            std::hint::black_box(buffer);
        }
        let duration = start.elapsed();
        
        let total_data = packet_size * packets_per_size;
        let throughput = total_data as f64 / duration.as_secs_f64() / 1024.0 / 1024.0;
        
        println!("  包大小 {} 字节: {:.2} MB/s ({} 包/秒)", 
                packet_size, throughput, 
                packets_per_size as f64 / duration.as_secs_f64() as f64);
    }
}

/// 文件批量处理场景
fn batch_processing_scenario() {
    println!("\n场景3: 文件批量处理");
    
    // 模拟批量处理多个文件
    let file_sizes = [4096, 16384, 65536, 262144]; // 4KB, 16KB, 64KB, 256KB
    let files_per_batch = 100;
    
    let config = SimdConfig::default();
    let simd = SimdMemcpy::new(config);
    
    for &file_size in &file_sizes {
        println!("\n  批量处理 {} 个 {} KB 文件:", files_per_batch, file_size / 1024);
        
        // 生成文件数据
        let files: Vec<Vec<u8>> = (0..files_per_batch)
            .map(|_| (0..file_size).map(|i| (i % 256) as u8).collect())
            .collect();
        
        let start = Instant::now();
        
        // 处理每个文件
        for file_data in &files {
            let mut processed = vec![0u8; file_data.len()];
            simd.copy(file_data, &mut processed).unwrap();
            
            // 模拟一些处理逻辑
            std::hint::black_box(processed);
        }
        
        let duration = start.elapsed();
        let total_data = file_size * files_per_batch;
        let throughput = total_data as f64 / duration.as_secs_f64() / 1024.0 / 1024.0;
        
        println!("    处理时间: {:?}", duration);
        println!("    吞吐量: {:.2} MB/s", throughput);
        println!("    平均每文件: {:?}", duration / files_per_batch as u32);
    }
}