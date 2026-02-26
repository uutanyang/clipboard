
#!/bin/bash

# Tauri 图标生成环境设置脚本
# 自动检测并安装必要依赖

echo "检查 Tauri CLI 是否已安装..."

# 检查 cargo 是否存在
if ! command -v cargo &> /dev/null
then
    echo "错误：未找到 Cargo。请先安装 Rust：https://www.rust-lang.org/tools/install"
    exit 1
fi

# 检查 tauri 命令是否存在
if ! cargo tauri --version &> /dev/null
then
    echo "Tauri CLI 未安装，正在安装..."
    cargo install tauri-cli
else
    echo "Tauri CLI 版本：$(cargo tauri --version)"
fi

# 设置源图标路径
SOURCE_ICON="./app-icon.png"
OUTPUT_DIR="./src-tauri/icons"

# 检查源图标是否存在
if [ ! -f "$SOURCE_ICON" ]; then
  echo "警告：未找到源图标文件 $SOURCE_ICON"
  echo "请提供一个至少 1024x1024 像素的 PNG 格式图标文件"
  exit 1
fi

# 创建输出目录
mkdir -p "$OUTPUT_DIR"

# 生成图标
echo "正在生成各平台图标..."
cargo tauri icon "$SOURCE_ICON" -o "$OUTPUT_DIR"

if [ $? -eq 0 ]; then
  echo "✅ 图标生成成功！"
  echo "📁 输出位置：$OUTPUT_DIR"
  echo ""
  echo "生成的文件包括："
  echo "• icon.icns - macOS 应用图标"
  echo "• icon.ico  - Windows 应用图标"  
  echo "• 各种尺寸的 PNG 文件 - Linux 应用图标"
else
  echo "❌ 图标生成失败，请检查错误信息"
  exit 1
fi
