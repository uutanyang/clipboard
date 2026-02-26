#!/bin/bash

# 草果剪贴板 - 快速启动脚本

echo "🍎 草果剪贴板 - 开发环境启动"
echo "================================"
echo ""

# 检查 Node.js 和 npm
if ! command -v node &> /dev/null; then
    echo "❌ 错误: Node.js 未安装"
    exit 1
fi

if ! command -v npm &> /dev/null; then
    echo "❌ 错误: npm 未安装"
    exit 1
fi

# 检查 Rust 和 Cargo
if ! command -v cargo &> /dev/null; then
    echo "❌ 错误: Rust/Cargo 未安装"
    exit 1
fi

echo "✅ 环境检查通过"
echo ""

# 检查并清理端口占用
PORT=1421
if lsof -Pi :$PORT -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo "⚠️  端口 $PORT 已被占用，正在清理..."
    lsof -ti:$PORT | xargs kill -9 2>/dev/null
    echo "✅ 端口 $PORT 已释放"
    echo ""
fi

# 检查依赖
if [ ! -d "node_modules" ]; then
    echo "📦 安装前端依赖..."
    npm install
    echo ""
fi

# 启动开发服务器
echo "🚀 启动开发服务器..."
echo "按 Ctrl+C 停止"
echo ""
npm run tauri dev
