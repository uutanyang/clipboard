# App Store 发布配置指南

## 证书配置

### 1. 查看 Apple Developer 账号信息

```bash
# 查看 Team ID
security find-identity -v -p codesigning

# 查看 provisioning profiles
ls -la ~/Library/MobileDevice/Provisioning\ Profiles/
```

### 2. 配置签名身份

编辑 `src-tauri/tauri.conf.json`，修改 `bundle.macOS.signingIdentity`：

```json
{
  "bundle": {
    "macOS": {
      "signingIdentity": "Developer ID Application: Your Name (TEAM_ID)"
    }
  }
}
```

### 3. 构建签名版本

```bash
# 构建 macOS 应用
npm run tauri build

# 或者指定目标
npm run tauri build -- --target universal-apple-darwin
```

## 公证配置

### 使用 notarytool 进行公证

首次使用需要配置 App 专用密码：

1. 登录 [Apple ID](https://appleid.apple.com/)
2. 生成 App 专用密码（用于 notarytool）
3. 获取 Team ID

### 公证命令

```bash
# 上传 DMG 到 Apple Notary Service
xcrun notarytool submit \
  src-tauri/target/release/bundle/dmg/贴立方_0.1.0_universal.dmg \
  --apple-id "your@email.com" \
  --password "app-specific-password" \
  --team-id "YOUR_TEAM_ID" \
  --wait

# 装订公证票据
xcrun stapler staple \
  src-tauri/target/release/bundle/dmg/贴立方_0.1.0_universal.dmg

# 验证公证状态
xcrun stapler validate \
  src-tauri/target/release/bundle/dmg/贴立方_0.1.0_universal.dmg
```

### 查看公证历史

```bash
xcrun notarytool history \
  --apple-id "your@email.com" \
  --password "app-specific-password" \
  --team-id "YOUR_TEAM_ID"
```

## App Store Connect 配置

### 1. 创建应用记录

1. 登录 [App Store Connect](https://appstoreconnect.apple.com/)
2. 点击 "我的 App" → "创建新 App"
3. 填写应用信息：
   - 平台：Mac
   - 名称：贴立方
   - 主要语言：简体中文
   - 套件 ID：com.yangtanfang.tie-lifang
   - SKU：TIE-LIFANG-001

### 2. 上传构建版本

```bash
# 使用 Transporter 上传
open -a Transporter
# 然后拖拽 .pkg 或 .dmg 文件到 Transporter

# 或者使用命令行
xcrun altool --upload-app \
  --type osx \
  --file src-tauri/target/release/bundle/pkg/贴立方_0.1.0_universal.pkg \
  --username "your@email.com" \
  --password "app-specific-password"
```

### 3. 填写应用信息

#### 基本信息
- **名称**: 贴立方
- **副标题**: 现代化的剪贴板管理工具
- **类别**: 生产力工具

#### 描述

**简体中文:**
```
贴立方是一款现代化的剪贴板管理工具，帮助你高效管理复制的所有内容。

主要功能：
• 自动监听剪贴板变化，实时保存历史记录
• 支持文本和图片的剪贴板管理
• 快捷键呼出面板（Cmd+Shift+V），快速选择粘贴
• 强大的搜索功能，快速定位历史内容
• 局域网设备同步，多设备共享剪贴板
• 文件传输功能，快速分享文件
• 本地 SQLite 存储，数据安全私密
• 美观的渐变 UI 设计，流畅的交互体验
• 开机自启动，随时待命

适用场景：
• 程序员快速复用代码片段
• 文字工作者编辑文章
• 设计师管理素材
• 任何需要频繁复制粘贴的用户

提升你的工作效率，让剪贴板管理变得更简单！
```

**English:**
```
Tie Lifang is a modern clipboard management tool that helps you efficiently manage all your copied content.

Key Features:
• Automatically monitor clipboard changes and save history in real-time
• Support for text and image clipboard management
• Quick access panel via shortcut keys (Cmd+Shift+V)
• Powerful search functionality to quickly locate historical content
• LAN device synchronization, share clipboard across devices
• File transfer functionality, quickly share files
• Local SQLite storage ensures data privacy and security
• Beautiful gradient UI design with smooth interactions
• Auto-start at login, always ready

Perfect for developers, writers, designers, and anyone who frequently copies and pastes content.
```

#### 关键词
```
剪贴板,clipboard,管理,复制粘贴,历史记录,搜索,同步,文件传输,效率工具,productivity
```

#### 支持网址
```
https://github.com/yourusername/tie-lifang/issues
```

#### 营销网址
```
https://github.com/yourusername/tie-lifang
```

### 4. 隐私政策 URL

创建隐私政策页面或使用 GitHub Pages 托管：

**示例内容:**
```
隐私政策

数据收集：
本应用不收集任何用户数据。所有剪贴板内容仅存储在本地数据库中，不会上传到任何服务器。

数据存储：
剪贴板数据存储在用户设备的本地 SQLite 数据库中，只有用户本人可以访问。

数据删除：
用户可以通过应用内的"清空所有记录"功能删除所有历史数据。卸载应用后，所有数据也会被删除。

权限使用：
• 剪贴板访问权限：用于监听和复制剪贴板内容
• 网络访问权限：用于局域网设备发现和数据同步
• 文件访问权限：用于保存图片和接收文件

数据共享：
本应用不会与任何第三方共享用户数据，除非用户主动进行局域网同步。

联系我们：
如有隐私相关问题，请联系：your@email.com
```

### 5. 审核信息

#### 审核说明
```
1. 应用功能说明：这是一个剪贴板管理工具，支持文本和图片记录、局域网同步、文件传输等功能

2. 测试账号：无需账号，应用可直接使用

3. 特殊说明：
   - 应用需要用户授予剪贴板访问权限
   - 应用使用本地网络进行设备发现和同步
   - 应用使用 mDNS 协议发现局域网设备
   - 应用支持开机自启动功能

4. 演示视频（可选）：展示应用主要功能
```

#### 联系信息
- **姓名**: Your Name
- **邮箱**: your@email.com
- **电话**: +86 138xxxx8888

## 版本号管理

### 统一版本号脚本

创建 `scripts/update-version.sh`:

```bash
#!/bin/bash

VERSION=$1

if [ -z "$VERSION" ]; then
  echo "Usage: ./update-version.sh <version>"
  exit 1
fi

# 更新 package.json
sed -i '' "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" package.json

# 更新 Cargo.toml
sed -i '' "s/^version = \".*\"/version = \"$VERSION\"/" src-tauri/Cargo.toml

# 更新 tauri.conf.json
sed -i '' "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" src-tauri/tauri.conf.json

echo "Version updated to $VERSION"
```

### 使用方法

```bash
chmod +x scripts/update-version.sh
./scripts/update-version.sh 1.0.0
```

## 构建脚本

### 构建 Universal Binary

```bash
#!/bin/bash

# 构建通用版本
npm run tauri build -- --target universal-apple-darwin

# 查看构建结果
ls -lh src-tauri/target/release/bundle/dmg/
ls -lh src-tauri/target/release/bundle/pkg/
```

### 构建并公证完整流程

```bash
#!/bin/bash

VERSION="1.0.0"
APP_NAME="贴立方"
DMG_PATH="src-tauri/target/release/bundle/dmg/${APP_NAME}_${VERSION}_universal.dmg"

# 1. 构建
echo "Building $VERSION..."
npm run tauri build -- --target universal-apple-darwin

# 2. 签名
echo "Signing..."
codesign --deep --force --verify --verbose \
  --sign "Developer ID Application: Your Name (TEAM_ID)" \
  "src-tauri/target/release/bundle/macos/${APP_NAME}.app"

# 3. 公证
echo "Notarizing..."
xcrun notarytool submit "$DMG_PATH" \
  --apple-id "your@email.com" \
  --password "app-specific-password" \
  --team-id "YOUR_TEAM_ID" \
  --wait

# 4. 装订
echo "Stapling..."
xcrun stapler staple "$DMG_PATH"

# 5. 验证
echo "Validating..."
xcrun stapler validate "$DMG_PATH"

echo "Build complete: $DMG_PATH"
```

## 审核常见问题

### Q: 为什么需要剪贴板访问权限？
A: 应用需要访问剪贴板来监听和管理用户的剪贴板历史记录。

### Q: 为什么需要网络访问权限？
A: 应用使用局域网网络来发现其他设备并进行剪贴板同步和文件传输。

### Q: 应用是否会上传用户数据？
A: 不会。所有数据都存储在本地，局域网同步仅在用户设备之间传输，不会上传到任何服务器。

### Q: 应用是否符合 App Store 审核指南？
A: 是的，应用遵循所有 App Store 审核指南，包括数据隐私、权限使用等规定。

## 检查清单

发布前请确认以下事项：

- [ ] 已注册 Apple Developer Program
- [ ] 已创建 App Store Connect 应用记录
- [ ] 已配置代码签名证书
- [ ] 已配置公证凭证
- [ ] 版本号已统一更新
- [ ] 应用图标符合规范（1024x1024）
- [ ] 截图已准备好（各分辨率）
- [ ] 应用描述已填写
- [ ] 隐私政策已发布
- [ ] 审核信息已填写
- [ ] 已测试所有功能
- [ ] 已在不同 macOS 版本上测试
- [ ] 已在 Intel 和 Apple Silicon 上测试
- [ ] 构建文件已签名
- [ ] 构建文件已公证
- [ ] 已上传到 App Store Connect
- [ ] 已提交审核

---

**祝发布顺利！**
