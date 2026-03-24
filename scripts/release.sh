#!/bin/bash

# 贴立方自动发布脚本
# 用法: ./scripts/release.sh [version]
# 示例: ./scripts/release.sh 0.1.0

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 获取版本号
if [ -z "$1" ]; then
    # 从 package.json 获取当前版本
    CURRENT_VERSION=$(node -p "require('./package.json').version")
    echo -e "${YELLOW}当前版本: v${CURRENT_VERSION}${NC}"
    echo -e "${YELLOW}请输入新版本号 (例如: 0.2.0):${NC}"
    read NEW_VERSION
else
    NEW_VERSION=$1
fi

# 移除 v 前缀（如果有）
NEW_VERSION=${NEW_VERSION#v}
TAG="v${NEW_VERSION}"

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  贴立方自动发布流程${NC}"
echo -e "${GREEN}========================================${NC}"
echo -e "版本: ${YELLOW}${TAG}${NC}"
echo ""

# 确认发布
echo -e "${YELLOW}即将执行以下操作:${NC}"
echo "1. 更新 package.json 版本号"
echo "2. 更新 src-tauri/Cargo.toml 版本号"
echo "3. 提交更改"
echo "4. 创建 Git 标签"
echo "5. 推送到 GitHub"
echo "6. 触发 GitHub Actions 自动构建"
echo ""
echo -e "${YELLOW}是否继续? (y/n)${NC}"
read -r CONFIRM

if [ "$CONFIRM" != "y" ] && [ "$CONFIRM" != "Y" ]; then
    echo "已取消发布"
    exit 0
fi

# 1. 更新 package.json
echo ""
echo -e "${GREEN}[1/6] 更新 package.json...${NC}"
CURRENT_PKG_VERSION=$(node -p "require('./package.json').version")
if [ "$CURRENT_PKG_VERSION" = "$NEW_VERSION" ]; then
    echo -e "${YELLOW}package.json 版本已是 $NEW_VERSION，跳过更新${NC}"
else
    npm version "$NEW_VERSION" --no-git-tag-version
fi

# 2. 更新 Cargo.toml
echo -e "${GREEN}[2/6] 更新 Cargo.toml...${NC}"
CURRENT_CARGO_VERSION=$(grep '^version = ' src-tauri/Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
if [ "$CURRENT_CARGO_VERSION" = "$NEW_VERSION" ]; then
    echo -e "${YELLOW}Cargo.toml 版本已是 $NEW_VERSION，跳过更新${NC}"
else
    sed -i.bak "s/^version = \".*\"/version = \"$NEW_VERSION\"/" src-tauri/Cargo.toml
    rm -f src-tauri/Cargo.toml.bak
fi

# 3. 提交更改
echo -e "${GREEN}[3/6] 提交更改...${NC}"
if git diff --quiet && git diff --cached --quiet; then
    echo -e "${YELLOW}没有版本变更，跳过提交${NC}"
else
    git add package.json package-lock.json src-tauri/Cargo.toml src-tauri/Cargo.lock
    git commit -m "chore: bump version to $TAG"
fi

# 4. 创建标签
echo -e "${GREEN}[4/6] 创建 Git 标签...${NC}"
git tag -a "$TAG" -m "Release $TAG"

# 5. 推送代码和标签
echo -e "${GREEN}[5/6] 推送到 GitHub...${NC}"
git push origin master
git push origin "$TAG"

# 6. 完成
echo ""
echo -e "${GREEN}[6/6] 发布流程已完成!${NC}"
echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  发布成功!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "GitHub Actions 正在构建以下平台:"
echo "  - macOS (Intel & Apple Silicon)"
echo "  - Windows"
echo "  - Linux"
echo ""
echo "查看构建状态:"
echo "  https://github.com/uutanyang/clipboard/actions"
echo ""
echo "构建完成后，Release 将出现在:"
echo "  https://github.com/uutanyang/clipboard/releases"
echo ""
