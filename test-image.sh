#!/bin/bash

# 图片显示功能测试脚本
# 用于验证图片在窗体列表中的显示功能

echo "=========================================="
echo "   图片显示功能测试"
echo "=========================================="
echo ""

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 测试计数
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# 测试函数
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

# 开始测试
echo "1. 检查源代码修改"
echo "----------------------------------------"

run_test "App.vue 包含图片状态管理" \
    "grep -q 'imageLoadingStates' src/App.vue"

run_test "App.vue 包含格式列表定义" \
    "grep -q 'imageFormats' src/App.vue"

run_test "App.vue 包含图片状态获取函数" \
    "grep -q 'getImageState' src/App.vue"

run_test "App.vue 包含图片 src 获取函数" \
    "grep -q 'getImageSrc' src/App.vue"

run_test "App.vue 包含智能错误处理" \
    "grep -q 'handleImageError(e, item)' src/App.vue"

run_test "App.vue 包含加载成功处理" \
    "grep -q 'handleImageLoad(item)' src/App.vue"

echo "2. 检查 UI 组件"
echo "----------------------------------------"

run_test "包含加载状态显示" \
    "grep -q 'image-loading' src/App.vue"

run_test "包含错误状态显示" \
    "grep -q 'image-error' src/App.vue"

run_test "包含加载动画" \
    "grep -q 'loading-spinner' src/App.vue"

run_test "包含动态图片 src 绑定" \
    "grep -q ':src=\"getImageSrc(item)\"' src/App.vue"

run_test "包含错误事件处理" \
    "grep -q '@error=\"(e) => handleImageError(e, item)\"' src/App.vue"

run_test "包含加载成功事件" \
    "grep -q '@load=\"handleImageLoad(item)\"' src/App.vue"

echo "3. 检查样式定义"
echo "----------------------------------------"

run_test "包含加载状态样式" \
    "grep -q '\.image-loading {' src/App.vue"

run_test "包含错误状态样式" \
    "grep -q '\.image-error {' src/App.vue"

run_test "包含旋转动画" \
    "grep -q '@keyframes spin' src/App.vue"

run_test "图片容器有最小高度" \
    "grep -q 'min-height: 80px' src/App.vue"

echo "4. 检查编译"
echo "----------------------------------------"

run_test "TypeScript 编译通过" \
    "npm run build > /dev/null 2>&1"

echo "5. 检查文档"
echo "----------------------------------------"

run_test "存在修复说明文档" \
    "[ -f IMAGE_FIX_SUMMARY.md ]"

run_test "文档包含问题描述" \
    "grep -q '## 问题描述' IMAGE_FIX_SUMMARY.md"

run_test "文档包含解决方案" \
    "grep -q '## 解决方案' IMAGE_FIX_SUMMARY.md"

run_test "文档包含测试步骤" \
    "grep -q '## 测试步骤' IMAGE_FIX_SUMMARY.md"

echo "6. 检查后端图片处理"
echo "----------------------------------------"

run_test "后端有图片处理逻辑" \
    "grep -q 'JpegEncoder' src-tauri/src/clipboard/mod.rs"

run_test "后端有 base64 编码" \
    "grep -q 'STANDARD.encode' src-tauri/src/clipboard/mod.rs"

run_test "后端有图片保存功能" \
    "grep -q 'save_image_to_file' src-tauri/src/lib.rs"

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
    echo "2. 测试图片复制和显示"
    echo "3. 检查加载状态和错误处理"
    echo ""
    echo "测试清单："
    echo "☐ 截图并粘贴到剪贴板"
    echo "☐ 图片在列表中正确显示"
    echo "☐ 大图片显示加载动画"
    echo "☐ 错误图片显示错误占位符"
    echo "☐ 点击图片可复制到剪贴板"
    echo "☐ 点击保存按钮可保存图片"
    echo ""
    exit 0
else
    echo -e "${RED}✗ 有 $FAILED_TESTS 个测试失败${NC}"
    echo ""
    echo "请检查失败的测试项并修复"
    exit 1
fi
