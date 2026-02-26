#!/bin/bash

# 测试清除缓存按钮功能

echo "🧪 测试清除同步缓存按钮..."
echo "=================================="

# 1. 编译检查
echo "📦 1. 编译检查..."
npm run build > /dev/null 2>&1
if [ $? -eq 0 ]; then
  echo "  ✓ 前端编译通过"
else
  echo "  ✗ 前端编译失败"
  exit 1
fi

# 2. 检查修复内容
echo "📦 2. 检查修复内容..."

# 检查状态变量
if grep -q "const clearingCache = ref" src/App.vue; then
  echo "  ✓ 状态管理已添加"
else
  echo "  ✗ 状态管理缺失"
  exit 1
fi

# 检查按钮状态绑定
if grep -q "btn-loading.*clearingCache" src/App.vue; then
  echo "  ✓ 按钮状态绑定正确"
else
  echo "  ✗ 按钮状态绑定缺失"
  exit 1
fi

# 检查样式
if grep -q "\.btn-success" src/App.vue; then
  echo "  ✓ 成功样式已添加"
else
  echo "  ✗ 成功样式缺失"
  exit 1
fi

if grep -q "\.btn-error" src/App.vue; then
  echo "  ✓ 错误样式已添加"
else
  echo "  ✗ 错误样式缺失"
  exit 1
fi

# 3. 检查后端命令
echo "📦 3. 检查后端命令..."

if grep -q "clear_hash_cache" src-tauri/src/lib.rs; then
  echo "  ✓ 后端命令存在"
else
  echo "  ✗ 后端命令缺失"
  exit 1
fi

# 4. 显示测试说明
echo ""
echo "=================================="
echo "✅ 编译和代码检查通过！"
echo ""
echo "📋 手动测试步骤："
echo ""
echo "1. 启动应用："
echo "   npm run tauri dev"
echo ""
echo "2. 测试清除缓存按钮："
echo "   - 点击顶部工具栏的'设置'按钮（齿轮图标）"
echo "   - 找到'清除同步缓存'设置项"
echo "   - 点击右侧的'清除'按钮"
echo ""
echo "3. 预期效果："
echo "   ✓ 按钮文字立即变为'清除中...'"
echo "   ✓ 按钮变为半透明禁用状态"
echo "   ✓ 1秒后按钮变为绿色，显示'✓ 已清除'"
echo "   ✓ 2秒后恢复初始状态"
echo ""
echo "4. 测试错误处理（可选）："
echo "   - 修改后端代码模拟错误"
echo "   - 点击按钮应显示红色错误状态"
echo "   - 3秒后自动恢复"
echo ""
echo "📖 详细说明请查看："
echo "   CLEAR_CACHE_BUTTON_FIX.md"
echo ""
