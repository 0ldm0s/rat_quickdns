#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
🚀 RatQuickMem Python 简化压力测试

无需额外依赖的轻量级压力测试套件
测试场景：
1. 并发编解码测试
2. 大数据处理测试
3. 批量操作测试
4. 内存池验证测试
"""

import rat_quickmem_py as rat_quickmem
import threading
import time
import random
import gc
from concurrent.futures import ThreadPoolExecutor, as_completed
from typing import List, Dict, Any

class SimpleStressTest:
    def __init__(self):
        self.results = {}
        self.errors = []
        self.start_time = None
        
    def log(self, message: str):
        """带时间戳的日志输出"""
        timestamp = time.strftime("%H:%M:%S")
        print(f"[{timestamp}] {message}")
        
    def generate_test_data(self, size_kb: int) -> bytes:
        """生成指定大小的测试数据"""
        return bytes(random.getrandbits(8) for _ in range(size_kb * 1024))
        
    def test_concurrent_operations(self, num_threads: int = 20, operations_per_thread: int = 50):
        """🔥 并发操作压力测试"""
        self.log(f"开始并发测试: {num_threads} 线程 x {operations_per_thread} 操作")
        
        def worker(thread_id: int) -> Dict[str, Any]:
            results = {"success": 0, "errors": 0, "total_time": 0}
            
            for i in range(operations_per_thread):
                try:
                    start = time.time()
                    
                    # 随机大小的数据 (1KB - 50KB)
                    data_size = random.randint(1, 50)
                    test_data = self.generate_test_data(data_size)
                    
                    # 编码解码
                    encoded = rat_quickmem.encode(test_data)
                    decoded = rat_quickmem.decode(encoded)
                    
                    # 验证数据完整性
                    if decoded != test_data:
                        raise ValueError("数据验证失败")
                    
                    results["total_time"] += time.time() - start
                    results["success"] += 1
                    
                except Exception as e:
                    results["errors"] += 1
                    self.errors.append(f"Thread {thread_id}, Op {i}: {str(e)}")
                    
            return results
        
        start_time = time.time()
        
        with ThreadPoolExecutor(max_workers=num_threads) as executor:
            futures = [executor.submit(worker, i) for i in range(num_threads)]
            thread_results = [future.result() for future in as_completed(futures)]
        
        end_time = time.time()
        
        # 统计结果
        total_success = sum(r["success"] for r in thread_results)
        total_errors = sum(r["errors"] for r in thread_results)
        total_ops = total_success + total_errors
        total_time = sum(r["total_time"] for r in thread_results)
        
        self.results["concurrent_operations"] = {
            "total_operations": total_ops,
            "success_count": total_success,
            "error_count": total_errors,
            "success_rate": total_success / total_ops * 100 if total_ops > 0 else 0,
            "ops_per_second": total_ops / (end_time - start_time),
            "avg_operation_time_ms": (total_time / total_ops * 1000) if total_ops > 0 else 0,
            "duration_seconds": end_time - start_time
        }
        
        self.log(f"✅ 并发测试完成: {total_success}/{total_ops} 成功, "
                f"{self.results['concurrent_operations']['ops_per_second']:.1f} ops/s")
    
    def test_large_data_processing(self, data_sizes_mb: List[int] = [1, 2, 5, 10]):
        """📦 大数据处理测试"""
        self.log(f"开始大数据处理测试: {data_sizes_mb} MB")
        
        large_data_results = []
        
        for size_mb in data_sizes_mb:
            try:
                self.log(f"  测试 {size_mb}MB 数据...")
                
                # 生成大数据
                large_data = self.generate_test_data(size_mb * 1024)
                
                # 编码测试
                encode_start = time.time()
                encoded = rat_quickmem.encode(large_data)
                encode_time = time.time() - encode_start
                
                # 解码测试
                decode_start = time.time()
                decoded = rat_quickmem.decode(encoded)
                decode_time = time.time() - decode_start
                
                # 验证数据完整性
                if decoded != large_data:
                    raise ValueError("大数据验证失败")
                
                result = {
                    "size_mb": size_mb,
                    "original_size": len(large_data),
                    "encoded_size": len(encoded),
                    "encode_time_ms": encode_time * 1000,
                    "decode_time_ms": decode_time * 1000,
                    "encode_speed_mbps": size_mb / encode_time if encode_time > 0 else 0,
                    "decode_speed_mbps": size_mb / decode_time if decode_time > 0 else 0,
                    "size_overhead_bytes": len(encoded) - len(large_data),
                    "overhead_percentage": (len(encoded) - len(large_data)) / len(large_data) * 100
                }
                
                large_data_results.append(result)
                self.log(f"    ✅ {size_mb}MB: 编码 {encode_time*1000:.1f}ms ({result['encode_speed_mbps']:.1f} MB/s), "
                        f"解码 {decode_time*1000:.1f}ms ({result['decode_speed_mbps']:.1f} MB/s)")
                self.log(f"    📊 开销: {result['size_overhead_bytes']} 字节 ({result['overhead_percentage']:.3f}%)")
                
                # 清理内存
                del large_data, encoded, decoded
                gc.collect()
                
            except Exception as e:
                self.log(f"    ❌ {size_mb}MB 测试失败: {str(e)}")
                self.errors.append(f"Large data {size_mb}MB: {str(e)}")
        
        self.results["large_data_processing"] = large_data_results
        self.log("✅ 大数据处理测试完成")
    
    def test_batch_operations(self):
        """📊 批量操作测试"""
        self.log("开始批量操作测试")
        
        batch_sizes = [10, 50, 100, 500, 1000, 2000, 5000, 8000, 9000, 9500, 9900, 9999]
        batch_results = []
        
        for batch_size in batch_sizes:
            try:
                self.log(f"  测试批量大小: {batch_size}")
                
                # 生成批量数据
                batch_data = []
                for i in range(batch_size):
                    data = {
                        "id": i,
                        "value": f"test_data_{i}_{random.randint(1000, 9999)}",
                        "timestamp": time.time(),
                        "random_bytes": self.generate_test_data(1)[:100].hex()  # 100字节随机数据
                    }
                    batch_data.append(data)
                
                start_time = time.time()
                
                # 批量编码
                encoded_batch = rat_quickmem.encode_batch(batch_data)
                encode_time = time.time() - start_time
                
                # 批量解码
                decode_start = time.time()
                decoded_batch = rat_quickmem.decode_batch(encoded_batch)
                decode_time = time.time() - decode_start
                
                total_time = time.time() - start_time
                
                # 验证数据完整性
                if len(decoded_batch) != batch_size:
                    raise ValueError(f"批量大小不匹配: 期望 {batch_size}, 实际 {len(decoded_batch)}")
                
                if decoded_batch[0]["id"] != 0 or decoded_batch[-1]["id"] != batch_size - 1:
                    raise ValueError("批量数据顺序错误")
                
                result = {
                    "batch_size": batch_size,
                    "success": True,
                    "encode_time_ms": encode_time * 1000,
                    "decode_time_ms": decode_time * 1000,
                    "total_time_ms": total_time * 1000,
                    "items_per_second": batch_size / total_time if total_time > 0 else 0,
                    "encode_items_per_second": batch_size / encode_time if encode_time > 0 else 0,
                    "decode_items_per_second": batch_size / decode_time if decode_time > 0 else 0
                }
                
                batch_results.append(result)
                self.log(f"    ✅ {batch_size} 项: {result['total_time_ms']:.1f}ms "
                        f"({result['items_per_second']:.0f} items/s)")
                
            except Exception as e:
                result = {
                    "batch_size": batch_size,
                    "success": False,
                    "error": str(e)
                }
                batch_results.append(result)
                self.log(f"    ❌ {batch_size} 项失败: {str(e)}")
                
                # 如果达到限制，记录并继续测试更小的批量
                if "CountExceeded" in str(e) or "exceeded" in str(e).lower():
                    self.log(f"    📝 达到批量操作限制")
        
        self.results["batch_operations"] = batch_results
        self.log("✅ 批量操作测试完成")
    
    def test_memory_pool_verification(self, iterations: int = 500):
        """🏊 内存池验证测试"""
        self.log(f"开始内存池验证测试: {iterations} 次迭代")
        
        pool_stats_samples = []
        
        # 获取初始池状态
        initial_stats = rat_quickmem.get_pool_stats()
        pool_stats_samples.append({
            "iteration": 0,
            "small_buffers": initial_stats.small_buffers,
            "medium_buffers": initial_stats.medium_buffers,
            "large_buffers": initial_stats.large_buffers,
            "total_buffers": initial_stats.total_buffers
        })
        
        self.log(f"  初始池状态: {initial_stats.total_buffers} 个缓冲区")
        
        for i in range(iterations):
            try:
                # 随机大小数据 (1KB - 500KB)
                data_size = random.randint(1, 500)
                test_data = self.generate_test_data(data_size)
                
                # 编解码操作
                encoded = rat_quickmem.encode(test_data)
                decoded = rat_quickmem.decode(encoded)
                
                if decoded != test_data:
                    raise ValueError(f"数据验证失败 at iteration {i}")
                
                # 每100次记录池状态
                if (i + 1) % 100 == 0:
                    pool_stats = rat_quickmem.get_pool_stats()
                    pool_stats_samples.append({
                        "iteration": i + 1,
                        "small_buffers": pool_stats.small_buffers,
                        "medium_buffers": pool_stats.medium_buffers,
                        "large_buffers": pool_stats.large_buffers,
                        "total_buffers": pool_stats.total_buffers
                    })
                    
                    self.log(f"  进度: {i+1}/{iterations}, 池缓冲区: {pool_stats.total_buffers}")
                
            except Exception as e:
                self.errors.append(f"Memory pool iteration {i}: {str(e)}")
        
        # 获取最终池状态
        final_stats = rat_quickmem.get_pool_stats()
        
        self.results["memory_pool_verification"] = {
            "iterations": iterations,
            "initial_pool_stats": {
                "small_buffers": initial_stats.small_buffers,
                "medium_buffers": initial_stats.medium_buffers,
                "large_buffers": initial_stats.large_buffers,
                "total_buffers": initial_stats.total_buffers
            },
            "final_pool_stats": {
                "small_buffers": final_stats.small_buffers,
                "medium_buffers": final_stats.medium_buffers,
                "large_buffers": final_stats.large_buffers,
                "total_buffers": final_stats.total_buffers
            },
            "pool_growth": final_stats.total_buffers - initial_stats.total_buffers,
            "pool_stats_samples": pool_stats_samples
        }
        
        self.log(f"✅ 内存池测试完成: 缓冲区从 {initial_stats.total_buffers} 增长到 {final_stats.total_buffers}")
    
    def test_mixed_workload(self, duration_seconds: int = 60):
        """🔀 混合负载测试"""
        self.log(f"开始混合负载测试: {duration_seconds} 秒")
        
        start_time = time.time()
        end_time = start_time + duration_seconds
        
        operation_counts = {
            "single_encode": 0,
            "batch_encode": 0,
            "large_data": 0,
            "errors": 0
        }
        
        while time.time() < end_time:
            try:
                # 随机选择操作类型
                operation_type = random.choice(["single", "single", "batch", "large"])  # 单个操作权重更高
                
                if operation_type == "single":
                    # 单个编解码
                    data_size = random.randint(1, 100)  # 1KB - 100KB
                    test_data = self.generate_test_data(data_size)
                    encoded = rat_quickmem.encode(test_data)
                    decoded = rat_quickmem.decode(encoded)
                    
                    if decoded != test_data:
                        raise ValueError("单个编解码验证失败")
                    
                    operation_counts["single_encode"] += 1
                    
                elif operation_type == "batch":
                    # 小批量操作
                    batch_size = random.randint(5, 50)
                    batch_data = [f"batch_item_{i}_{random.randint(1000, 9999)}" for i in range(batch_size)]
                    
                    encoded_batch = rat_quickmem.encode_batch(batch_data)
                    decoded_batch = rat_quickmem.decode_batch(encoded_batch)
                    
                    if decoded_batch != batch_data:
                        raise ValueError("批量操作验证失败")
                    
                    operation_counts["batch_encode"] += 1
                    
                elif operation_type == "large":
                    # 大数据操作
                    data_size = random.randint(100, 1000)  # 100KB - 1MB
                    large_data = self.generate_test_data(data_size)
                    
                    encoded = rat_quickmem.encode(large_data)
                    decoded = rat_quickmem.decode(encoded)
                    
                    if decoded != large_data:
                        raise ValueError("大数据操作验证失败")
                    
                    operation_counts["large_data"] += 1
                
            except Exception as e:
                operation_counts["errors"] += 1
                self.errors.append(f"Mixed workload: {str(e)}")
        
        actual_duration = time.time() - start_time
        total_operations = sum(operation_counts.values()) - operation_counts["errors"]
        
        self.results["mixed_workload"] = {
            "duration_seconds": actual_duration,
            "operation_counts": operation_counts,
            "total_operations": total_operations,
            "ops_per_second": total_operations / actual_duration if actual_duration > 0 else 0,
            "error_rate": operation_counts["errors"] / (total_operations + operation_counts["errors"]) * 100 if total_operations > 0 else 0
        }
        
        self.log(f"✅ 混合负载测试完成: {total_operations} 操作, "
                f"{self.results['mixed_workload']['ops_per_second']:.1f} ops/s, "
                f"错误率 {self.results['mixed_workload']['error_rate']:.2f}%")
    
    def run_all_tests(self):
        """🚀 运行所有压力测试"""
        self.log("="*60)
        self.log("🚀 开始 RatQuickMem Python 简化压力测试")
        self.log("="*60)
        
        self.start_time = time.time()
        
        try:
            # 1. 并发操作测试
            self.test_concurrent_operations(num_threads=10, operations_per_thread=30)
            
            # 2. 大数据处理测试
            self.test_large_data_processing([1, 2, 5])
            
            # 3. 批量操作测试
            self.test_batch_operations()
            
            # 4. 内存池验证测试
            self.test_memory_pool_verification(iterations=300)
            
            # 5. 混合负载测试
            self.test_mixed_workload(duration_seconds=30)
            
        except KeyboardInterrupt:
            self.log("⚠️ 测试被用户中断")
        except Exception as e:
            self.log(f"❌ 测试套件执行错误: {str(e)}")
            self.errors.append(f"Test suite error: {str(e)}")
        
        # 生成测试报告
        self.generate_report()
    
    def generate_report(self):
        """📋 生成压力测试报告"""
        total_time = time.time() - self.start_time
        
        self.log("\n" + "="*60)
        self.log("📋 简化压力测试报告")
        self.log("="*60)
        
        # 总体统计
        self.log(f"⏱️  总测试时间: {total_time:.1f} 秒")
        self.log(f"❌ 总错误数: {len(self.errors)}")
        
        # 各项测试结果
        for test_name, result in self.results.items():
            self.log(f"\n🔍 {test_name.replace('_', ' ').title()}:")
            
            if test_name == "concurrent_operations":
                self.log(f"  - 成功率: {result['success_rate']:.1f}%")
                self.log(f"  - 吞吐量: {result['ops_per_second']:.1f} ops/s")
                self.log(f"  - 平均延迟: {result['avg_operation_time_ms']:.2f} ms")
                
            elif test_name == "large_data_processing":
                for item in result:
                    self.log(f"  - {item['size_mb']}MB: 编码 {item['encode_speed_mbps']:.1f} MB/s, "
                            f"解码 {item['decode_speed_mbps']:.1f} MB/s")
                    self.log(f"    开销: {item['size_overhead_bytes']} 字节 ({item['overhead_percentage']:.3f}%)")
                        
            elif test_name == "batch_operations":
                successful_batches = [r for r in result if r.get('success', False)]
                failed_batches = [r for r in result if not r.get('success', False)]
                
                if successful_batches:
                    max_batch = max(r['batch_size'] for r in successful_batches)
                    self.log(f"  - 最大成功批量: {max_batch}")
                    
                    # 显示性能最好的几个批量大小
                    top_performers = sorted(successful_batches, key=lambda x: x['items_per_second'], reverse=True)[:3]
                    for perf in top_performers:
                        self.log(f"  - {perf['batch_size']} 项: {perf['items_per_second']:.0f} items/s")
                
                if failed_batches:
                    self.log(f"  - 失败的批量大小: {[r['batch_size'] for r in failed_batches]}")
                    
            elif test_name == "memory_pool_verification":
                self.log(f"  - 缓冲区增长: {result['initial_pool_stats']['total_buffers']} → {result['final_pool_stats']['total_buffers']} (+{result['pool_growth']})")
                self.log(f"  - 最终池状态: 小型 {result['final_pool_stats']['small_buffers']}, "
                        f"中型 {result['final_pool_stats']['medium_buffers']}, "
                        f"大型 {result['final_pool_stats']['large_buffers']}")
                
            elif test_name == "mixed_workload":
                self.log(f"  - 运行时间: {result['duration_seconds']:.1f} 秒")
                self.log(f"  - 总操作数: {result['total_operations']}")
                self.log(f"  - 吞吐量: {result['ops_per_second']:.1f} ops/s")
                self.log(f"  - 错误率: {result['error_rate']:.2f}%")
                self.log(f"  - 操作分布: 单个 {result['operation_counts']['single_encode']}, "
                        f"批量 {result['operation_counts']['batch_encode']}, "
                        f"大数据 {result['operation_counts']['large_data']}")
        
        # 错误汇总
        if self.errors:
            self.log(f"\n❌ 错误详情 (前5个):")
            for error in self.errors[:5]:
                self.log(f"  - {error}")
            if len(self.errors) > 5:
                self.log(f"  ... 还有 {len(self.errors) - 5} 个错误")
        
        # 性能评级
        self.log(f"\n🏆 性能评级:")
        
        # 并发性能评级
        if "concurrent_operations" in self.results:
            concurrent_result = self.results["concurrent_operations"]
            if concurrent_result["success_rate"] >= 99 and concurrent_result["ops_per_second"] >= 500:
                self.log("  - 并发性能: 🌟🌟🌟 优秀")
            elif concurrent_result["success_rate"] >= 95 and concurrent_result["ops_per_second"] >= 200:
                self.log("  - 并发性能: 🌟🌟 良好")
            else:
                self.log("  - 并发性能: 🌟 一般")
        
        # 批量处理评级
        if "batch_operations" in self.results:
            batch_result = self.results["batch_operations"]
            successful_batches = [r for r in batch_result if r.get('success', False)]
            if successful_batches:
                max_batch = max(r['batch_size'] for r in successful_batches)
                if max_batch >= 5000:
                    self.log("  - 批量处理: 🌟🌟🌟 优秀")
                elif max_batch >= 1000:
                    self.log("  - 批量处理: 🌟🌟 良好")
                else:
                    self.log("  - 批量处理: 🌟 一般")
        
        # 稳定性评级
        if "mixed_workload" in self.results:
            mixed_result = self.results["mixed_workload"]
            if mixed_result["error_rate"] <= 0.1 and mixed_result["ops_per_second"] >= 100:
                self.log("  - 稳定性: 🌟🌟🌟 优秀")
            elif mixed_result["error_rate"] <= 1.0 and mixed_result["ops_per_second"] >= 50:
                self.log("  - 稳定性: 🌟🌟 良好")
            else:
                self.log("  - 稳定性: 🌟 一般")
        
        self.log("\n✅ 简化压力测试完成！")
        self.log("\n💡 提示:")
        self.log("  - 如需更详细的测试，请安装 psutil 并运行 stress_test.py")
        self.log("  - 测试结果仅供参考，实际性能可能因环境而异")
        self.log("  - 如发现问题，请检查错误详情并调整测试参数")

def main():
    """主函数"""
    print("🚀 RatQuickMem Python 简化压力测试")
    print("无需额外依赖的轻量级测试套件")
    print("按 Ctrl+C 可随时中断测试\n")
    
    # 运行测试
    runner = SimpleStressTest()
    runner.run_all_tests()

if __name__ == "__main__":
    main()