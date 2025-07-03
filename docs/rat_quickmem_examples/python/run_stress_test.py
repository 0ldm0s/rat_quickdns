#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
🚀 RatQuickMem 压力测试启动器

自动检测环境并选择合适的测试程序
"""

import sys
import subprocess
import os

def check_psutil_available():
    """检查 psutil 是否可用"""
    try:
        import psutil
        return True
    except ImportError:
        return False

def run_test(test_type="auto"):
    """运行压力测试"""
    print("🚀 RatQuickMem 压力测试启动器")
    print("=" * 50)
    
    # 检查当前目录
    current_dir = os.path.dirname(os.path.abspath(__file__))
    stress_test_path = os.path.join(current_dir, "stress_test.py")
    simple_test_path = os.path.join(current_dir, "simple_stress_test.py")
    
    if not os.path.exists(simple_test_path):
        print("❌ 错误: 找不到测试文件")
        print(f"请确保在正确的目录中运行: {current_dir}")
        return False
    
    # 自动选择测试类型
    if test_type == "auto":
        if check_psutil_available() and os.path.exists(stress_test_path):
            test_type = "full"
            print("✅ 检测到 psutil，将运行完整压力测试")
        else:
            test_type = "simple"
            if not check_psutil_available():
                print("⚠️  未检测到 psutil，将运行简化压力测试")
                print("💡 提示: 运行 'pip install psutil' 以启用完整测试")
            else:
                print("ℹ️  运行简化压力测试")
    
    print(f"📋 测试类型: {'完整压力测试' if test_type == 'full' else '简化压力测试'}")
    print("⏱️  预计耗时:", "10-15分钟" if test_type == "full" else "3-5分钟")
    print("\n按 Enter 开始测试，或 Ctrl+C 取消...")
    
    try:
        input()
    except KeyboardInterrupt:
        print("\n❌ 测试已取消")
        return False
    
    # 运行测试
    try:
        if test_type == "full":
            print("\n🚀 启动完整压力测试...")
            result = subprocess.run([sys.executable, "stress_test.py"], 
                                  cwd=current_dir, 
                                  capture_output=False)
        else:
            print("\n🚀 启动简化压力测试...")
            result = subprocess.run([sys.executable, "simple_stress_test.py"], 
                                  cwd=current_dir, 
                                  capture_output=False)
        
        if result.returncode == 0:
            print("\n✅ 测试完成！")
            return True
        else:
            print(f"\n❌ 测试失败，退出码: {result.returncode}")
            return False
            
    except KeyboardInterrupt:
        print("\n⚠️ 测试被用户中断")
        return False
    except Exception as e:
        print(f"\n❌ 运行测试时出错: {str(e)}")
        return False

def main():
    """主函数"""
    import argparse
    
    parser = argparse.ArgumentParser(description="RatQuickMem 压力测试启动器")
    parser.add_argument("--type", 
                       choices=["auto", "full", "simple"], 
                       default="auto",
                       help="测试类型 (auto: 自动选择, full: 完整测试, simple: 简化测试)")
    parser.add_argument("--install-deps", 
                       action="store_true",
                       help="安装完整测试所需的依赖")
    
    args = parser.parse_args()
    
    # 安装依赖
    if args.install_deps:
        print("📦 安装依赖: psutil")
        try:
            subprocess.run([sys.executable, "-m", "pip", "install", "psutil"], 
                         check=True)
            print("✅ 依赖安装完成")
        except subprocess.CalledProcessError as e:
            print(f"❌ 依赖安装失败: {e}")
            return
    
    # 运行测试
    success = run_test(args.type)
    
    if success:
        print("\n🎉 测试成功完成！")
        if args.type in ["auto", "full"] and check_psutil_available():
            print("📄 详细报告已保存到 stress_test_report.json")
    else:
        print("\n💡 故障排除建议:")
        print("  1. 检查 Python 环境和依赖")
        print("  2. 尝试运行简化测试: python run_stress_test.py --type simple")
        print("  3. 查看错误信息并调整测试参数")
        print("  4. 参考 README_STRESS_TESTS.md 获取更多帮助")

if __name__ == "__main__":
    main()