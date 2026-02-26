#!/bin/bash

# 生成 App Store 所需的大尺寸图标

set -e

INPUT_ICON="src-tauri/icons/icon.png"
OUTPUT_DIR="src-tauri/icons/appstore"

echo "🎨 生成 App Store 图标..."
echo "=================================="

# 创建输出目录
mkdir -p "$OUTPUT_DIR"

# 检查输入图标
if [ ! -f "$INPUT_ICON" ]; then
  echo "❌ 错误: 未找到源图标 $INPUT_ICON"
  echo "请将 1024x1024 的图标放置在 src-tauri/icons/icon.png"
  exit 1
fi

# 使用 sips 调整图标大小（macOS 内置工具）

echo "📐 生成各种尺寸图标..."

# App Store 图标 (1024x1024)
echo "  - 1024x1024 (App Store)"
sips -z 1024 1024 "$INPUT_ICON" --out "$OUTPUT_DIR/AppIcon-1024x1024.png"

# macOS 图标尺寸
echo "  - 16x16"
sips -z 16 16 "$INPUT_ICON" --out "$OUTPUT_DIR/AppIcon-16x16.png"

echo "  - 32x32"
sips -z 32 32 "$INPUT_ICON" --out "$OUTPUT_DIR/AppIcon-32x32.png"

echo "  - 64x64"
sips -z 64 64 "$INPUT_ICON" --out "$OUTPUT_DIR/AppIcon-64x64.png"

echo "  - 128x128"
sips -z 128 128 "$INPUT_ICON" --out "$OUTPUT_DIR/AppIcon-128x128.png"

echo "  - 256x256"
sips -z 256 256 "$INPUT_ICON" --out "$OUTPUT_DIR/AppIcon-256x256.png"

echo "  - 512x512"
sips -z 512 512 "$INPUT_ICON" --out "$OUTPUT_DIR/AppIcon-512x512.png"

# 生成 @2x Retina 图标
echo "  - 32x32@2x (64x64)"
sips -z 64 64 "$INPUT_ICON" --out "$OUTPUT_DIR/AppIcon-32x32@2x.png"

echo "  - 64x64@2x (128x128)"
sips -z 128 128 "$INPUT_ICON" --out "$OUTPUT_DIR/AppIcon-64x64@2x.png"

echo "  - 128x128@2x (256x256)"
sips -z 256 256 "$INPUT_ICON" --out "$OUTPUT_DIR/AppIcon-128x128@2x.png"

echo "  - 256x256@2x (512x512)"
sips -z 512 512 "$INPUT_ICON" --out "$OUTPUT_DIR/AppIcon-256x256@2x.png"

# 生成 .icns 文件（使用 iconutil）
echo "  📦 生成 .icns 文件..."
mkdir -p "$OUTPUT_DIR/icon.iconset"

# 复制图标到 iconset
cp "$OUTPUT_DIR/AppIcon-16x16.png" "$OUTPUT_DIR/icon.iconset/icon_16x16.png"
cp "$OUTPUT_DIR/AppIcon-32x32.png" "$OUTPUT_DIR/icon.iconset/icon_16x16@2x.png"
cp "$OUTPUT_DIR/AppIcon-32x32.png" "$OUTPUT_DIR/icon.iconset/icon_32x32.png"
cp "$OUTPUT_DIR/AppIcon-64x64.png" "$OUTPUT_DIR/icon.iconset/icon_32x32@2x.png"
cp "$OUTPUT_DIR/AppIcon-128x128.png" "$OUTPUT_DIR/icon.iconset/icon_128x128.png"
cp "$OUTPUT_DIR/AppIcon-256x256.png" "$OUTPUT_DIR/icon.iconset/icon_128x128@2x.png"
cp "$OUTPUT_DIR/AppIcon-256x256.png" "$OUTPUT_DIR/icon.iconset/icon_256x256.png"
cp "$OUTPUT_DIR/AppIcon-512x512.png" "$OUTPUT_DIR/icon.iconset/icon_256x256@2x.png"
cp "$OUTPUT_DIR/AppIcon-512x512.png" "$OUTPUT_DIR/icon.iconset/icon_512x512.png"
cp "$OUTPUT_DIR/AppIcon-1024x1024.png" "$OUTPUT_DIR/icon.iconset/icon_512x512@2x.png"

# 生成 icns
iconutil -c icns "$OUTPUT_DIR/icon.iconset" -o "$OUTPUT_DIR/icon.icns"

echo ""
echo "✅ 图标生成完成！"
echo ""
echo "生成的图标位于: $OUTPUT_DIR"
echo ""
echo "主要内容:"
ls -lh "$OUTPUT_DIR"
echo ""
echo "使用说明:"
echo "  - App Store: 上传 AppIcon-1024x1024.png"
echo "  - 替换当前图标: cp $OUTPUT_DIR/icon.icns src-tauri/icons/"
