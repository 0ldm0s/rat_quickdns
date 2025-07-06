#!/bin/bash

# DNS 解析器重命名脚本 - 支持 msys2
# 将 Resolver 重命名为 CoreResolver，EasyDnsResolver 重命名为 SmartDnsResolver

set -e

echo "🔄 开始 DNS 解析器重命名操作..."

# 定义项目根目录
PROJECT_ROOT="$(cd "$(dirname "$0")" && pwd)"
SRC_DIR="$PROJECT_ROOT/src"

echo "📁 项目根目录: $PROJECT_ROOT"
echo "📁 源码目录: $SRC_DIR"

# 检查源码目录是否存在
if [ ! -d "$SRC_DIR" ]; then
    echo "❌ 错误: 源码目录不存在: $SRC_DIR"
    exit 1
fi

# 备份函数
backup_files() {
    local backup_dir="$PROJECT_ROOT/backup_$(date +%Y%m%d_%H%M%S)"
    echo "💾 创建备份目录: $backup_dir"
    mkdir -p "$backup_dir"
    
    # 备份所有 .rs 文件
    find "$SRC_DIR" -name "*.rs" -exec cp --parents {} "$backup_dir" \;
    echo "✅ 备份完成"
}

# 重命名函数
rename_resolvers() {
    echo "🔧 开始重命名操作..."
    
    # 查找所有 .rs 文件
    local rs_files=$(find "$SRC_DIR" -name "*.rs" -type f)
    
    if [ -z "$rs_files" ]; then
        echo "⚠️  警告: 未找到 .rs 文件"
        return 1
    fi
    
    echo "📝 找到 $(echo "$rs_files" | wc -l) 个 .rs 文件"
    
    # 1. 将 EasyDnsResolver 重命名为 SmartDnsResolver
    echo "🔄 重命名 EasyDnsResolver -> SmartDnsResolver"
    for file in $rs_files; do
        if grep -q "EasyDnsResolver" "$file"; then
            echo "  📝 处理文件: $file"
            # 使用 sed 进行替换（兼容 msys2）
            sed -i 's/EasyDnsResolver/SmartDnsResolver/g' "$file"
        fi
    done
    
    # 2. 将 Resolver 重命名为 CoreResolver（但不影响已经重命名的 SmartDnsResolver）
    echo "🔄 重命名 Resolver -> CoreResolver"
    for file in $rs_files; do
        if grep -q "\bResolver\b" "$file" && ! grep -q "SmartDnsResolver" "$file"; then
            echo "  📝 处理文件: $file"
            # 精确匹配 Resolver，避免影响其他包含 Resolver 的词
            sed -i 's/\bResolver\b/CoreResolver/g' "$file"
        fi
    done
    
    # 3. 处理特殊情况：结构体定义和 impl 块
    echo "🔧 处理特殊情况..."
    for file in $rs_files; do
        # 处理可能遗漏的 struct Resolver 定义
        if grep -q "struct.*Resolver[^a-zA-Z]" "$file"; then
            echo "  🔍 检查结构体定义: $file"
            sed -i 's/struct Resolver/struct CoreResolver/g' "$file"
        fi
        
        # 处理 impl Resolver 块
        if grep -q "impl.*Resolver[^a-zA-Z]" "$file"; then
            echo "  🔍 检查 impl 块: $file"
            sed -i 's/impl Resolver/impl CoreResolver/g' "$file"
        fi
    done
    
    echo "✅ 重命名操作完成"
}

# 验证函数
verify_changes() {
    echo "🔍 验证重命名结果..."
    
    local rs_files=$(find "$SRC_DIR" -name "*.rs" -type f)
    local easy_dns_count=$(grep -r "EasyDnsResolver" $rs_files | wc -l || echo "0")
    local smart_dns_count=$(grep -r "SmartDnsResolver" $rs_files | wc -l || echo "0")
    local old_resolver_count=$(grep -r "\bResolver\b" $rs_files | grep -v "SmartDnsResolver\|CoreResolver" | wc -l || echo "0")
    local core_resolver_count=$(grep -r "CoreResolver" $rs_files | wc -l || echo "0")
    
    echo "📊 重命名统计:"
    echo "  - EasyDnsResolver 剩余: $easy_dns_count"
    echo "  - SmartDnsResolver 新增: $smart_dns_count"
    echo "  - 旧 Resolver 剩余: $old_resolver_count"
    echo "  - CoreResolver 新增: $core_resolver_count"
    
    if [ "$easy_dns_count" -eq 0 ] && [ "$smart_dns_count" -gt 0 ] && [ "$core_resolver_count" -gt 0 ]; then
        echo "✅ 重命名验证通过"
        return 0
    else
        echo "⚠️  重命名可能不完整，请手动检查"
        return 1
    fi
}

# 主执行流程
main() {
    echo "🚀 DNS 解析器重命名脚本启动"
    echo "📋 操作计划:"
    echo "  1. EasyDnsResolver -> SmartDnsResolver"
    echo "  2. Resolver -> CoreResolver"
    echo ""
    
    # 询问用户确认
    read -p "❓ 是否继续执行重命名操作？(y/N): " -n 1 -r
    echo ""
    
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "❌ 操作已取消"
        exit 0
    fi
    
    # 执行操作
    backup_files
    rename_resolvers
    verify_changes
    
    echo ""
    echo "🎉 重命名操作完成！"
    echo "💡 建议执行以下命令验证编译:"
    echo "   cargo check"
    echo "   cargo build"
    echo "   cargo build --features pyo3"
}

# 执行主函数
main "$@"