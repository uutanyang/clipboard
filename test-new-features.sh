#!/bin/bash

# 测试新实施的功能

set -e

echo "🧪 开始测试新功能..."
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

# 2. 检查新组件是否存在
echo "📦 2. 检查新组件..."

components=(
  "src/components/PrivacyPolicy.vue"
  "src/components/WelcomeGuide.vue"
)

for component in "${components[@]}"; do
  if [ -f "$component" ]; then
    echo "  ✓ $component 存在"
  else
    echo "  ✗ $component 不存在"
    exit 1
  fi
done

# 3. 检查脚本
echo "📦 3. 检查构建脚本..."

scripts=(
  "scripts/update-version.sh"
  "scripts/build-appstore.sh"
  "scripts/generate-icons.sh"
)

for script in "${scripts[@]}"; do
  if [ -f "$script" ]; then
    # 检查脚本是否可执行
    if [ -x "$script" ]; then
      echo "  ✓ $script 存在且可执行"
    else
      echo "  ⚠️  $script 存在但不可执行"
    fi
  else
    echo "  ✗ $script 不存在"
    exit 1
  fi
done

# 4. 检查配置文件
echo "📦 4. 检查配置文件..."

configs=(
  "src-tauri/entitlements.plist"
  "src-tauri/entitlements.mas.plist"
  "APPSTORE_CONFIG.md"
  "TASK_PROGRESS.md"
)

for config in "${configs[@]}"; do
  if [ -f "$config" ]; then
    echo "  ✓ $config 存在"
  else
    echo "  ✗ $config 不存在"
    exit 1
  fi
done

# 5. 检查 App.vue 集成
echo "📦 5. 检查 App.vue 集成..."

if grep -q "PrivacyPolicy" src/App.vue; then
  echo "  ✓ PrivacyPolicy 已集成"
else
  echo "  ✗ PrivacyPolicy 未集成"
  exit 1
fi

if grep -q "WelcomeGuide" src/App.vue; then
  echo "  ✓ WelcomeGuide 已集成"
else
  echo "  ✗ WelcomeGuide 未集成"
  exit 1
fi

# 6. 检查隐私政策按钮
if grep -q "privacyPolicy?.openPrivacyPolicy" src/App.vue; then
  echo "  ✓ 隐私政策按钮已添加"
else
  echo "  ⚠️  隐私政策按钮未添加"
fi

# 7. 测试版本号更新脚本
echo "📦 6. 测试版本号更新脚本..."

# 创建临时备份
cp package.json package.json.backup
cp src-tauri/Cargo.toml src-tauri/Cargo.toml.backup
cp src-tauri/tauri.conf.json src-tauri/tauri.conf.json.backup

# 运行更新脚本
./scripts/update-version.sh 9.9.9 > /dev/null 2>&1

# 检查版本号是否更新
if grep -q "9.9.9" package.json && \
   grep -q 'version = "9.9.9"' src-tauri/Cargo.toml && \
   grep -q '"version": "9.9.9"' src-tauri/tauri.conf.json; then
  echo "  ✓ 版本号更新脚本工作正常"

  # 恢复原始版本号
  mv package.json.backup package.json
  mv src-tauri/Cargo.toml.backup src-tauri/Cargo.toml
  mv src-tauri/tauri.conf.json.backup src-tauri/tauri.conf.json
else
  echo "  ✗ 版本号更新脚本失败"
  # 恢复备份
  mv package.json.backup package.json 2>/dev/null || true
  mv src-tauri/Cargo.toml.backup src-tauri/Cargo.toml 2>/dev/null || true
  mv src-tauri/tauri.conf.json.backup src-tauri/tauri.conf.json 2>/dev/null || true
  exit 1
fi

echo ""
echo "=================================="
echo "✅ 所有测试通过！"
echo ""
echo "新功能清单:"
echo "  1. App Store 发布配置"
echo "  2. 版本号统一管理"
echo "  3. 应用图标生成脚本"
echo "  4. 隐私政策页面"
echo "  5. 首次启动引导"
echo ""
echo "下一步:"
echo "  1. 运行应用测试新功能: npm run tauri dev"
echo "  2. 查看 TASK_PROGRESS.md 了解详细进度"
echo "  3. 查看 APPSTORE_CONFIG.md 了解发布流程"
echo ""
