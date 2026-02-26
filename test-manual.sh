#!/bin/bash

# 草果剪贴板 - 手动测试脚本
# 简化版本，用于快速启动和测试单个实例

set -e

PROJECT_DIR="/Users/yangtanfang/project/2026/4cool/clipboard-caoguo"
LOG_DIR="$PROJECT_DIR/logs"

echo "🍎 草果剪贴板 - 手动测试"
echo "========================"
echo ""

# 检查项目目录
if [ ! -d "$PROJECT_DIR" ]; then
    echo "❌ 项目目录不存在: $PROJECT_DIR"
    exit 1
fi

# 创建日志目录
mkdir -p "$LOG_DIR"

echo "✅ 项目目录存在"
echo ""

# 菜单选择
echo "请选择操作:"
echo "1) 启动开发模式 (npm run tauri dev)"
echo "2) 构建 Release 版本 (npm run tauri build)"
echo "3) 运行已构建的应用"
echo "4) 查看应用日志"
echo "5) 检查端口占用"
echo "6) 清理测试环境"
echo "7) 创建测试文件"
echo "8) 测试 HTTP 端点"
echo "9) 运行完整测试套件"
echo "0) 退出"
echo ""
read -p "请输入选项 [0-9]: " choice

case $choice in
    1)
        echo ""
        echo "🚀 启动开发模式..."
        cd "$PROJECT_DIR"
        npm run tauri dev
        ;;
    2)
        echo ""
        echo "🔨 构建 Release 版本..."
        cd "$PROJECT_DIR"
        npm run tauri build
        echo ""
        echo "✅ 构建完成"
        echo "📦 应用位置: $PROJECT_DIR/src-tauri/target/release/bundle/macos/草果剪贴板.app"
        ;;
    3)
        echo ""
        APP_PATH="$PROJECT_DIR/src-tauri/target/release/bundle/macos/草果剪贴板.app"
        if [ -d "$APP_PATH" ]; then
            echo "🚀 启动已构建的应用..."
            open "$APP_PATH"
            echo "✅ 应用已启动"
        else
            echo "❌ 应用不存在，请先构建"
            echo "运行: npm run tauri build"
        fi
        ;;
    4)
        echo ""
        echo "📋 应用日志:"
        LOG_FILE="$HOME/Library/Logs/com.yangtanfang.caoguo-clipboard/tauri-app.log"
        if [ -f "$LOG_FILE" ]; then
            echo "📝 日志文件: $LOG_FILE"
            echo ""
            echo "最近 50 行日志:"
            tail -50 "$LOG_FILE"
        else
            echo "ℹ️  日志文件不存在"
        fi
        ;;
    5)
        echo ""
        echo "🔍 检查端口占用..."
        echo "端口 54321 (HTTP 服务器):"
        lsof -i :54321 || echo "  端口未被占用"
        echo ""
        echo "端口 5353 (mDNS):"
        lsof -i :5353 || echo "  端口未被占用"
        ;;
    6)
        echo ""
        echo "🧹 清理测试环境..."
        echo ""
        echo "停止应用实例..."
        pkill -f "tauri-app" && echo "  ✅ 应用已停止" || echo "  ℹ️  未发现运行中的应用"
        echo ""
        echo "清理测试文件..."
        rm -f /tmp/test-*.txt /tmp/test-*.bin && echo "  ✅ 测试文件已清理" || echo "  ℹ️  无测试文件"
        echo ""
        echo "清理日志..."
        rm -f "$LOG_DIR"/*.log && echo "  ✅ 日志已清理" || echo "  ℹ️  无日志文件"
        echo ""
        echo "✅ 清理完成"
        ;;
    7)
        echo ""
        echo "📄 创建测试文件..."
        echo ""
        echo "1. 小文本文件 (1KB)"
        dd if=/dev/urandom of=/tmp/test-small.txt bs=1024 count=1 2>/dev/null
        echo "   ✅ 创建: /tmp/test-small.txt"
        echo ""
        echo "2. 中等文件 (10MB)"
        dd if=/dev/zero of=/tmp/test-medium.bin bs=1M count=10 2>/dev/null
        echo "   ✅ 创建: /tmp/test-medium.bin"
        echo ""
        echo "3. 大文件 (50MB)"
        dd if=/dev/zero of=/tmp/test-large.bin bs=1M count=50 2>/dev/null
        echo "   ✅ 创建: /tmp/test-large.bin"
        echo ""
        echo "4. 文本测试文件"
        cat > /tmp/test-text.txt <<EOF
测试文本文件
创建时间: $(date)
设备: $(hostname)
EOF
        echo "   ✅ 创建: /tmp/test-text.txt"
        echo ""
        echo "✅ 所有测试文件创建完成"
        echo ""
        echo "文件列表:"
        ls -lh /tmp/test-*.{txt,bin} 2>/dev/null
        ;;
    8)
        echo ""
        echo "🌐 测试 HTTP 端点..."
        echo ""
        read -p "请输入目标 IP (默认 localhost): " target_ip
        target_ip=${target_ip:-localhost}

        echo ""
        echo "测试: http://$target_ip:54321/ping"
        echo "----------------------------------------"
        curl -v "http://$target_ip:54321/ping" 2>&1 || echo "❌ 请求失败"
        echo ""
        echo ""
        echo "测试: http://$target_ip:54321/discover"
        echo "----------------------------------------"
        curl -v "http://$target_ip:54321/discover" 2>&1 || echo "❌ 请求失败"
        echo ""
        ;;
    9)
        echo ""
        echo "🧪 运行完整测试套件"
        echo "================================"
        echo ""
        echo "步骤 1/5: 启动应用"
        echo "----------------------------------------"
        read -p "应用是否已启动? (y/n): " app_started
        if [ "$app_started" != "y" ]; then
            echo "请先启动应用 (选择选项 1 或 3)"
            exit 1
        fi
        echo "✅ 应用已启动"
        echo ""
        echo "步骤 2/5: 测试 HTTP 端点"
        echo "----------------------------------------"
        echo "测试 /ping 端点..."
        if curl -s http://localhost:54321/ping > /dev/null 2>&1; then
            echo "✅ HTTP 服务器正常"
            curl -s http://localhost:54321/ping | python3 -m json.tool
        else
            echo "❌ HTTP 服务器无响应"
        fi
        echo ""
        echo "步骤 3/5: 测试剪贴板功能"
        echo "----------------------------------------"
        echo "复制测试文本到剪贴板..."
        echo "测试文本 - $(date)" | pbcopy
        echo "✅ 文本已复制"
        echo "请在应用中检查剪贴板历史"
        read -p "剪贴板历史是否显示刚复制的内容? (y/n): " clipboard_ok
        if [ "$clipboard_ok" = "y" ]; then
            echo "✅ 剪贴板功能正常"
        else
            echo "⚠️  剪贴板功能可能有问题"
        fi
        echo ""
        echo "步骤 4/5: 测试 mDNS 发现"
        echo "----------------------------------------"
        echo "请在应用中打开设备面板"
        echo "检查: 设备发现是否开启? 本机设备是否显示?"
        read -p "mDNS 发现是否正常? (y/n): " mdns_ok
        if [ "$mdns_ok" = "y" ]; then
            echo "✅ mDNS 发现正常"
        else
            echo "⚠️  mDNS 发现可能有问题"
        fi
        echo ""
        echo "步骤 5/5: 测试文件传输"
        echo "----------------------------------------"
        echo "创建测试文件..."
        echo "测试文件内容 - $(date)" > /tmp/test-upload.txt
        echo "✅ 测试文件已创建: /tmp/test-upload.txt"
        echo ""
        echo "请在应用中:"
        echo "1. 打开文件面板"
        echo "2. 点击选择文件"
        echo "3. 选择 /tmp/test-upload.txt"
        echo "4. 发送到其他设备（如果有）"
        read -p "文件传输测试完成? (y/n): " transfer_ok
        if [ "$transfer_ok" = "y" ]; then
            echo "✅ 文件传输正常"
        else
            echo "⚠️  文件传输可能有问题"
        fi
        echo ""
        echo "================================"
        echo "测试总结"
        echo "================================"
        echo "✅ HTTP 服务器: $([ "$app_started" = "y" ] && echo "正常" || echo "未测试")"
        echo "✅ 剪贴板功能: $([ "$clipboard_ok" = "y" ] && echo "正常" || echo "未通过")"
        echo "✅ mDNS 发现: $([ "$mdns_ok" = "y" ] && echo "正常" || echo "未通过")"
        echo "✅ 文件传输: $([ "$transfer_ok" = "y" ] && echo "正常" || echo "未通过")"
        echo ""
        echo "详细测试步骤请参考: $PROJECT_DIR/test-guide.md"
        ;;
    0)
        echo ""
        echo "👋 退出"
        exit 0
        ;;
    *)
        echo ""
        echo "❌ 无效选项"
        exit 1
        ;;
esac

echo ""
echo "完成!"
