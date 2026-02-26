#!/bin/bash

# 草果剪贴板 - 完整功能测试脚本
# 测试 mDNS 发现、配对流程、文本同步、文件传输

set -e

PROJECT_DIR="/Users/yangtanfang/project/2026/4cool/clipboard-caoguo"
INSTANCE1_DIR="$PROJECT_DIR/instance1"
INSTANCE2_DIR="$PROJECT_DIR/instance2"
INSTANCE2_PORT=54322
LOG_DIR="$PROJECT_DIR/logs"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 测试结果统计
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_TOTAL=0

# 打印函数
print_header() {
    echo -e "\n${BLUE}================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}================================${NC}\n"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}ℹ $1${NC}"
}

print_test() {
    echo -e "\n${YELLOW}📋 测试: $1${NC}"
}

# 测试结果记录
record_test() {
    local test_name="$1"
    local result="$2"
    local details="$3"

    ((TESTS_TOTAL++))
    if [ "$result" = "PASS" ]; then
        ((TESTS_PASSED++))
        print_success "$test_name"
        echo "  详情: $details"
    else
        ((TESTS_FAILED++))
        print_error "$test_name"
        echo "  详情: $details"
    fi
    echo "[$result] $test_name: $details" >> "$LOG_DIR/test-results.log"
}

# 检查函数
check_http_server() {
    local port=$1
    local name=$2

    print_test "检查 $name HTTP 服务器 (端口 $port)"

    if curl -s --max-time 5 "http://localhost:$port/ping" > /dev/null 2>&1; then
        local response=$(curl -s "http://localhost:$port/ping")
        echo "  响应: $response"
        record_test "$name HTTP 服务器启动" "PASS" "端口 $port 响应正常"
        return 0
    else
        record_test "$name HTTP 服务器启动" "FAIL" "端口 $port 无响应"
        return 1
    fi
}

check_mdns_service() {
    local name=$1
    local log_file=$2

    print_test "检查 $name mDNS 服务"

    if grep -q "mDNS.*start" "$log_file" 2>/dev/null || grep -q "ServiceBrowser.*start" "$log_file" 2>/dev/null; then
        record_test "$name mDNS 服务启动" "PASS" "日志显示 mDNS 服务已启动"
        return 0
    else
        record_test "$name mDNS 服务启动" "FAIL" "日志中未找到 mDNS 启动信息"
        return 1
    fi
}

wait_for_mdns_discovery() {
    local seconds=$1
    print_info "等待 $seconds 秒以完成 mDNS 发现..."
    sleep $seconds
}

check_device_discovery() {
    local name=$1
    local log_file=$2

    print_test "检查 $name 设备发现"

    if grep -q "Device discovered" "$log_file" 2>/dev/null || grep -q "Device found" "$log_file" 2>/dev/null; then
        local discovered=$(grep -c "Device discovered\|Device found" "$log_file" 2>/dev/null || echo "0")
        record_test "$name 设备发现" "PASS" "发现 $discovered 个设备"
        return 0
    else
        record_test "$name 设备发现" "FAIL" "未发现其他设备"
        return 1
    fi
}

check_pairing_status() {
    local name=$1
    local log_file=$2

    print_test "检查 $name 配对状态"

    if grep -q "Pairing.*accepted\|Paired.*successfully" "$log_file" 2>/dev/null; then
        record_test "$name 配对成功" "PASS" "配对完成"
        return 0
    else
        record_test "$name 配对状态" "FAIL" "配对未完成"
        return 1
    fi
}

check_clipboard_sync() {
    local name=$1
    local log_file=$2

    print_test "检查 $name 剪贴板同步"

    if grep -q "Clipboard.*sync\|Received.*clipboard" "$log_file" 2>/dev/null; then
        local synced=$(grep -c "Clipboard.*sync\|Received.*clipboard" "$log_file" 2>/dev/null || echo "0")
        record_test "$name 剪贴板同步" "PASS" "同步了 $synced 条记录"
        return 0
    else
        record_test "$name 剪贴板同步" "FAIL" "未检测到剪贴板同步"
        return 1
    fi
}

check_file_transfer() {
    local name=$1
    local log_file=$2

    print_test "检查 $name 文件传输"

    if grep -q "File.*transfer.*complete\|Received.*file" "$log_file" 2>/dev/null; then
        record_test "$name 文件传输" "PASS" "文件传输完成"
        return 0
    else
        record_test "$name 文件传输" "FAIL" "未检测到文件传输"
        return 1
    fi
}

# 清理函数
cleanup() {
    echo ""
    print_header "清理环境"

    if [ -n "$PID1" ]; then
        print_info "停止实例 1 (PID: $PID1)"
        kill $PID1 2>/dev/null || true
    fi

    if [ -n "$PID2" ]; then
        print_info "停止实例 2 (PID: $PID2)"
        kill $PID2 2>/dev/null || true
    fi

    # 恢复服务器端口
    restore_server_port

    print_success "清理完成"
}

# 恢复服务器端口
restore_server_port() {
    local src_file="$PROJECT_DIR/src-tauri/src/server/mod.rs"
    local backup_file="$src_file.bak"

    if [ -f "$backup_file" ]; then
        print_info "恢复服务器端口"
        cp "$backup_file" "$src_file"
        rm -f "$backup_file"
        print_success "端口恢复完成"
    fi
}

# 设置退出时清理
trap cleanup EXIT INT TERM

# 创建测试文件
create_test_file() {
    local filename=$1
    local size_mb=$2

    print_info "创建测试文件: $filename (${size_mb}MB)"

    mkdir -p "$LOG_DIR/test-files"
    dd if=/dev/urandom of="$LOG_DIR/test-files/$filename" bs=1M count=$size_mb 2>/dev/null
    print_success "测试文件创建完成"
}

# 修改服务器端口
modify_server_port() {
    local src_file="$PROJECT_DIR/src-tauri/src/server/mod.rs"
    local backup_file="$src_file.bak"
    local instance2_port=$1

    print_info "修改实例 2 服务器端口为 $instance2_port"

    if [ ! -f "$backup_file" ]; then
        cp "$src_file" "$backup_file"
    fi

    sed -i.tmp "s/const DEFAULT_HTTP_SERVER_PORT: u16 = [0-9]*/const DEFAULT_HTTP_SERVER_PORT: u16 = $instance2_port/" "$src_file"
    rm -f "$src_file.tmp"

    print_success "端口修改完成"
}

# 启动实例
start_instance() {
    local instance_dir="$1"
    local instance_name="$2"
    local log_file="$3"

    print_info "启动实例: $instance_name"

    cd "$instance_dir"

    # 启动应用（开发模式）
    npm run tauri dev > "$log_file" 2>&1 &
    local pid=$!

    print_success "实例 $instance_name 已启动 (PID: $pid)"
    echo "  日志文件: $log_file"

    echo "$pid"
}

# 主测试流程
run_tests() {
    mkdir -p "$LOG_DIR"
    mkdir -p "$LOG_DIR/test-files"

    # 清空测试结果日志
    echo "草果剪贴板 - 功能测试报告" > "$LOG_DIR/test-results.log"
    echo "测试时间: $(date)" >> "$LOG_DIR/test-results.log"
    echo "================================" >> "$LOG_DIR/test-results.log"
    echo "" >> "$LOG_DIR/test-results.log"

    print_header "🍎 草果剪贴板 - 功能测试"

    # 步骤 1: 准备测试环境
    print_test "步骤 1: 准备测试环境"

    # 创建测试文件
    create_test_file "test-small.txt" 1
    create_test_file "test-medium.bin" 10
    create_test_file "test-large.bin" 50

    # 修改实例 2 的服务器端口
    modify_server_port $INSTANCE2_PORT

    print_info "等待 3 秒..."
    sleep 3

    # 步骤 2: 启动两个应用实例
    print_test "步骤 2: 启动两个应用实例"

    # 创建软链接
    if [ ! -L "$INSTANCE1_DIR/src" ]; then
        mkdir -p "$INSTANCE1_DIR"
        ln -sf "$PROJECT_DIR/src" "$INSTANCE1_DIR/src"
        ln -sf "$PROJECT_DIR/public" "$INSTANCE1_DIR/public"
        ln -sf "$PROJECT_DIR/index.html" "$INSTANCE1_DIR/index.html"
        ln -sf "$PROJECT_DIR/package.json" "$INSTANCE1_DIR/package.json"
        ln -sf "$PROJECT_DIR/package-lock.json" "$INSTANCE1_DIR/package-lock.json"
        ln -sf "$PROJECT_DIR/vite.config.ts" "$INSTANCE1_DIR/vite.config.ts"
        ln -sf "$PROJECT_DIR/tsconfig.json" "$INSTANCE1_DIR/tsconfig.json"
        ln -sf "$PROJECT_DIR/tsconfig.node.json" "$INSTANCE1_DIR/tsconfig.node.json"
        ln -sf "$PROJECT_DIR/src-tauri" "$INSTANCE1_DIR/src-tauri"
    fi

    if [ ! -L "$INSTANCE2_DIR/src" ]; then
        mkdir -p "$INSTANCE2_DIR"
        ln -sf "$PROJECT_DIR/src" "$INSTANCE2_DIR/src"
        ln -sf "$PROJECT_DIR/public" "$INSTANCE2_DIR/public"
        ln -sf "$PROJECT_DIR/index.html" "$INSTANCE2_DIR/index.html"
        ln -sf "$PROJECT_DIR/package.json" "$INSTANCE2_DIR/package.json"
        ln -sf "$PROJECT_DIR/package-lock.json" "$INSTANCE2_DIR/package-lock.json"
        ln -sf "$PROJECT_DIR/vite.config.ts" "$INSTANCE2_DIR/vite.config.ts"
        ln -sf "$PROJECT_DIR/tsconfig.json" "$INSTANCE2_DIR/tsconfig.json"
        ln -sf "$PROJECT_DIR/tsconfig.node.json" "$INSTANCE2_DIR/tsconfig.node.json"
        ln -sf "$PROJECT_DIR/src-tauri" "$INSTANCE2_DIR/src-tauri"
    fi

    # 启动实例 1
    print_info "启动实例 1 (端口 54321)..."
    LOG1="$LOG_DIR/instance1.log"
    PID1=$(start_instance "$INSTANCE1_DIR" "实例1" "$LOG1")
    sleep 10  # 等待实例 1 完全启动

    # 启动实例 2
    print_info "启动实例 2 (端口 $INSTANCE2_PORT)..."
    LOG2="$LOG_DIR/instance2.log"
    PID2=$(start_instance "$INSTANCE2_DIR" "实例2" "$LOG2")
    sleep 10  # 等待实例 2 完全启动

    print_success "两个实例已启动"
    echo "  实例 1 (PID: $PID1): 端口 54321, 日志: $LOG1"
    echo "  实例 2 (PID: $PID2): 端口 $INSTANCE2_PORT, 日志: $LOG2"

    # 步骤 3: 测试 HTTP 服务器
    print_header "步骤 3: 测试 HTTP 服务器"

    check_http_server 54321 "实例1"
    check_http_server $INSTANCE2_PORT "实例2"

    # 步骤 4: 测试 mDNS 服务
    print_header "步骤 4: 测试 mDNS 服务"

    check_mdns_service "实例1" "$LOG1"
    check_mdns_service "实例2" "$LOG2"

    # 步骤 5: 测试设备发现
    print_header "步骤 5: 测试设备发现"

    wait_for_mdns_discovery 10

    print_info "请手动执行以下操作:"
    echo "  1. 在实例 1 中点击 '设备' 按钮"
    echo "  2. 在实例 2 中点击 '设备' 按钮"
    echo "  3. 确认两个设备都能看到对方"
    echo ""
    read -p "按 Enter 键继续设备发现测试..." </dev/tty

    check_device_discovery "实例1" "$LOG1"
    check_device_discovery "实例2" "$LOG2"

    # 步骤 6: 测试配对流程
    print_header "步骤 6: 测试配对流程"

    print_info "请手动执行以下操作:"
    echo "  1. 在实例 1 的设备列表中，找到实例 2"
    echo "  2. 点击 '配对' 按钮"
    echo "  3. 在实例 2 的配对请求对话框中，点击 '接受'"
    echo "  4. 确认两个设备状态都变为 '已配对'"
    echo ""
    read -p "配对完成后按 Enter 键继续..." </dev/tty

    check_pairing_status "实例1" "$LOG1"
    check_pairing_status "实例2" "$LOG2"

    # 步骤 7: 测试文本同步
    print_header "步骤 7: 测试文本同步"

    print_info "请手动执行以下操作:"
    echo "  1. 在实例 1 的系统剪贴板中复制一段文本:"
    echo "     echo '测试文本同步 - $(date)' | pbcopy"
    echo "  2. 等待 5 秒"
    echo "  3. 在实例 2 中查看剪贴板历史"
    echo "  4. 确认能看到实例 1 复制的文本"
    echo ""
    read -p "文本同步测试完成后按 Enter 键继续..." </dev/tty

    check_clipboard_sync "实例1" "$LOG1"
    check_clipboard_sync "实例2" "$LOG2"

    # 步骤 8: 测试双向文本同步
    print_header "步骤 8: 测试双向文本同步"

    print_info "请手动执行以下操作:"
    echo "  1. 在实例 2 中复制文本:"
    echo "     echo '从实例B同步 - $(date)' | pbcopy"
    echo "  2. 等待 5 秒"
    echo "  3. 在实例 1 中查看剪贴板历史"
    echo "  4. 在实例 1 中复制另一段文本:"
    echo "     echo '从实例A同步 - $(date)' | pbcopy"
    echo "  5. 等待 5 秒"
    echo "  6. 在实例 2 中查看剪贴板历史"
    echo ""
    read -p "双向同步测试完成后按 Enter 键继续..." </dev/tty

    # 步骤 9: 测试文件传输
    print_header "步骤 9: 测试文件传输"

    print_info "测试文件已创建在: $LOG_DIR/test-files/"
    ls -lh "$LOG_DIR/test-files/"

    print_info "请手动执行以下操作:"
    echo "  1. 在实例 1 中点击 '文件' 按钮"
    echo "  2. 点击 '选择文件' 按钮"
    echo "  3. 选择测试文件: $LOG_DIR/test-files/test-small.txt"
    echo "  4. 选择目标设备: 实例 2"
    echo "  5. 点击 '发送' 按钮"
    echo "  6. 观察传输进度"
    echo "  7. 传输完成后，在实例 2 中查看文件传输历史"
    echo "  8. 检查下载目录: open ~/Downloads"
    echo ""
    read -p "文件传输测试完成后按 Enter 键继续..." </dev/tty

    check_file_transfer "实例1" "$LOG1"
    check_file_transfer "实例2" "$LOG2"

    # 步骤 10: 测试大文件传输（可选）
    print_header "步骤 10: 测试大文件传输（可选）"

    print_info "是否要测试大文件传输 (50MB)? (y/n)"
    read -r choice </dev/tty

    if [ "$choice" = "y" ]; then
        print_info "请手动执行以下操作:"
        echo "  1. 在实例 1 中发送大文件: $LOG_DIR/test-files/test-large.bin"
        echo "  2. 观察传输进度和内存占用"
        echo ""
        read -p "大文件传输测试完成后按 Enter 键继续..." </dev/tty
    fi

    # 步骤 11: 测试离线重连
    print_header "步骤 11: 测试离线重连"

    print_info "请手动执行以下操作:"
    echo "  1. 关闭实例 2 的窗口（或按 Cmd+Q 退出）"
    echo "  2. 在实例 1 中观察设备状态（可能显示离线）"
    echo "  3. 重新启动实例 2:"
        echo "   cd $INSTANCE2_DIR && npm run tauri dev > $LOG_DIR/instance2-restart.log 2>&1 &"
    echo "  4. 等待 10 秒"
    echo "  5. 在实例 1 中观察设备状态（应该重新上线）"
    echo "  6. 确认配对状态保持（不需要重新配对）"
    echo ""
    read -p "离线重连测试完成后按 Enter 键继续..." </dev/tty

    # 打印测试结果摘要
    print_header "测试结果摘要"

    echo "总测试数: $TESTS_TOTAL"
    echo -e "${GREEN}通过: $TESTS_PASSED${NC}"
    echo -e "${RED}失败: $TESTS_FAILED${NC}"

    if [ $TESTS_FAILED -eq 0 ]; then
        print_success "所有测试通过！"
    else
        print_error "有 $TESTS_FAILED 个测试失败，请查看日志"
    fi

    echo ""
    echo "详细测试报告: $LOG_DIR/test-results.log"
    echo "实例 1 日志: $LOG1"
    echo "实例 2 日志: $LOG2"
    echo ""
    echo "查看日志命令:"
    echo "  tail -f $LOG1"
    echo "  tail -f $LOG2"
    echo ""

    # 等待用户按 Ctrl+C 退出
    print_info "测试完成，按 Ctrl+C 退出..."
    wait
}

# 主函数
main() {
    print_header "🍎 草果剪贴板 - 功能测试套件"

    echo "此脚本将测试以下功能:"
    echo "  1. 应用启动和 HTTP 服务器"
    echo "  2. mDNS 设备发现"
    echo "  3. 配对流程"
    echo "  4. 文本同步"
    echo "  5. 文件传输"
    echo "  6. 离线重连"
    echo ""
    echo "准备开始测试..."
    echo ""

    read -p "按 Enter 键开始测试..." </dev/tty

    run_tests
}

# 运行主函数
main
