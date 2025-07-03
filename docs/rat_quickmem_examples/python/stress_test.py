#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
🚀 RatQuickMem Python 压力测试套件

测试场景：
1. 高并发编解码压力测试
2. 大数据量处理压力测试
3. 内存池压力测试
4. 长时间运行稳定性测试
5. 批量操作极限测试
6. 混合负载压力测试
"""

import rat_quickmem_py as rat_quickmem
import threading
import time
import random
import gc
import psutil
import os
from concurrent.futures import ThreadPoolExecutor, as_completed
from typing import List, Dict, Any
import json

class StressTestRunner:
    def __init__(self):
        self.results = {}
        self.errors = []
        self.start_time = None
        self.process = psutil.Process(os.getpid())
        
    def log(self, message: str):
        """带时间戳的日志输出"""
        timestamp = time.strftime("%H:%M:%S")
        print(f"[{timestamp}] {message}")
        
    def get_memory_usage(self) -> Dict[str, float]:
        """获取当前内存使用情况"""
        memory_info = self.process.memory_info()
        return {
            "rss_mb": memory_info.rss / 1024 / 1024,  # 物理内存
            "vms_mb": memory_info.vms / 1024 / 1024,  # 虚拟内存
        }
        
    def generate_test_data(self, size_kb: int) -> bytes:
        """生成指定大小的测试数据"""
        return bytes(random.getrandbits(8) for _ in range(size_kb * 1024))
        
    def test_concurrent_encoding(self, num_threads: int = 50, operations_per_thread: int = 100):
        """🔥 高并发编码压力测试"""
        self.log(f"开始高并发编码测试: {num_threads} 线程 x {operations_per_thread} 操作")
        
        def encode_worker(thread_id: int) -> Dict[str, Any]:
            results = {"success": 0, "errors": 0, "total_time": 0}
            
            for i in range(operations_per_thread):
                try:
                    start = time.time()
                    
                    # 随机大小的数据 (1KB - 100KB)
                    data_size = random.randint(1, 100)
                    test_data = self.generate_test_data(data_size)
                    
                    # 编码
                    encoded = rat_quickmem.encode(test_data)
                    
                    # 验证
                    decoded = rat_quickmem.decode(encoded)
                    assert decoded == test_data
                    
                    results["total_time"] += time.time() - start
                    results["success"] += 1
                    
                except Exception as e:
                    results["errors"] += 1
                    self.errors.append(f"Thread {thread_id}, Op {i}: {str(e)}")
                    
            return results
        
        start_time = time.time()
        start_memory = self.get_memory_usage()
        
        with ThreadPoolExecutor(max_workers=num_threads) as executor:
            futures = [executor.submit(encode_worker, i) for i in range(num_threads)]
            thread_results = [future.result() for future in as_completed(futures)]
        
        end_time = time.time()
        end_memory = self.get_memory_usage()
        
        # 统计结果
        total_success = sum(r["success"] for r in thread_results)
        total_errors = sum(r["errors"] for r in thread_results)
        total_ops = total_success + total_errors
        avg_time = sum(r["total_time"] for r in thread_results) / total_ops if total_ops > 0 else 0
        
        self.results["concurrent_encoding"] = {
            "total_operations": total_ops,
            "success_rate": total_success / total_ops * 100 if total_ops > 0 else 0,
            "ops_per_second": total_ops / (end_time - start_time),
            "avg_operation_time_ms": avg_time * 1000,
            "memory_delta_mb": end_memory["rss_mb"] - start_memory["rss_mb"],
            "duration_seconds": end_time - start_time
        }
        
        self.log(f"✅ 并发编码测试完成: {total_success}/{total_ops} 成功, "
                f"{self.results['concurrent_encoding']['ops_per_second']:.1f} ops/s")
    
    def test_large_data_processing(self, data_sizes_mb: List[int] = [1, 5, 10, 20]):
        """📦 大数据量处理压力测试"""
        self.log(f"开始大数据处理测试: {data_sizes_mb} MB")
        
        large_data_results = []
        
        for size_mb in data_sizes_mb:
            try:
                self.log(f"  测试 {size_mb}MB 数据...")
                
                # 生成大数据
                large_data = self.generate_test_data(size_mb * 1024)
                
                start_memory = self.get_memory_usage()
                
                # 编码测试
                encode_start = time.time()
                encoded = rat_quickmem.encode(large_data)
                encode_time = time.time() - encode_start
                
                # 解码测试
                decode_start = time.time()
                decoded = rat_quickmem.decode(encoded)
                decode_time = time.time() - decode_start
                
                end_memory = self.get_memory_usage()
                
                # 验证数据完整性
                assert decoded == large_data
                
                result = {
                    "size_mb": size_mb,
                    "encode_time_ms": encode_time * 1000,
                    "decode_time_ms": decode_time * 1000,
                    "encode_speed_mbps": size_mb / encode_time,
                    "decode_speed_mbps": size_mb / decode_time,
                    "compression_ratio": len(encoded) / len(large_data),
                    "memory_peak_mb": end_memory["rss_mb"] - start_memory["rss_mb"]
                }
                
                large_data_results.append(result)
                self.log(f"    ✅ {size_mb}MB: 编码 {encode_time*1000:.1f}ms, "
                        f"解码 {decode_time*1000:.1f}ms")
                
                # 强制垃圾回收
                del large_data, encoded, decoded
                gc.collect()
                
            except Exception as e:
                self.log(f"    ❌ {size_mb}MB 测试失败: {str(e)}")
                self.errors.append(f"Large data {size_mb}MB: {str(e)}")
        
        self.results["large_data_processing"] = large_data_results
        self.log("✅ 大数据处理测试完成")
    
    def test_memory_pool_stress(self, iterations: int = 1000):
        """🏊 内存池压力测试"""
        self.log(f"开始内存池压力测试: {iterations} 次迭代")
        
        start_memory = self.get_memory_usage()
        pool_stats_history = []
        
        for i in range(iterations):
            try:
                # 随机大小数据 (1KB - 1MB)
                data_size = random.randint(1, 1024)
                test_data = self.generate_test_data(data_size)
                
                # 编解码
                encoded = rat_quickmem.encode(test_data)
                decoded = rat_quickmem.decode(encoded)
                assert decoded == test_data
                
                # 每100次记录池状态
                if i % 100 == 0:
                    pool_stats = rat_quickmem.get_pool_stats()
                    pool_stats_history.append({
                        "iteration": i,
                        "small_buffers": pool_stats.small_buffers,
                        "medium_buffers": pool_stats.medium_buffers,
                        "large_buffers": pool_stats.large_buffers,
                        "total_buffers": pool_stats.total_buffers,
                        "memory_mb": self.get_memory_usage()["rss_mb"]
                    })
                    
                    if i % 200 == 0:
                        self.log(f"  进度: {i}/{iterations}, 池缓冲区: {pool_stats.total_buffers}")
                
            except Exception as e:
                self.errors.append(f"Memory pool iteration {i}: {str(e)}")
        
        end_memory = self.get_memory_usage()
        final_pool_stats = rat_quickmem.get_pool_stats()
        
        self.results["memory_pool_stress"] = {
            "iterations": iterations,
            "final_pool_stats": {
                "small_buffers": final_pool_stats.small_buffers,
                "medium_buffers": final_pool_stats.medium_buffers,
                "large_buffers": final_pool_stats.large_buffers,
                "total_buffers": final_pool_stats.total_buffers
            },
            "memory_delta_mb": end_memory["rss_mb"] - start_memory["rss_mb"],
            "pool_stats_history": pool_stats_history
        }
        
        self.log(f"✅ 内存池测试完成: 最终缓冲区数量 {final_pool_stats.total_buffers}")
    
    def test_batch_operations_limit(self):
        """📊 批量操作极限测试"""
        self.log("开始批量操作极限测试")
        
        batch_sizes = [100, 500, 1000, 2000, 5000, 8000, 9000, 9500, 9900, 9999]
        batch_results = []
        
        for batch_size in batch_sizes:
            try:
                self.log(f"  测试批量大小: {batch_size}")
                
                # 生成批量数据
                batch_data = []
                for i in range(batch_size):
                    data = {
                        "id": i,
                        "data": self.generate_test_data(1).hex(),  # 1KB 数据转hex
                        "timestamp": time.time()
                    }
                    batch_data.append(data)
                
                start_time = time.time()
                
                # 批量编码
                encoded_batch = rat_quickmem.encode_batch(batch_data)
                
                # 批量解码
                decoded_batch = rat_quickmem.decode_batch(encoded_batch)
                
                end_time = time.time()
                
                # 验证数据完整性
                assert len(decoded_batch) == batch_size
                assert decoded_batch[0]["id"] == 0
                assert decoded_batch[-1]["id"] == batch_size - 1
                
                result = {
                    "batch_size": batch_size,
                    "success": True,
                    "total_time_ms": (end_time - start_time) * 1000,
                    "items_per_second": batch_size / (end_time - start_time)
                }
                
                batch_results.append(result)
                self.log(f"    ✅ {batch_size} 项: {result['total_time_ms']:.1f}ms")
                
            except Exception as e:
                result = {
                    "batch_size": batch_size,
                    "success": False,
                    "error": str(e)
                }
                batch_results.append(result)
                self.log(f"    ❌ {batch_size} 项失败: {str(e)}")
                
                # 如果达到限制，停止测试
                if "CountExceeded" in str(e) or "exceeded" in str(e).lower():
                    self.log(f"    达到批量操作限制，停止测试")
                    break
        
        self.results["batch_operations_limit"] = batch_results
        self.log("✅ 批量操作极限测试完成")
    
    def test_long_running_stability(self, duration_minutes: int = 5):
        """⏰ 长时间运行稳定性测试"""
        self.log(f"开始长时间稳定性测试: {duration_minutes} 分钟")
        
        start_time = time.time()
        end_time = start_time + (duration_minutes * 60)
        
        operation_count = 0
        error_count = 0
        memory_samples = []
        
        while time.time() < end_time:
            try:
                # 随机操作类型
                operation_type = random.choice(["single", "batch", "large"])
                
                if operation_type == "single":
                    # 单个编解码
                    data = self.generate_test_data(random.randint(1, 100))
                    encoded = rat_quickmem.encode(data)
                    decoded = rat_quickmem.decode(encoded)
                    assert decoded == data
                    
                elif operation_type == "batch":
                    # 小批量操作
                    batch_size = random.randint(10, 100)
                    batch_data = [f"item_{i}" for i in range(batch_size)]
                    encoded_batch = rat_quickmem.encode_batch(batch_data)
                    decoded_batch = rat_quickmem.decode_batch(encoded_batch)
                    assert decoded_batch == batch_data
                    
                elif operation_type == "large":
                    # 大数据操作
                    large_data = self.generate_test_data(random.randint(100, 1000))
                    encoded = rat_quickmem.encode(large_data)
                    decoded = rat_quickmem.decode(encoded)
                    assert decoded == large_data
                
                operation_count += 1
                
                # 每1000次操作记录内存
                if operation_count % 1000 == 0:
                    memory_info = self.get_memory_usage()
                    memory_samples.append({
                        "operation": operation_count,
                        "time": time.time() - start_time,
                        "memory_mb": memory_info["rss_mb"]
                    })
                    
                    elapsed_minutes = (time.time() - start_time) / 60
                    self.log(f"  运行 {elapsed_minutes:.1f}分钟: {operation_count} 操作, "
                            f"内存 {memory_info['rss_mb']:.1f}MB")
                
            except Exception as e:
                error_count += 1
                self.errors.append(f"Long running test operation {operation_count}: {str(e)}")
        
        total_time = time.time() - start_time
        
        self.results["long_running_stability"] = {
            "duration_minutes": total_time / 60,
            "total_operations": operation_count,
            "error_count": error_count,
            "success_rate": (operation_count - error_count) / operation_count * 100 if operation_count > 0 else 0,
            "ops_per_minute": operation_count / (total_time / 60),
            "memory_samples": memory_samples,
            "memory_growth_mb": memory_samples[-1]["memory_mb"] - memory_samples[0]["memory_mb"] if memory_samples else 0
        }
        
        self.log(f"✅ 长时间稳定性测试完成: {operation_count} 操作, "
                f"成功率 {self.results['long_running_stability']['success_rate']:.1f}%")
    
    def run_all_tests(self):
        """🚀 运行所有压力测试"""
        self.log("="*60)
        self.log("🚀 开始 RatQuickMem Python 压力测试套件")
        self.log("="*60)
        
        self.start_time = time.time()
        initial_memory = self.get_memory_usage()
        
        try:
            # 1. 高并发编码测试
            self.test_concurrent_encoding(num_threads=20, operations_per_thread=50)
            
            # 2. 大数据处理测试
            self.test_large_data_processing([1, 5, 10])
            
            # 3. 内存池压力测试
            self.test_memory_pool_stress(iterations=500)
            
            # 4. 批量操作极限测试
            self.test_batch_operations_limit()
            
            # 5. 长时间稳定性测试 (较短时间)
            self.test_long_running_stability(duration_minutes=2)
            
        except KeyboardInterrupt:
            self.log("⚠️ 测试被用户中断")
        except Exception as e:
            self.log(f"❌ 测试套件执行错误: {str(e)}")
            self.errors.append(f"Test suite error: {str(e)}")
        
        # 生成测试报告
        self.generate_report(initial_memory)
    
    def generate_report(self, initial_memory: Dict[str, float]):
        """📋 生成压力测试报告"""
        total_time = time.time() - self.start_time
        final_memory = self.get_memory_usage()
        
        self.log("\n" + "="*60)
        self.log("📋 压力测试报告")
        self.log("="*60)
        
        # 总体统计
        self.log(f"⏱️  总测试时间: {total_time:.1f} 秒")
        self.log(f"💾 内存使用变化: {final_memory['rss_mb'] - initial_memory['rss_mb']:+.1f} MB")
        self.log(f"❌ 总错误数: {len(self.errors)}")
        
        # 各项测试结果
        for test_name, result in self.results.items():
            self.log(f"\n🔍 {test_name.replace('_', ' ').title()}:")
            
            if test_name == "concurrent_encoding":
                self.log(f"  - 成功率: {result['success_rate']:.1f}%")
                self.log(f"  - 吞吐量: {result['ops_per_second']:.1f} ops/s")
                self.log(f"  - 平均延迟: {result['avg_operation_time_ms']:.2f} ms")
                
            elif test_name == "large_data_processing":
                for item in result:
                    if item.get('encode_speed_mbps'):
                        self.log(f"  - {item['size_mb']}MB: 编码 {item['encode_speed_mbps']:.1f} MB/s, "
                                f"解码 {item['decode_speed_mbps']:.1f} MB/s")
                        
            elif test_name == "memory_pool_stress":
                self.log(f"  - 最终缓冲区: {result['final_pool_stats']['total_buffers']}")
                self.log(f"  - 内存增长: {result['memory_delta_mb']:+.1f} MB")
                
            elif test_name == "batch_operations_limit":
                successful_batches = [r for r in result if r.get('success', False)]
                if successful_batches:
                    max_batch = max(r['batch_size'] for r in successful_batches)
                    self.log(f"  - 最大批量大小: {max_batch}")
                    
            elif test_name == "long_running_stability":
                self.log(f"  - 运行时间: {result['duration_minutes']:.1f} 分钟")
                self.log(f"  - 总操作数: {result['total_operations']}")
                self.log(f"  - 成功率: {result['success_rate']:.1f}%")
                self.log(f"  - 内存增长: {result['memory_growth_mb']:+.1f} MB")
        
        # 错误汇总
        if self.errors:
            self.log(f"\n❌ 错误详情 (前10个):")
            for error in self.errors[:10]:
                self.log(f"  - {error}")
            if len(self.errors) > 10:
                self.log(f"  ... 还有 {len(self.errors) - 10} 个错误")
        
        # 性能评级
        self.log(f"\n🏆 性能评级:")
        
        # 并发性能评级
        if "concurrent_encoding" in self.results:
            concurrent_result = self.results["concurrent_encoding"]
            if concurrent_result["success_rate"] >= 99 and concurrent_result["ops_per_second"] >= 1000:
                self.log("  - 并发性能: 🌟🌟🌟 优秀")
            elif concurrent_result["success_rate"] >= 95 and concurrent_result["ops_per_second"] >= 500:
                self.log("  - 并发性能: 🌟🌟 良好")
            else:
                self.log("  - 并发性能: 🌟 一般")
        
        # 稳定性评级
        if "long_running_stability" in self.results:
            stability_result = self.results["long_running_stability"]
            if stability_result["success_rate"] >= 99.9 and stability_result["memory_growth_mb"] < 10:
                self.log("  - 稳定性: 🌟🌟🌟 优秀")
            elif stability_result["success_rate"] >= 99 and stability_result["memory_growth_mb"] < 50:
                self.log("  - 稳定性: 🌟🌟 良好")
            else:
                self.log("  - 稳定性: 🌟 一般")
        
        self.log("\n✅ 压力测试完成！")
        
        # 保存详细结果到文件
        report_file = "stress_test_report.json"
        with open(report_file, 'w', encoding='utf-8') as f:
            json.dump({
                "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
                "total_time_seconds": total_time,
                "memory_delta_mb": final_memory['rss_mb'] - initial_memory['rss_mb'],
                "error_count": len(self.errors),
                "results": self.results,
                "errors": self.errors
            }, f, indent=2, ensure_ascii=False)
        
        self.log(f"📄 详细报告已保存到: {report_file}")

def main():
    """主函数"""
    print("🚀 RatQuickMem Python 压力测试")
    print("按 Ctrl+C 可随时中断测试\n")
    
    # 检查依赖
    try:
        import psutil
    except ImportError:
        print("❌ 缺少依赖: pip install psutil")
        return
    
    # 运行测试
    runner = StressTestRunner()
    runner.run_all_tests()

if __name__ == "__main__":
    main()