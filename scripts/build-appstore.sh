#!/bin/bash

# App Store 完整构建脚本

set -e

# 配置
VERSION=${1:-"1.0.0"}
APP_NAME="贴立方"
BUNDLE_ID="com.yangtanfang.tie-lifang"
DMG_NAME="${APP_NAME}_${VERSION}_universal"
PKG_NAME="${APP_NAME}_${VERSION}_universal"

# 证书和公证配置（请根据实际情况修改）
SIGNING_IDENTITY="${APPLE_SIGNING_IDENTITY:-Developer ID Application: Your Name (TEAM_ID)}"
APPLE_ID="${APPLE_ID:-your@email.com}"
TEAM_ID="${TEAM_ID:-YOUR_TEAM_ID}"
APPLE_PASSWORD="${APPLE_PASSWORD:-app-specific-password}"

echo "🚀 开始构建 App Store 版本 $VERSION"
echo "=================================="

# 1. 清理之前的构建
echo "🧹 清理之前的构建..."
rm -rf src-tauri/target/release/bundle

# 2. 更新版本号
echo "📝 更新版本号..."
./scripts/update-version.sh "$VERSION"

# 3. 构建 Universal Binary
echo "🔨 构建 Universal Binary..."
npm run tauri build -- --target universal-apple-darwin

# 4. 查找构建输出
APP_PATH="src-tauri/target/release/bundle/macos/${APP_NAME}.app"
DMG_PATH="src-tauri/target/release/bundle/dmg/${DMG_NAME}.dmg"
PKG_PATH="src-tauri/target/release/bundle/pkg/${PKG_NAME}.pkg"

if [ ! -f "$APP_PATH" ]; then
  echo "❌ 错误: 未找到构建的应用: $APP_PATH"
  exit 1
fi

# 5. 签名应用（如果配置了签名身份）
if [ "$SIGNING_IDENTITY" != "Developer ID Application: Your Name (TEAM_ID)" ]; then
  echo "🔐 签名应用..."
  codesign --deep --force --verify --verbose \
    --sign "$SIGNING_IDENTITY" \
    "$APP_PATH"

  echo "✅ 应用签名完成"
else
  echo "⚠️  跳过签名（未配置 SIGNING_IDENTITY）"
fi

# 6. 公证 DMG（如果配置了公证凭证）
if [ "$APPLE_ID" != "your@email.com" ]; then
  if [ -f "$DMG_PATH" ]; then
    echo "📤 公证 DMG..."
    xcrun notarytool submit "$DMG_PATH" \
      --apple-id "$APPLE_ID" \
      --password "$APPLE_PASSWORD" \
      --team-id "$TEAM_ID" \
      --wait

    echo "📎 装订公证票据..."
    xcrun stapler staple "$DMG_PATH"

    echo "✅ 公证完成"
  else
    echo "⚠️  未找到 DMG 文件: $DMG_PATH"
  fi

  if [ -f "$PKG_PATH" ]; then
    echo "📤 公证 PKG..."
    xcrun notarytool submit "$PKG_PATH" \
      --apple-id "$APPLE_ID" \
      --password "$APPLE_PASSWORD" \
      --team-id "$TEAM_ID" \
      --wait

    echo "📎 装订公证票据..."
    xcrun stapler staple "$PKG_PATH"

    echo "✅ 公证完成"
  else
    echo "⚠️  未找到 PKG 文件: $PKG_PATH"
  fi
else
  echo "⚠️  跳过公证（未配置公证凭证）"
fi

# 7. 显示构建结果
echo ""
echo "=================================="
echo "✅ 构建完成！"
echo ""
echo "构建输出:"
if [ -f "$DMG_PATH" ]; then
  echo "  📦 DMG: $DMG_PATH ($(du -h "$DMG_PATH" | cut -f1))"
fi
if [ -f "$PKG_PATH" ]; then
  echo "  📦 PKG: $PKG_PATH ($(du -h "$PKG_PATH" | cut -f1))"
fi
echo "  📦 APP: $APP_PATH ($(du -sh "$APP_PATH" | cut -f1))"
echo ""
echo "下一步:"
echo "  1. 使用 Transporter 上传 PKG 到 App Store Connect"
echo "  2. 或者手动上传: open -a Transporter"
echo ""
echo "验证签名:"
echo "  codesign -dv --verbose=4 \"$APP_PATH\""
echo "  spctl -a -v -t execute \"$APP_PATH\""
echo ""
