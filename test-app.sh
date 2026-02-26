#!/bin/bash

echo "🍎 测试草果剪贴板应用"
echo "===================="
echo ""

APP_PATH="/Users/yangtanfang/project/2026/4cool/clipboard-caoguo/src-tauri/target/release/bundle/macos/草果剪贴板.app"

# 检查应用文件是否存在
if [ ! -d "$APP_PATH" ]; then
    echo "❌ 应用文件不存在: $APP_PATH"
    exit 1
fi

echo "✅ 应用文件存在"
echo ""

# 检查可执行文件
EXEC_PATH="$APP_PATH/Contents/MacOS/tauri-app"
if [ ! -f "$EXEC_PATH" ]; then
    echo "❌ 可执行文件不存在"
    exit 1
fi

echo "✅ 可执行文件存在"
echo "📊 文件大小: $(du -h "$EXEC_PATH" | cut -f1)"
echo ""

# 检查前端资源
RESOURCES="$APP_PATH/Contents/Resources"
echo "📁 Resources 目录内容:"
ls -lh "$RESOURCES"
echo ""

# 检查应用是否已在运行
if pgrep -f "tauri-app" > /dev/null; then
    echo "⚠️  应用已在运行"
    echo "🔍 进程信息:"
    ps aux | grep -i tauri-app | grep -v grep
    echo ""
else
    echo "ℹ️  应用未运行"
    echo ""
fi

# 检查应用日志
LOG_FILE="$HOME/Library/Logs/com.yangtanfang.caoguo-clipboard"
if [ -d "$LOG_FILE" ]; then
    echo "📋 应用日志:"
    ls -lh "$LOG_FILE"
    if [ -f "$LOG_FILE/tauri-app.log" ]; then
        echo ""
        echo "📝 最新日志 (最后 20 行):"
        tail -20 "$LOG_FILE/tauri-app.log"
    fi
else
    echo "ℹ️  未找到应用日志目录"
fi

echo ""
echo "🚀 启动应用..."
echo "提示: 应用启动后，按 Cmd+Shift+V 呼出窗口"
echo "按 Ctrl+C 停止此脚本（应用将继续运行）"

# 启动应用
open "$APP_PATH"
