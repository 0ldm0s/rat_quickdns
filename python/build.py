#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
rat-quickdns-py 构建脚本

用于构建和安装rat-quickdns-py Python绑定。
"""

import os
import sys
import subprocess
import shutil
import argparse
from pathlib import Path


def check_requirements():
    """检查构建环境要求"""
    print("检查构建环境...")
    
    # 检查Rust工具链
    try:
        rustc_version = subprocess.check_output(["rustc", "--version"], text=True)
        cargo_version = subprocess.check_output(["cargo", "--version"], text=True)
        print(f"发现Rust编译器: {rustc_version.strip()}")
        print(f"发现Cargo: {cargo_version.strip()}")
    except (subprocess.SubprocessError, FileNotFoundError):
        print("错误: 未找到Rust工具链。请安装Rust: https://rustup.rs/")
        return False
    
    # 检查Maturin
    try:
        maturin_version = subprocess.check_output(["maturin", "--version"], text=True)
        print(f"发现Maturin: {maturin_version.strip()}")
    except (subprocess.SubprocessError, FileNotFoundError):
        print("警告: 未找到Maturin。将尝试安装...")
        try:
            subprocess.check_call([sys.executable, "-m", "pip", "install", "maturin"])
            maturin_version = subprocess.check_output(["maturin", "--version"], text=True)
            print(f"已安装Maturin: {maturin_version.strip()}")
        except subprocess.SubprocessError:
            print("错误: 无法安装Maturin。请手动安装: pip install maturin")
            return False
    
    return True


def build_wheel(release=True, universal2=False, output_dir=None):
    """构建Python wheel包"""
    print("\n开始构建Python wheel包...")
    
    # 确定项目根目录
    script_dir = Path(__file__).resolve().parent
    project_root = script_dir.parent
    
    # 构建命令
    cmd = ["maturin", "build"]
    
    # 添加选项
    if release:
        cmd.append("--release")
    else:
        cmd.append("--debug")
    
    # 添加特性
    cmd.extend(["--features", "python-bindings"])
    
    # 指定输出目录
    if output_dir:
        output_path = Path(output_dir).resolve()
        cmd.extend(["--out", str(output_path)])
    else:
        output_path = project_root / "target" / "wheels"
        cmd.extend(["--out", str(output_path)])
    
    # macOS universal2支持
    if universal2 and sys.platform == "darwin":
        cmd.append("--universal2")
    
    # 执行构建
    print(f"执行命令: {' '.join(cmd)}")
    try:
        subprocess.check_call(cmd, cwd=str(project_root))
        print(f"\n构建成功! wheel包已保存到: {output_path}")
        return True
    except subprocess.SubprocessError as e:
        print(f"\n构建失败: {e}")
        return False


def install_wheel(develop=False):
    """安装构建好的wheel包"""
    print("\n开始安装Python包...")
    
    # 确定项目根目录
    script_dir = Path(__file__).resolve().parent
    project_root = script_dir.parent
    
    # 安装命令
    if develop:
        cmd = ["maturin", "develop"]
        if develop == "release":
            cmd.append("--release")
        cmd.extend(["--features", "python-bindings"])
    else:
        cmd = [sys.executable, "-m", "pip", "install", "--force-reinstall"]
        wheel_dir = project_root / "target" / "wheels"
        if not wheel_dir.exists() or not list(wheel_dir.glob("*.whl")):
            print("错误: 未找到wheel包。请先运行构建命令。")
            return False
        
        # 找到最新的wheel包
        wheels = list(wheel_dir.glob("*.whl"))
        latest_wheel = max(wheels, key=lambda p: p.stat().st_mtime)
        cmd.append(str(latest_wheel))
    
    # 执行安装
    print(f"执行命令: {' '.join(cmd)}")
    try:
        subprocess.check_call(cmd, cwd=str(project_root))
        print("\n安装成功!")
        return True
    except subprocess.SubprocessError as e:
        print(f"\n安装失败: {e}")
        return False


def run_tests():
    """运行测试"""
    print("\n开始运行测试...")
    
    # 确定测试目录
    script_dir = Path(__file__).resolve().parent
    tests_dir = script_dir / "tests"
    
    if not tests_dir.exists():
        print(f"错误: 未找到测试目录: {tests_dir}")
        return False
    
    # 运行测试
    try:
        subprocess.check_call([sys.executable, "-m", "unittest", "discover", "-s", str(tests_dir)], 
                             cwd=str(script_dir))
        print("\n测试通过!")
        return True
    except subprocess.SubprocessError as e:
        print(f"\n测试失败: {e}")
        return False


def run_example(example_name=None):
    """运行示例"""
    # 确定示例目录
    script_dir = Path(__file__).resolve().parent
    examples_dir = script_dir / "examples"
    
    if not examples_dir.exists():
        print(f"错误: 未找到示例目录: {examples_dir}")
        return False
    
    # 列出可用示例
    examples = [f.stem for f in examples_dir.glob("*.py")]
    
    if not examples:
        print("错误: 未找到示例文件")
        return False
    
    if example_name is None:
        # 显示可用示例列表
        print("\n可用示例:")
        for i, example in enumerate(examples, 1):
            print(f"  {i}. {example}")
        
        # 让用户选择
        try:
            choice = int(input("\n请选择要运行的示例 (输入编号): "))
            if choice < 1 or choice > len(examples):
                print("无效的选择")
                return False
            example_name = examples[choice - 1]
        except (ValueError, IndexError):
            print("无效的选择")
            return False
    elif example_name not in examples:
        print(f"错误: 未找到示例 '{example_name}'")
        print("可用示例: " + ", ".join(examples))
        return False
    
    # 运行选择的示例
    example_path = examples_dir / f"{example_name}.py"
    print(f"\n运行示例: {example_name}")
    
    try:
        subprocess.check_call([sys.executable, str(example_path)])
        print(f"\n示例 '{example_name}' 运行完成")
        return True
    except subprocess.SubprocessError as e:
        print(f"\n示例运行失败: {e}")
        return False


def clean():
    """清理构建文件"""
    print("\n清理构建文件...")
    
    # 确定项目根目录
    script_dir = Path(__file__).resolve().parent
    project_root = script_dir.parent
    
    # 要清理的目录
    dirs_to_clean = [
        project_root / "target",
        script_dir / "__pycache__",
        script_dir / "build",
        script_dir / "dist",
        script_dir / "*.egg-info",
    ]
    
    # 清理目录
    for dir_path in dirs_to_clean:
        if dir_path.exists():
            if dir_path.is_dir():
                print(f"删除目录: {dir_path}")
                shutil.rmtree(dir_path, ignore_errors=True)
            else:
                print(f"删除文件: {dir_path}")
                dir_path.unlink()
    
    # 清理__pycache__目录
    for pycache in script_dir.glob("**/__pycache__"):
        print(f"删除目录: {pycache}")
        shutil.rmtree(pycache, ignore_errors=True)
    
    print("清理完成")
    return True


def main():
    """主函数"""
    parser = argparse.ArgumentParser(description="rat-quickdns-py 构建工具")
    subparsers = parser.add_subparsers(dest="command", help="命令")
    
    # build命令
    build_parser = subparsers.add_parser("build", help="构建Python wheel包")
    build_parser.add_argument("--debug", action="store_true", help="构建调试版本")
    build_parser.add_argument("--universal2", action="store_true", help="构建macOS universal2包")
    build_parser.add_argument("--out", help="输出目录")
    
    # install命令
    install_parser = subparsers.add_parser("install", help="安装Python包")
    install_parser.add_argument("--develop", action="store_true", help="以开发模式安装")
    install_parser.add_argument("--release", action="store_true", help="以发布模式安装开发版本")
    
    # test命令
    subparsers.add_parser("test", help="运行测试")
    
    # example命令
    example_parser = subparsers.add_parser("example", help="运行示例")
    example_parser.add_argument("name", nargs="?", help="示例名称")
    
    # clean命令
    subparsers.add_parser("clean", help="清理构建文件")
    
    # all命令
    all_parser = subparsers.add_parser("all", help="构建、安装并测试")
    all_parser.add_argument("--debug", action="store_true", help="构建调试版本")
    
    args = parser.parse_args()
    
    if args.command == "build":
        if not check_requirements():
            return 1
        success = build_wheel(release=not args.debug, universal2=args.universal2, output_dir=args.out)
        return 0 if success else 1
    
    elif args.command == "install":
        if args.develop:
            if not check_requirements():
                return 1
            success = install_wheel(develop="release" if args.release else True)
        else:
            success = install_wheel()
        return 0 if success else 1
    
    elif args.command == "test":
        success = run_tests()
        return 0 if success else 1
    
    elif args.command == "example":
        success = run_example(args.name)
        return 0 if success else 1
    
    elif args.command == "clean":
        success = clean()
        return 0 if success else 1
    
    elif args.command == "all":
        if not check_requirements():
            return 1
        
        # 构建
        if not build_wheel(release=not args.debug):
            return 1
        
        # 安装
        if not install_wheel():
            return 1
        
        # 测试
        if not run_tests():
            return 1
        
        print("\n全部步骤完成!")
        return 0
    
    else:
        parser.print_help()
        return 0


if __name__ == "__main__":
    sys.exit(main())