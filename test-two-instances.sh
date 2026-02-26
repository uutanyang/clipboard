#!/bin/bash

# 草果剪贴板 - 双实例测试脚本
# 用于在同一台机器上启动两个应用实例进行测试

set -e

PROJECT_DIR="/Users/yangtanfang/project/2026/4cool/clipboard-caoguo"
INSTANCE1_DIR="$PROJECT_DIR/instance1"
INSTANCE2_DIR="$PROJECT_DIR/instance2"
INSTANCE2_PORT=54322

echo "🍎 草果剪贴板 - 双实例测试"
echo "=========================="
echo ""

# 清理函数
cleanup() {
    echo ""
    echo "🧹 清理环境..."
    if [ -n "$PID1" ]; then
        echo "停止实例 1 (PID: $PID1)"
        kill $PID1 2>/dev/null || true
    fi
    if [ -n "$PID2" ]; then
        echo "停止实例 2 (PID: $PID2)"
        kill $PID2 2>/dev/null || true
    fi
    echo "✅ 清理完成"
}

# 设置退出时清理
trap cleanup EXIT INT TERM

# 函数：创建实例配置
create_instance_config() {
    local instance_dir="$1"
    local instance_name="$2"
    local instance_id="$3"
    local server_port="$4"

    echo "📝 创建实例配置: $instance_name"

    mkdir -p "$instance_dir"

    # 创建 tauri.conf.json
    cat > "$instance_dir/tauri.conf.json" <<EOF
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "$instance_name",
  "version": "0.1.0",
  "identifier": "$instance_id",
  "build": {
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:1421",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "title": "$instance_name",
        "width": 600,
        "height": 500,
        "resizable": false,
        "center": true,
        "decorations": false,
        "alwaysOnTop": true,
        "skipTaskbar": false,
        "visible": true
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
EOF

    # 创建链接到原始项目文件
    if [ ! -L "$instance_dir/src" ]; then
        ln -sf "$PROJECT_DIR/src" "$instance_dir/src"
    fi
    if [ ! -L "$instance_dir/public" ]; then
        ln -sf "$PROJECT_DIR/public" "$instance_dir/public"
    fi
    if [ ! -L "$instance_dir/index.html" ]; then
        ln -sf "$PROJECT_DIR/index.html" "$instance_dir/index.html"
    fi
    if [ ! -L "$instance_dir/package.json" ]; then
        ln -sf "$PROJECT_DIR/package.json" "$instance_dir/package.json"
    fi
    if [ ! -L "$instance_dir/package-lock.json" ]; then
        ln -sf "$PROJECT_DIR/package-lock.json" "$instance_dir/package-lock.json"
    fi
    if [ ! -L "$instance_dir/vite.config.ts" ]; then
        ln -sf "$PROJECT_DIR/vite.config.ts" "$instance_dir/vite.config.ts"
    fi
    if [ ! -L "$instance_dir/tsconfig.json" ]; then
        ln -sf "$PROJECT_DIR/tsconfig.json" "$instance_dir/tsconfig.json"
    fi
    if [ ! -L "$instance_dir/tsconfig.node.json" ]; then
        ln -sf "$PROJECT_DIR/tsconfig.node.json" "$instance_dir/tsconfig.node.json"
    fi

    echo "✅ 实例配置创建完成"
}

# 函数：修改服务器端口
modify_server_port() {
    local src_file="$PROJECT_DIR/src-tauri/src/server/mod.rs"
    local backup_file="$src_file.bak"
    local instance2_port=$1

    echo "🔧 修改服务器端口为 $instance2_port"

    # 备份原文件
    if [ ! -f "$backup_file" ]; then
        cp "$src_file" "$backup_file"
    fi

    # 修改默认端口
    sed -i.tmp "s/const DEFAULT_HTTP_SERVER_PORT: u16 = [0-9]*/const DEFAULT_HTTP_SERVER_PORT: u16 = $instance2_port/" "$src_file"
    rm -f "$src_file.tmp"

    echo "✅ 端口修改完成"
}

# 函数：恢复服务器端口
restore_server_port() {
    local src_file="$PROJECT_DIR/src-tauri/src/server/mod.rs"
    local backup_file="$src_file.bak"

    if [ -f "$backup_file" ]; then
        echo "🔄 恢复服务器端口"
        cp "$backup_file" "$src_file"
        rm -f "$backup_file"
        echo "✅ 端口恢复完成"
    fi
}

# 函数：启动实例
start_instance() {
    local instance_dir="$1"
    local instance_name="$2"
    local log_file="$3"

    echo "🚀 启动实例: $instance_name"
    cd "$instance_dir"

    # 使用不同的 Tauri 配置文件
    export TAURI_CONFIG="$instance_dir/tauri.conf.json"

    # 启动应用（开发模式）
    npm run tauri dev > "$log_file" 2>&1 &
    local pid=$!

    echo "✅ 实例 $instance_name 已启动 (PID: $pid)"
    echo "📝 日志文件: $log_file"

    echo "$pid"
}

# 主流程
main() {
    # 检查项目目录
    if [ ! -d "$PROJECT_DIR" ]; then
        echo "❌ 项目目录不存在: $PROJECT_DIR"
        exit 1
    fi

    echo "✅ 项目目录存在"
    echo ""

    # 修改实例 2 的服务器端口
    modify_server_port $INSTANCE2_PORT

    # 创建实例 1 配置
    create_instance_config "$INSTANCE1_DIR" "草果剪贴板-实例1" "com.yangtanfang.caoguo-clipboard.instance1"

    # 创建实例 2 配置
    create_instance_config "$INSTANCE2_DIR" "草果剪贴板-实例2" "com.yangtanfang.caoguo-clipboard.instance2"

    echo ""
    echo "================================"
    echo "准备启动两个应用实例"
    echo "================================"
    echo ""

    # 启动实例 1
    echo "📌 启动实例 1 (端口 54321)..."
    LOG1="$PROJECT_DIR/logs/instance1.log"
    mkdir -p "$(dirname "$LOG1")"
    PID1=$(start_instance "$INSTANCE1_DIR" "实例1" "$LOG1")
    sleep 5  # 等待实例 1 启动

    # 启动实例 2
    echo ""
    echo "📌 启动实例 2 (端口 $INSTANCE2_PORT)..."
    LOG2="$PROJECT_DIR/logs/instance2.log"
    mkdir -p "$(dirname "$LOG2")"
    PID2=$(start_instance "$INSTANCE2_DIR" "实例2" "$LOG2")
    sleep 5  # 等待实例 2 启动

    echo ""
    echo "================================"
    echo "✅ 两个实例已启动"
    echo "================================"
    echo ""
    echo "实例 1 (PID: $PID1):"
    echo "  - 配置: $INSTANCE1_DIR/tauri.conf.json"
    echo "  - 日志: $LOG1"
    echo "  - 端口: 54321"
    echo ""
    echo "实例 2 (PID: $PID2):"
    echo "  - 配置: $INSTANCE2_DIR/tauri.conf.json"
    echo "  - 日志: $LOG2"
    echo "  - 端口: $INSTANCE2_PORT"
    echo ""
    echo "================================"
    echo "📋 测试步骤:"
    echo "================================"
    echo ""
    echo "1. 检查两个实例是否正常启动"
    echo "   tail -f $LOG1"
    echo "   tail -f $LOG2"
    echo ""
    echo "2. 在两个实例中打开设备面板"
    echo ""
    echo "3. 验证设备发现是否正常"
    echo ""
    echo "4. 测试配对功能"
    echo ""
    echo "5. 测试剪贴板同步"
    echo ""
    echo "6. 测试文件传输"
    echo ""
    echo "================================"
    echo "💡 提示:"
    echo "================================"
    echo "- 按 Ctrl+C 停止所有实例"
    echo "- 查看日志: tail -f $LOG1 或 tail -f $LOG2"
    echo "- 服务器端口将在退出时自动恢复"
    echo ""
    echo "测试进行中... (按 Ctrl+C 退出)"
    echo ""

    # 等待用户按 Ctrl+C
    wait
}

# 运行主流程
main
