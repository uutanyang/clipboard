#!/bin/bash

# 版本号统一管理脚本

VERSION=$1

if [ -z "$VERSION" ]; then
  echo "❌ 错误: 请提供版本号"
  echo "用法: ./scripts/update-version.sh <version>"
  echo "示例: ./scripts/update-version.sh 1.0.0"
  exit 1
fi

# 验证版本号格式
if ! [[ $VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "❌ 错误: 版本号格式不正确，应为 X.Y.Z 格式 (如 1.0.0)"
  exit 1
fi

echo "🔄 更新版本号到 $VERSION..."

# 1. 更新 package.json
echo "  📄 更新 package.json"
if [ -f "package.json" ]; then
  sed -i '' "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" package.json
  echo "  ✓ package.json 已更新"
else
  echo "  ⚠️  package.json 未找到"
fi

# 2. 更新 Cargo.toml
echo "  📄 更新 src-tauri/Cargo.toml"
if [ -f "src-tauri/Cargo.toml" ]; then
  sed -i '' "s/^version = \".*\"/version = \"$VERSION\"/" src-tauri/Cargo.toml
  echo "  ✓ Cargo.toml 已更新"
else
  echo "  ⚠️  Cargo.toml 未找到"
fi

# 3. 更新 tauri.conf.json
echo "  📄 更新 src-tauri/tauri.conf.json"
if [ -f "src-tauri/tauri.conf.json" ]; then
  sed -i '' "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" src-tauri/tauri.conf.json
  echo "  ✓ tauri.conf.json 已更新"
else
  echo "  ⚠️  tauri.conf.json 未找到"
fi

# 4. 更新 README.md 中的版本号（如果存在）
echo "  📄 更新 README.md"
if [ -f "README.md" ]; then
  sed -i '' "s/版本：[0-9]\+\.[0-9]\+\.[0-9]\+/版本：$VERSION/g" README.md
  sed -i '' "s/Version: [0-9]\+\.[0-9]\+\.[0-9]\+/Version: $VERSION/g" README.md
  echo "  ✓ README.md 已更新"
fi

# 5. 创建 Git 标签（可选）
echo ""
echo "📌 创建 Git 标签 v$VERSION..."
git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json README.md
git commit -m "chore: bump version to $VERSION"
git tag -a "v$VERSION" -m "Release version $VERSION"

echo ""
echo "✅ 版本号更新完成！"
echo ""
echo "下一步:"
echo "  1. 运行 'npm run tauri build' 构建应用"
echo "  2. 推送标签: git push origin v$VERSION"
echo "  3. 创建 GitHub Release"
