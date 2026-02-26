#!/bin/bash

echo "📸 保存图片功能测试"
echo "===================="
echo ""

# 检查下载目录是否存在
DOWNLOAD_DIR="$HOME/Downloads"
if [ ! -d "$DOWNLOAD_DIR" ]; then
    echo "❌ 下载目录不存在: $DOWNLOAD_DIR"
    exit 1
fi

echo "✅ 下载目录存在: $DOWNLOAD_DIR"
echo ""

# 统计现有的截图文件数量
BEFORE_COUNT=$(ls -1 "$DOWNLOAD_DIR"/screenshot_*.jpg 2>/dev/null | wc -l | tr -d ' ')
echo "📊 当前目录中的截图数量: $BEFORE_COUNT"
echo ""

echo "📝 测试步骤："
echo "1. 使用 Cmd+Shift+4 截图"
echo "2. 打开草果剪贴板应用"
echo "3. 将鼠标悬停在图片上"
echo "4. 点击绿色的保存按钮 (✓)"
echo "5. 检查下载目录中的新文件"
echo ""

echo "⏳ 等待你完成测试..."
echo ""
echo "完成后，按 Enter 键检查结果..."
read

# 统计测试后的截图文件数量
AFTER_COUNT=$(ls -1 "$DOWNLOAD_DIR"/screenshot_*.jpg 2>/dev/null | wc -l | tr -d ' ')

echo ""
echo "📊 测试后目录中的截图数量: $AFTER_COUNT"

if [ "$AFTER_COUNT" -gt "$BEFORE_COUNT" ]; then
    NEW_COUNT=$((AFTER_COUNT - BEFORE_COUNT))
    echo "✅ 测试成功！新增了 $NEW_COUNT 个截图文件"

    # 显示最新的截图文件
    echo ""
    echo "📁 最新的截图文件："
    ls -lt "$DOWNLOAD_DIR"/screenshot_*.jpg 2>/dev/null | head -n $NEW_COUNT | awk '{print "   " $NF}'
else
    echo "⚠️  没有检测到新文件"
    echo "   请检查："
    echo "   1. 是否成功点击了保存按钮"
    echo "   2. 应用是否显示错误信息"
    echo "   3. 查看控制台日志获取详细错误"
fi

echo ""
echo "🔍 查看下载目录："
echo "   open $DOWNLOAD_DIR"
