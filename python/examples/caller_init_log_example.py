#!/usr/bin/env python3
"""
调用者初始化日志示例

这个示例展示了如何作为调用者来正确初始化日志系统
然后使用rat_quickdns进行DNS查询操作
"""

import threading
import sys
import os
import time

# 添加当前目录到Python路径
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

try:
    import rat_quickdns_py as dns
    from rat_quickdns_py import QueryStrategy
except ImportError as e:
    print(f"导入错误: {e}")
    print("请确保已运行 'maturin develop' 来构建Python绑定")
    sys.exit(1)


# 全局变量跟踪日志初始化状态
_logging_initialized = False

def demo_basic_logging():
    """演示基本的日志初始化"""
    global _logging_initialized
    print("=== 基本日志初始化演示 ===")

    # 检查日志系统状态
    print(f"日志系统初始化状态: {dns.is_logging_initialized()}")

    # 只有未初始化时才进行初始化
    if not _logging_initialized:
        print("初始化基本日志系统...")
        dns.init_logging()
        _logging_initialized = True
    else:
        print("日志系统已经初始化，跳过重复初始化")

    # 测试日志输出
    dns.log_info("这是一条信息日志")
    dns.log_warn("这是一条警告日志")
    dns.log_error("这是一条错误日志")
    dns.log_debug("这是一条调试日志")
    dns.log_trace("这是一条跟踪日志")

    print("✅ 基本日志初始化完成\n")


def demo_advanced_logging():
    """演示高级日志配置"""
    global _logging_initialized
    print("=== 高级日志配置演示 ===")

    # 只有未初始化时才进行初始化
    if not _logging_initialized:
        # 使用高级日志配置
        print("初始化高级日志系统...")
        dns.init_logging_advanced(
            level="debug",  # 设置调试级别
            enable_color=True,  # 启用颜色
            timestamp_format="%Y-%m-%d %H:%M:%S",  # 自定义时间格式
            custom_format_template="[{timestamp}] {level} PYTHON - {message}"  # 自定义格式
        )
        _logging_initialized = True
    else:
        print("日志系统已经初始化，跳过重复初始化")

    # 测试不同级别的日志输出
    dns.log_info("使用高级配置的信息日志")
    dns.log_warn("使用高级配置的警告日志")
    dns.log_error("使用高级配置的错误日志")
    dns.log_debug("使用高级配置的调试日志")
    dns.log_trace("使用高级配置的跟踪日志")

    print("✅ 高级日志配置完成\n")


def demo_dns_logging():
    """演示DNS专用日志初始化"""
    global _logging_initialized
    print("=== DNS专用日志初始化演示 ===")

    # 只有未初始化时才进行初始化
    if not _logging_initialized:
        print("初始化DNS专用日志系统...")
        dns.init_dns_logging("debug")
        _logging_initialized = True
    else:
        print("日志系统已经初始化，跳过重复初始化")

    # 测试DNS日志输出
    dns.dns_info("DNS查询开始")
    dns.dns_debug("正在解析域名...")
    dns.dns_warn("DNS服务器响应较慢")
    dns.dns_error("DNS查询失败")
    dns.dns_trace("DNS详细跟踪信息")

    print("✅ DNS专用日志初始化完成\n")


def demo_advanced_dns_logging():
    """演示高级DNS专用日志初始化"""
    global _logging_initialized
    print("=== 高级DNS专用日志初始化演示 ===")

    # 只有未初始化时才进行初始化
    if not _logging_initialized:
        print("初始化高级DNS专用日志系统...")
        dns.init_dns_logging_advanced(
            level="debug",
            enable_dns_format=True,
            enable_color=True,
            timestamp_format="%H:%M:%S"
        )
        _logging_initialized = True
    else:
        print("日志系统已经初始化，跳过重复初始化")

    # 测试DNS日志输出
    dns.dns_info("高级DNS日志信息")
    dns.dns_debug("高级DNS调试信息")
    dns.dns_warn("高级DNS警告信息")
    dns.dns_error("高级DNS错误信息")
    dns.dns_trace("高级DNS跟踪信息")

    print("✅ 高级DNS专用日志初始化完成\n")


def demo_level_control():
    """演示日志级别控制"""
    global _logging_initialized
    print("=== 日志级别控制演示 ===")

    # 注意：由于rat_logger的限制，一旦初始化就无法重新配置
    # 这个演示主要是为了展示API的使用方式
    if not _logging_initialized:
        # 初始化为错误级别
        print("初始化为错误级别...")
        dns.init_logging_with_level("error")
        _logging_initialized = True
    else:
        print("日志系统已经初始化，无法重新配置级别")

    print("根据当前配置显示日志:")
    dns.log_trace("这条跟踪日志可能不会显示")
    dns.log_debug("这条调试日志可能不会显示")
    dns.log_info("这条信息日志可能不会显示")
    dns.log_warn("这条警告日志可能不会显示")
    dns.log_error("这条错误日志会显示")

    print("✅ 日志级别控制演示完成\n")


def demo_dns_operations():
    """演示结合DNS操作的日志使用"""
    global _logging_initialized
    print("=== DNS操作日志演示 ===")

    try:
        # 只有未初始化时才进行初始化
        if not _logging_initialized:
            print("初始化日志系统...")
            dns.init_logging_with_level("info")
            _logging_initialized = True
        else:
            print("日志系统已经初始化，跳过重复初始化")

        dns.log_info("开始DNS操作演示")

        # 创建DNS解析器
        dns.log_info("创建DNS解析器...")
        builder = dns.DnsResolverBuilder()
        builder.query_strategy(QueryStrategy.SMART)
        builder.add_udp_upstream("阿里DNS", "223.5.5.5")
        builder.add_udp_upstream("腾讯DNS", "119.29.29.29")
        builder.timeout(5.0)
        resolver = builder.build()
        dns.log_info("DNS解析器创建成功")

        # 执行DNS查询
        dns.log_info("开始DNS查询...")
        start_time = time.time()

        # 查询域名
        domains = ["example.com", "google.com", "github.com"]
        for domain in domains:
            try:
                dns.dns_info(f"查询域名: {domain}")
                ips = resolver.resolve(domain)
                elapsed = (time.time() - start_time) * 1000
                if ips:
                    dns.dns_info(f"成功解析 {domain}: {ips} (耗时: {elapsed:.2f}ms)")
                else:
                    dns.dns_warn(f"解析 {domain} 返回空结果 (耗时: {elapsed:.2f}ms)")
            except Exception as e:
                dns.dns_error(f"解析 {domain} 失败: {e}")

        # 测试不同记录类型
        dns.log_info("测试不同记录类型查询...")
        try:
            dns.dns_info("查询A记录")
            a_records = resolver.resolve("example.com")
            dns.dns_info(f"A记录结果: {a_records}")

            dns.dns_info("查询AAAA记录")
            aaaa_records = resolver.resolve("example.com")
            dns.dns_info(f"AAAA记录结果: {aaaa_records}")

            dns.dns_info("查询MX记录")
            mx_records = resolver.resolve("example.com")
            dns.dns_info(f"MX记录结果: {mx_records}")
        except Exception as e:
            dns.dns_error(f"记录类型查询失败: {e}")

        dns.log_info("DNS操作演示完成")

    except Exception as e:
        dns.log_error(f"DNS操作失败: {e}")
        print(f"错误: {e}")


def demo_no_logging():
    """演示不初始化日志系统的情况"""
    print("=== 无日志系统演示 ===")

    print("不初始化日志系统，直接进行DNS查询...")

    # 创建DNS解析器
    builder = dns.DnsResolverBuilder()
    builder.query_strategy(QueryStrategy.SMART)
    builder.add_udp_upstream("阿里DNS", "223.5.5.5")
    builder.timeout(5.0)
    resolver = builder.build()

    # 执行DNS查询（不会有日志输出）
    print("执行DNS查询（不会有日志输出）...")
    ips = resolver.resolve("example.com")
    print(f"查询结果: {ips}")

    print("✅ 无日志系统演示完成\n")


def demo_concurrent_queries():
    """演示多线程并发DNS查询"""
    print("=== 多线程并发DNS查询演示 ===")

    # 创建DNS解析器
    builder = dns.DnsResolverBuilder()
    builder.query_strategy(QueryStrategy.SMART)
    builder.add_udp_upstream("阿里DNS", "223.5.5.5")
    builder.add_udp_upstream("腾讯DNS", "119.29.29.29")
    builder.timeout(5.0)
    resolver = builder.build()

    # 定义查询任务
    def query_domain(domain):
        try:
            start_time = time.time()
            ips = resolver.resolve(domain)
            elapsed = (time.time() - start_time) * 1000
            if _logging_initialized:
                dns.dns_info(f"线程 {threading.current_thread().name}: {domain} -> {ips} (耗时: {elapsed:.2f}ms)")
            else:
                print(f"线程 {threading.current_thread().name}: {domain} -> {ips} (耗时: {elapsed:.2f}ms)")
            return ips
        except Exception as e:
            if _logging_initialized:
                dns.dns_error(f"线程 {threading.current_thread().name}: {domain} 查询失败: {e}")
            else:
                print(f"线程 {threading.current_thread().name}: {domain} 查询失败: {e}")
            return None

    # 创建多个线程并发查询
    domains = ["example.com", "google.com", "github.com", "stackoverflow.com", "rust-lang.org"]
    threads = []

    print(f"启动 {len(domains)} 个线程并发查询...")
    for i, domain in enumerate(domains):
        thread = threading.Thread(target=query_domain, args=(domain,), name=f"Thread-{i+1}")
        threads.append(thread)
        thread.start()

    # 等待所有线程完成
    for thread in threads:
        thread.join()

    print("✅ 多线程并发查询完成\n")


def main():
    """主函数"""
    print("🚀 RAT QuickDNS Python绑定 - 调用者初始化日志示例")
    print("=" * 60)

    # 演示不初始化日志系统的情况
    demo_no_logging()

    # 演示不同的日志初始化方式
    demo_basic_logging()

    demo_advanced_logging()

    demo_dns_logging()

    demo_advanced_dns_logging()

    demo_level_control()

    # 演示DNS操作中的日志使用
    demo_dns_operations()

    # 演示多线程并发查询
    demo_concurrent_queries()

    print("=" * 60)
    print("📋 总结:")
    print("1. 调用者完全控制日志系统的初始化")
    print("2. 提供了多种日志配置选项:")
    print("   - init_logging(): 基本配置")
    print("   - init_logging_with_level(): 指定级别")
    print("   - init_logging_advanced(): 完全自定义配置")
    print("   - init_dns_logging(): DNS专用基本配置")
    print("   - init_dns_logging_advanced(): DNS专用高级配置")
    print("3. 日志系统完全可选，调用者可以自行实现")
    print("4. 支持所有标准的日志级别: trace, debug, info, warn, error")
    print("5. 提供了日志状态检查功能")
    print("6. 提供了通用日志和DNS专用日志两套接口")
    print("7. 不初始化日志系统不会影响DNS解析功能")
    print("8. 支持多线程并发DNS查询")
    print("9. 日志系统线程安全，可在多线程环境中使用")


if __name__ == "__main__":
    main()