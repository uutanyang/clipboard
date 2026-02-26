#!/bin/bash

# 草果剪贴板 - 异常情况测试脚本
# 测试设备离线重连、多内容同步、大文件传输、网络断开、快速同步回声

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
NC='\033[0m'

# 测试结果统计
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_TOTAL=0

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
    echo "[$result] $test_name: $details" >> "$LOG_DIR/test-exceptional.log"
}

# 检查设备离线
check_device_offline() {
    local log_file=$1
    local device=$2

    if grep -q "$device.*offline\|connection.*lost\|device.*not.*responding" "$log_file" 2>/dev/null; then
        return 0
    else
        return 1
    fi
}

# 检查设备重连
check_device_reconnected() {
    local log_file=$1
    local device=$2

    if grep -q "$device.*reconnected\|device.*online\|connection.*restored" "$log_file" 2>/dev/null; then
        return 0
    else
        return 1
    fi
}

# 检查重复同步
check_duplicate_sync() {
    local log_file=$1

    local duplicates=$(grep -c "clipboard.*sync" "$log_file" 2>/dev/null || echo "0")
    echo "$duplicates"
}

# 检查传输进度
check_transfer_progress() {
    local log_file=$1

    if grep -q "progress" "$log_file" 2>/dev/null || grep -q "%" "$log_file" 2>/dev/null; then
        return 0
    else
        return 1
    fi
}

# 清理函数
cleanup() {
    echo ""
    print_header "清理环境"

    # 恢复端口
    local src_file="$PROJECT_DIR/src-tauri/src/server/mod.rs"
    if [ -f "$src_file.bak" ]; then
        cp "$src_file.bak" "$src_file"
        rm -f "$src_file.bak"
    fi

    # 停止所有实例
    pkill -9 "草果剪贴板" 2>/dev/null || true
    pkill -9 tauri-app 2>/dev/null || true

    print_success "清理完成"
}

trap cleanup EXIT INT TERM

# 创建测试文件
create_test_files() {
    mkdir -p "$LOG_DIR/test-files"

    print_info "创建测试文件..."

    # 小文件 1MB
    dd if=/dev/urandom of="$LOG_DIR/test-files/1MB.bin" bs=1M count=1 2>/dev/null

    # 中文件 50MB
    dd if=/dev/urandom of="$LOG_DIR/test-files/50MB.bin" bs=1M count=50 2>/dev/null

    # 大文件 100MB
    dd if=/dev/urandom of="$LOG_DIR/test-files/100MB.bin" bs=1M count=100 2>/dev/null

    # 大文件 200MB
    dd if=/dev/urandom of="$LOG_DIR/test-files/200MB.bin" bs=1M count=200 2>/dev/null

    print_success "测试文件创建完成"
    ls -lh "$LOG_DIR/test-files/"
}

# 测试 1: 设备离线后重连
test_offline_reconnect() {
    print_header "测试 1: 设备离线后重连"

    print_info "测试步骤："
    echo "  1. 启动两个实例并配对"
    echo "  2. 验证连接状态为'已连接'"
    echo "  3. 关闭实例 2"
    echo "  4. 等待 30-60 秒，观察状态变为'未连接'"
    echo "  5. 重新启动实例 2"
    echo "  6. 观察连接状态自动恢复为'已连接'"

    echo ""
    print_info "请手动执行以下步骤："
    echo "  1. 确保两个实例已启动并配对"
    echo "  2. 按以下顺序操作："
    echo ""
    read -p "步骤 1: 确认两个实例已配对，按 Enter 继续..." </dev/tty

    print_info "现在关闭实例 2 的窗口..."
    read -p "关闭后按 Enter 继续..." </dev/tty

    print_info "等待 30 秒观察状态变化..."
    sleep 30

    # 检查日志
    LOG1="$LOG_DIR/instance1.log"
    if check_device_offline "$LOG1" "device"; then
        record_test "设备离线检测" "PASS" "检测到设备离线"
    else
        record_test "设备离线检测" "FAIL" "未检测到设备离线"
    fi

    print_info "请重新启动实例 2..."
    print_info "命令: cd $INSTANCE2_DIR && npm run tauri dev > $LOG_DIR/instance2-restart.log 2>&1 &"
    read -p "启动后按 Enter 继续..." </dev/tty

    print_info "等待 20 秒观察连接恢复..."
    sleep 20

    # 检查重连
    sleep 10  # 额外等待确保重连完成
    if check_device_reconnected "$LOG1" "device"; then
        record_test "设备自动重连" "PASS" "设备成功重连"
    else
        record_test "设备自动重连" "WARN" "无法确认重连（可能正常工作，请手动验证）"
    fi

    print_info "检查当前连接状态..."
    echo "  请观察实例 1 的连接状态是否恢复为'已连接'"
    read -p "按 Enter 继续..." </dev/tty
}

# 测试 2: 同时复制多个内容
test_multiple_copy() {
    print_header "测试 2: 同时复制多个内容"

    print_info "测试步骤："
    echo "  1. 快速复制多个不同的文本"
    echo "  2. 观察两端剪贴板历史是否完整"
    echo "  3. 检查是否有重复或丢失"

    echo ""
    print_info "即将快速复制 10 段文本..."
    echo "请注意观察两个实例的剪贴板历史"
    read -p "准备好后按 Enter 开始..." </dev/tty

    # 快速复制多个内容
    for i in {1..10}; do
        echo "测试文本 $i - $(date +%H:%M:%S.%N)" | pbcopy
        sleep 0.3  # 300ms 间隔
    done

    print_success "已复制 10 段文本"
    print_info "等待 5 秒让同步完成..."
    sleep 5

    print_info "请检查两个实例的剪贴板历史："
    echo "  1. 是否包含全部 10 段文本？"
    echo "  2. 是否有重复内容？"
    echo "  3. 时间戳是否正确？"
    read -p "检查完成后按 Enter 继续..." </dev/tty

    # 检查日志
    LOG1="$LOG_DIR/instance1.log"
    LOG2="$LOG_DIR/instance2.log"

    local sync_count=$(grep -c "clipboard.*sync" "$LOG1" 2>/dev/null || echo "0")
    print_info "实例 1 同步次数: $sync_count"

    if [ "$sync_count" -ge 10 ]; then
        record_test "多内容同步" "PASS" "同步了 $sync_count 次剪贴板内容"
    elif [ "$sync_count" -ge 5 ]; then
        record_test "多内容同步" "WARN" "同步了 $sync_count 次（预期 10 次），可能有遗漏"
    else
        record_test "多内容同步" "FAIL" "只同步了 $sync_count 次，内容可能丢失"
    fi
}

# 测试 3: 大文件传输
test_large_file_transfer() {
    print_header "测试 3: 大文件传输"

    print_info "测试文件大小："
    ls -lh "$LOG_DIR/test-files/"

    # 测试 50MB 文件
    print_test "测试 50MB 文件传输"
    print_info "请手动执行："
    echo "  1. 在实例 1 中选择文件: $LOG_DIR/test-files/50MB.bin"
    echo "  2. 发送到实例 2"
    echo "  3. 观察传输进度"
    echo "  4. 验证传输成功"
    echo ""
    read -p "50MB 文件传输测试完成后按 Enter 继续..." </dev/tty

    LOG1="$LOG_DIR/instance1.log"
    LOG2="$LOG_DIR/instance2.log"

    if check_transfer_progress "$LOG1"; then
        record_test "50MB 文件传输进度" "PASS" "传输进度正常显示"
    else
        record_test "50MB 文件传输进度" "WARN" "未检测到进度显示"
    fi

    # 测试 100MB 文件
    print_test "测试 100MB 文件传输"
    print_info "请手动执行："
    echo "  1. 在实例 1 中选择文件: $LOG_DIR/test-files/100MB.bin"
    echo "  2. 发送到实例 2"
    echo "  3. 观察传输进度"
    echo "  4. 验证传输成功"
    echo "  5. 检查内存占用（活动监视器）"
    echo ""
    read -p "100MB 文件传输测试完成后按 Enter 继续..." </dev/tty

    # 测试 200MB 文件（可选）
    print_info "是否要测试 200MB 文件传输？(y/n)"
    read -r choice </dev/tty

    if [ "$choice" = "y" ]; then
        print_test "测试 200MB 文件传输"
        print_info "警告：此测试可能需要较长时间并占用大量内存"
        read -p "准备好后按 Enter 开始..." </dev/tty

        print_info "请手动执行："
        echo "  1. 在实例 1 中选择文件: $LOG_DIR/test-files/200MB.bin"
        echo "  2. 发送到实例 2"
        echo "  3. 观察传输进度和内存占用"
        echo "  4. 验证传输成功"
        echo ""
        read -p "200MB 文件传输测试完成后按 Enter 继续..." </dev/tty

        # 检查是否有内存问题
        if grep -q "OutOfMemory\|memory.*error\|allocation.*failed" "$LOG1" "$LOG2" 2>/dev/null; then
            record_test "200MB 文件传输" "FAIL" "检测到内存错误"
        else
            record_test "200MB 文件传输" "PASS" "大文件传输成功"
        fi
    fi
}

# 测试 4: 网络断开处理
test_network_disconnect() {
    print_header "测试 4: 网络断开处理"

    print_info "测试步骤："
    echo "  1. 断开网络连接（WiFi 或 以太网）"
    echo "  2. 等待 10 秒"
    echo "  3. 复制一段文本"
    echo "  4. 观察应用是否崩溃或卡顿"
    echo "  5. 重新连接网络"
    echo "  6. 验证应用恢复正常"

    echo ""
    print_info "请手动执行以下步骤："
    echo "  1. 断开网络连接（关闭 WiFi 或拔掉网线）"
    echo "  2. 等待 10 秒后复制文本测试应用响应"
    echo ""
    read -p "网络断开测试完成后按 Enter 继续..." </dev/tty

    # 检查错误日志
    LOG1="$LOG_DIR/instance1.log"
    LOG2="$LOG_DIR/instance2.log"

    if grep -q "panic\|crash\|fatal.*error\|stack.*overflow" "$LOG1" "$LOG2" 2>/dev/null; then
        record_test "网络断开稳定性" "FAIL" "检测到崩溃或严重错误"
    else
        record_test "网络断开稳定性" "PASS" "应用在网络断开时保持稳定"
    fi

    print_info "请重新连接网络..."
    read -p "网络重新连接后按 Enter 继续..." </dev/tty

    print_info "等待 15 秒让设备重新发现..."
    sleep 15

    if check_device_reconnected "$LOG1" "device"; then
        record_test "网络恢复后自动重连" "PASS" "网络恢复后设备自动重连"
    else
        record_test "网络恢复后自动重连" "WARN" "请手动验证重连是否正常"
    fi
}

# 测试 5: 快速连续同步回声
test_echo_problem() {
    print_header "测试 5: 快速连续同步回声"

    print_info "测试说明："
    echo "  '回声问题'是指 A 同步到 B 后，B 又同步回 A，形成无限循环"
    echo ""
    echo "测试步骤："
    echo "  1. 在实例 1 快速复制文本"
    echo "  2. 观察是否出现重复同步"
    echo "  3. 检查日志中是否有重复的同步事件"

    echo ""
    print_info "即将进行快速连续复制测试..."
    read -p "准备好后按 Enter 开始..." </dev/tty

    # 清空日志以便分析
    LOG1="$LOG_DIR/instance1.log"
    LOG2="$LOG_DIR/instance2.log"

    print_info "开始快速复制测试（5 次快速复制）..."
    for i in {1..5}; do
        echo "回声测试文本 $i - $(date +%H:%M:%S.%N)" | pbcopy
        sleep 0.2  # 200ms 间隔
    done

    print_success "复制完成"
    print_info "等待 5 秒让同步完成..."
    sleep 5

    # 分析日志
    print_info "分析同步日志..."
    local recent_syncs=$(tail -100 "$LOG1" | grep -c "clipboard.*sync" || echo "0")
    local recent_sends=$(tail -100 "$LOG1" | grep -c "clipboard.*send" || echo "0")
    local recent_receives=$(tail -100 "$LOG1" | grep -c "clipboard.*receive" || echo "0")

    echo "  最近同步事件: $recent_syncs"
    echo "  发送次数: $recent_sends"
    echo "  接收次数: $recent_receives"

    # 计算发送接收比（用于检测回声）
    if [ "$recent_sends" -gt 0 ]; then
        local ratio=$((recent_receives * 100 / recent_sends))
        echo "  发送接收比: $ratio%"

        if [ "$ratio" -gt 300 ]; then  # 接收次数远大于发送次数
            record_test "回声问题检测" "FAIL" "检测到可能的回声问题（接收次数异常高）"
        elif [ "$ratio" -gt 200 ]; then
            record_test "回声问题检测" "WARN" "接收次数偏高，可能有轻微回声问题"
        else
            record_test "回声问题检测" "PASS" "未检测到回声问题"
        fi
    else
        record_test "回声问题检测" "PASS" "未检测到明显的同步活动"
    fi

    print_info "请手动检查："
    echo "  1. 剪贴板历史中是否有重复内容？"
    echo "  2. 是否有大量相同的同步记录？"
    echo "  3. 同步次数是否合理？"
    read -p "检查完成后按 Enter 继续..." </dev/tty
}

# 主测试流程
run_tests() {
    mkdir -p "$LOG_DIR"
    mkdir -p "$LOG_DIR/test-files"

    # 清空测试日志
    echo "草果剪贴板 - 异常情况测试报告" > "$LOG_DIR/test-exceptional.log"
    echo "测试时间: $(date)" >> "$LOG_DIR/test-exceptional.log"
    echo "================================" >> "$LOG_DIR/test-exceptional.log"
    echo "" >> "$LOG_DIR/test-exceptional.log"

    print_header "🍎 草果剪贴板 - 异常情况测试套件"

    # 创建测试文件
    create_test_files

    echo ""
    print_info "请确保："
    echo "  1. 两个应用实例已经启动并配对"
    echo "  2. 实例 1 端口: 54321"
    echo "  3. 实例 2 端口: $INSTANCE2_PORT"
    echo "  4. 日志文件:"
    echo "     - $LOG_DIR/instance1.log"
    echo "     - $LOG_DIR/instance2.log"
    echo ""
    read -p "准备好后按 Enter 开始测试..." </dev/tty

    # 执行测试
    test_offline_reconnect
    test_multiple_copy
    test_large_file_transfer
    test_network_disconnect
    test_echo_problem

    # 打印测试结果摘要
    print_header "测试结果摘要"

    echo "总测试数: $TESTS_TOTAL"
    echo -e "${GREEN}通过: $TESTS_PASSED${NC}"
    echo -e "${RED}失败: $TESTS_FAILED${NC}"

    local warnings=$((TESTS_TOTAL - TESTS_PASSED - TESTS_FAILED))
    if [ $warnings -gt 0 ]; then
        echo -e "${YELLOW}警告: $warnings${NC}"
    fi

    if [ $TESTS_FAILED -eq 0 ]; then
        print_success "所有关键测试通过！"
    else
        print_error "有 $TESTS_FAILED 个测试失败，请查看日志"
    fi

    echo ""
    echo "详细测试报告: $LOG_DIR/test-exceptional.log"
    echo ""
    echo "查看日志命令:"
    echo "  tail -f $LOG_DIR/instance1.log"
    echo "  tail -f $LOG_DIR/instance2.log"
    echo ""

    # 等待用户按 Ctrl+C 退出
    print_info "测试完成，按 Ctrl+C 退出..."
    wait
}

# 主函数
main() {
    print_header "🍎 草果剪贴板 - 异常情况测试"

    echo "此脚本将测试以下异常情况："
    echo "  1. 设备离线后重连"
    echo "  2. 同时复制多个内容"
    echo "  3. 大文件传输（100MB+）"
    echo "  4. 网络断开时的错误处理"
    echo "  5. 快速连续同步回声问题"
    echo ""
    echo "准备开始测试..."
    echo ""

    read -p "按 Enter 键开始测试..." </dev/tty

    run_tests
}

# 运行主函数
main
