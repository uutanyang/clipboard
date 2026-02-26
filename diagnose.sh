#!/bin/bash

echo "🔍 草果剪贴板诊断工具"
echo "======================"
echo ""

# 1. 检查应用状态
echo "1️⃣  应用状态检查"
echo "-------------------"
if pgrep -f "tauri-app" > /dev/null; then
    echo "✅ 应用正在运行"
    echo "📊 进程信息:"
    ps aux | grep -i "tauri-app" | grep -v grep
else
    echo "❌ 应用未运行"
    echo ""
    read -p "是否启动应用？(y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        open "/Users/yangtanfang/project/2026/4cool/clipboard-caoguo/src-tauri/target/release/bundle/macos/草果剪贴板.app"
        sleep 3
    else
        exit 0
    fi
fi

echo ""
echo "2️⃣  窗口可见性检查"
echo "--------------------"
# 尝试列出所有窗口
echo "📋 所有应用窗口:"
osascript -e 'tell application "System Events" to get name of every window of every process' 2>/dev/null | head -20

echo ""
echo "3️⃣  前端资源检查"
echo "-------------------"
RES_DIR="/Users/yangtanfang/project/2026/4cool/clipboard-caoguo/src-tauri/target/release/bundle/macos/草果剪贴板.app/Contents/Resources"
echo "📁 Resources 目录:"
ls -lh "$RES_DIR" 2>/dev/null || echo "❌ 目录不存在"

echo ""
echo "4️⃣  快捷键测试"
echo "---------------"
echo "💡 请按 Cmd+Shift+V 尝试呼出窗口"
echo "如果窗口出现，说明快捷键功能正常"
echo ""

echo "5️⃣  权限检查"
echo "---------------"
echo "⚠️  如果应用无法访问剪贴板，请检查:"
echo "   系统设置 > 隐私与安全性 > 辅助功能"
echo "   确保 '草果剪贴板' 已勾选"
echo ""

echo "6️⃣  日志位置"
echo "---------------"
echo "📋 应用日志目录:"
echo "   ~/Library/Logs/com.yangtanfang.caoguo-clipboard/"
echo ""

echo "7️⃣  解决方案"
echo "---------------"
echo "如果窗口不可见，可能的原因:"
echo ""
echo "1. 窗口是无边框的（decorations: false）"
echo "   - 这可能导致窗口难以发现"
echo "   - 尝试按 Cmd+Tab 切换应用"
echo ""
echo "2. 窗口可能在屏幕外"
echo "   - 尝试在系统设置中重置窗口位置"
echo ""
echo "3. 前端资源缺失"
echo "   - 需要重新打包应用"
echo ""

read -p "是否要重新打包应用？(y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "🔄 正在重新打包..."
    cd /Users/yangtanfang/project/2026/4cool/clipboard-caoguo
    npm run tauri build
    echo "✅ 打包完成"
    echo "🚀 请重新运行测试"
fi

echo ""
echo "📞 需要帮助？"
echo "   - 查看日志: ~/Library/Logs/com.yangtanfang.caoguo-clipboard/"
echo "   - 检查进程: ps aux | grep tauri-app"
echo "   - 查看文档: cat README.md"
