#!/bin/bash

# 图片存储架构重构测试脚本

echo "=========================================="
echo "   图片存储架构重构测试"
echo "=========================================="
echo ""

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# 测试计数
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

run_test() {
    local test_name="$1"
    local test_command="$2"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    echo "测试 $TOTAL_TESTS: $test_name"
    
    if eval "$test_command"; then
        echo -e "${GREEN}✓ 通过${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}✗ 失败${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    echo ""
}

echo "1. 检查数据结构修改"
echo "----------------------------------------"

run_test "ClipboardItem 包含 file_path 字段" \
    "grep -q 'file_path: Option<String>' src-tauri/src/lib.rs"

run_test "数据库表包含 file_path 列" \
    "grep -q 'file_path TEXT' src-tauri/src/lib.rs"

run_test "insert_item 函数接受 file_path 参数" \
    "grep -q 'file_path: Option<&str>' src-tauri/src/lib.rs"

echo "2. 检查图片保存功能"
echo "----------------------------------------"

run_test "包含图片文件保存逻辑" \
    "grep -q 'std::fs::write.*jpeg_bytes' src-tauri/src/clipboard/mod.rs"

run_test "生成唯一文件名" \
    "grep -q 'rand::random' src-tauri/src/clipboard/mod.rs"

run_test "创建 images 目录" \
    "grep -q 'create_dir_all.*images' src-tauri/src/clipboard/mod.rs"

run_test "保存相对路径到数据库" \
    "grep -q 'insert_item.*image.*relative_path' src-tauri/src/clipboard/mod.rs"

echo "3. 检查图片删除功能"
echo "----------------------------------------"

run_test "删除时获取文件路径" \
    "grep -q 'SELECT file_path FROM clipboard_items' src-tauri/src/lib.rs"

run_test "delete_item 返回文件路径" \
    "grep -q 'delete_item.*Option<String>' src-tauri/src/lib.rs"

run_test "delete_item 命令删除文件" \
    "grep -q 'delete_image_file' src-tauri/src/lib.rs"

run_test "clear_all 删除图片目录" \
    "grep -q 'remove_dir_all.*images' src-tauri/src/lib.rs"

echo "4. 检查前端图片显示"
echo "----------------------------------------"

run_test "导入 convertFileSrc" \
    "grep -q \"import { convertFileSrc }\" src/App.vue"

run_test "定义 getImageSrc 函数" \
    "grep -q 'function getImageSrc' src/App.vue"

run_test "使用 convertFileSrc 转换路径" \
    "grep -q 'convertFileSrc(item.file_path)' src/App.vue"

run_test "图片元素使用动态 src" \
    "grep -q ':src=\"getImageSrc(item)\"' src/App.vue"

echo "5. 检查 Tauri 配置"
echo "----------------------------------------"

run_test "启用 asset 协议" \
    "grep -q '\"assetProtocol\"' src-tauri/tauri.conf.json"

run_test "配置 CSP 允许 asset:" \
    "grep -q 'img-src.*asset:' src-tauri/tauri.conf.json"

run_test "asset scope 配置为 **" \
    "grep -q '\"scope\":.*\\[\"\\*\\*\"\\]' src-tauri/tauri.conf.json"

echo "6. 检查依赖"
echo "----------------------------------------"

run_test "Cargo.toml 包含 rand 依赖" \
    "grep -q 'rand = ' src-tauri/Cargo.toml"

echo "7. 检查编译"
echo "----------------------------------------"

run_test "Rust 代码编译通过" \
    "cd src-tauri && cargo check > /dev/null 2>&1"

run_test "前端代码编译通过" \
    "npm run build > /dev/null 2>&1"

echo "8. 检查文档"
echo "----------------------------------------"

run_test "存在架构设计文档" \
    "[ -f IMAGE_STORAGE_REDESIGN.md ]"

run_test "文档包含问题描述" \
    "grep -q '## 问题背景' IMAGE_STORAGE_REDESIGN.md"

run_test "文档包含解决方案" \
    "grep -q '## 新的架构设计' IMAGE_STORAGE_REDESIGN.md"

# 显示测试结果
echo "=========================================="
echo "   测试结果"
echo "=========================================="
echo ""
echo -e "总测试数: $TOTAL_TESTS"
echo -e "${GREEN}通过: $PASSED_TESTS${NC}"
echo -e "${RED}失败: $FAILED_TESTS${NC}"
echo ""

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}✓ 所有测试通过！${NC}"
    echo ""
    echo "下一步操作："
    echo "1. 运行应用: npm run tauri dev"
    echo "2. 测试图片功能:"
    echo "   - 截图并查看图片显示"
    echo "   - 点击图片测试复制"
    echo "   - 点击保存按钮测试保存"
    echo "   - 删除图片记录"
    echo "   - 清空所有记录"
    echo ""
    echo "预期效果："
    echo "✓ 图片正常显示在列表中"
    echo "✓ 图片存储在应用数据目录的 images 文件夹"
    echo "✓ 数据库只存储文件路径，不存储 base64"
    echo "✓ 删除记录时图片文件被删除"
    echo "✓ 清空时图片目录被清空"
    echo ""
    exit 0
else
    echo -e "${RED}✗ 有 $FAILED_TESTS 个测试失败${NC}"
    echo ""
    echo "请检查失败的测试项并修复"
    exit 1
fi
