#!/bin/bash
# 修复 macOS 应用的中文显示名

APP_PATH="$1"
INFO_PLIST="$APP_PATH/Contents/Info.plist"

if [ -f "$INFO_PLIST" ]; then
    # 设置中文显示名
    /usr/libexec/PlistBuddy -c "Set :CFBundleDisplayName 贴立方" "$INFO_PLIST" 2>/dev/null || \
    /usr/libexec/PlistBuddy -c "Add :CFBundleDisplayName string 贴立方" "$INFO_PLIST"
    
    /usr/libexec/PlistBuddy -c "Set :CFBundleName 贴立方" "$INFO_PLIST" 2>/dev/null || \
    /usr/libexec/PlistBuddy -c "Add :CFBundleName string 贴立方" "$INFO_PLIST"
    
    echo "✅ 已设置应用显示名为: 贴立方"
else
    echo "❌ 未找到 Info.plist: $INFO_PLIST"
fi
