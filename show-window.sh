#!/bin/bash

# 显示草果剪贴板窗口
echo "🍎 尝试显示草果剪贴板窗口..."
echo ""

# 方法1: 使用 osascript
osascript -e 'tell application "System Events"
    tell process "草果剪贴板"
        set frontmost to true
    end tell
end tell' 2>/dev/null && echo "✅ 使用 osascript 成功" || echo "⚠️  osascript 方法失败"

# 方法2: 尝试使用快捷键模拟
echo ""
echo "💡 提示: 按 Cmd+Shift+V 呼出窗口"

# 检查应用状态
echo ""
if pgrep -f "tauri-app" > /dev/null; then
    echo "✅ 应用正在运行"
    echo "🔍 进程 ID: $(pgrep -f tauri-app | head -1)"
else
    echo "❌ 应用未运行"
    echo "🚀 正在启动..."
    open "/Users/yangtanfang/project/2026/4cool/clipboard-caoguo/src-tauri/target/release/bundle/macos/草果剪贴板.app"
fi
