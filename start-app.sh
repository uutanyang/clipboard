#!/bin/bash

# 停止所有 tauri 进程
echo "🛑 停止现有进程..."
pkill -9 -f tauri 2>/dev/null || true
pkill -9 -f vite 2>/dev/null || true
sleep 2

# 检查端口
echo "🔍 检查端口 1421..."
if lsof -ti:1421 >/dev/null 2>&1; then
    echo "⚠️  端口 1421 仍被占用，强制关闭..."
    lsof -ti:1421 | xargs kill -9 2>/dev/null || true
    sleep 1
fi

# 启动开发服务器
echo "🚀 启动开发服务器..."
cd "$(dirname "$0")"
npm run tauri dev &

echo "⏳ 等待应用启动..."
sleep 10

# 检查应用状态
echo "📊 检查应用状态..."
if ps aux | grep -v grep | grep "tauri-app" > /dev/null; then
    echo "✅ 应用已启动"
    echo ""
    echo "现在可以测试图片功能了："
    echo "1. 使用 Cmd+Shift+4 截图"
    echo "2. 查看应用窗口中的图片缩略图"
    echo "3. 点击图片复制到剪贴板"
    echo ""
    echo "查看日志: tail -f /tmp/tauri-dev.log"
else
    echo "❌ 应用启动失败"
    echo "查看日志: cat /tmp/tauri-dev.log"
fi
