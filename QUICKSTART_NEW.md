# 快速启动指南 - 贴立方

## 🎉 新功能已就绪！

已完成以下高优先级任务：
- ✅ App Store 发布配置
- ✅ 版本号统一管理
- ✅ 应用图标生成脚本
- ✅ 隐私政策页面
- ✅ 首次启动引导

---

## 🚀 立即测试

### 方法 1: 开发环境测试

```bash
# 1. 启动应用
npm run tauri dev

# 2. 清除 localStorage 以查看首次启动引导
# 在浏览器控制台运行：
localStorage.removeItem('welcome_shown')
# 然后刷新页面

# 3. 测试项：
# - 是否显示欢迎引导
# - 点击"下一步"查看各个权限说明
# - 完成引导后，点击设置中的"查看隐私政策"
```

### 方法 2: 运行自动化测试

```bash
# 测试新功能
./test-new-features.sh

# 查看详细任务进度
cat TASK_PROGRESS.md
```

---

## 📦 准备 App Store 发布

### 步骤 1: 准备图标

```bash
# 1. 准备 1024x1024 的应用图标
# 将你的图标复制到：
src-tauri/icons/icon.png

# 2. 生成所有所需尺寸的图标
./scripts/generate-icons.sh

# 3. 查看 App Store 图标
open src-tauri/icons/appstore/
```

### 步骤 2: 更新版本号

```bash
# 更新版本号到 1.0.0
./scripts/update-version.sh 1.0.0

# 这会自动更新：
# - package.json
# - src-tauri/Cargo.toml
# - src-tauri/tauri.conf.json
# - 创建 Git 标签
```

### 步骤 3: 配置 Apple Developer

```bash
# 1. 查看可用的签名身份
security find-identity -v -p codesigning

# 2. 生成 App 专用密码（用于公证）
# 登录 https://appleid.apple.com/
# 安全 -> App 专用密码 -> 生成

# 3. 记录以下信息：
# - Team ID
# - 签名身份 (Signing Identity)
# - App 专用密码
```

### 步骤 4: 构建 App Store 版本

```bash
# 设置环境变量
export APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAM_ID)"
export APPLE_ID="your@email.com"
export TEAM_ID="YOUR_TEAM_ID"
export APPLE_PASSWORD="app-specific-password"

# 构建（自动签名和公证）
./scripts/build-appstore.sh 1.0.0

# 输出位置：
# src-tauri/target/release/bundle/dmg/贴立方_1.0.0_universal.dmg
# src-tauri/target/release/bundle/pkg/贴立方_1.0.0_universal.pkg
```

### 步骤 5: 上传到 App Store Connect

```bash
# 方法 1: 使用 Transporter（推荐）
open -a Transporter
# 拖拽 .pkg 文件到 Transporter

# 方法 2: 使用命令行
xcrun altool --upload-app \
  --type osx \
  --file src-tauri/target/release/bundle/pkg/贴立方_1.0.0_universal.pkg \
  --username "your@email.com" \
  --password "app-specific-password"
```

---

## 📋 发布检查清单

### 代码准备
- [x] 所有高优先级任务已完成
- [ ] 版本号已更新为 1.0.0
- [ ] 所有功能测试通过
- [ ] 兼容性测试通过
- [ ] 性能测试通过

### 资源准备
- [ ] 1024x1020 应用图标已准备
- [ ] 各分辨率截图已准备
- [ ] 应用描述已填写
- [ ] 隐私政策已发布
- [ ] 审核信息已填写

### 账号准备
- [ ] Apple Developer 账号已注册
- [ ] App Store Connect 应用已创建
- [ ] Bundle ID 已配置
- [ ] 签名证书已配置
- [ ] 公证凭证已配置

### 发布流程
- [ ] 构建文件已生成
- [ ] 构建文件已签名
- [ ] 构建文件已公证
- [ ] 文件已上传到 App Store Connect
- [ ] 已提交审核

---

## 📚 相关文档

| 文档 | 说明 |
|------|------|
| `APPSTORE_CONFIG.md` | 完整的 App Store 发布指南 |
| `TASK_PROGRESS.md` | 任务进度跟踪和待办事项 |
| `IMPLEMENTATION_SUMMARY.md` | 实施总结和已完成任务 |
| `test-new-features.sh` | 自动化测试脚本 |
| `QUICKSTART_NEW.md` | 本文档 |

---

## 🔗 快速链接

### 测试命令
```bash
# 测试新功能
./test-new-features.sh

# 功能测试
chmod +x test-functional.sh
./test-functional.sh

# 启动开发服务器
npm run tauri dev
```

### 构建命令
```bash
# 更新版本号
./scripts/update-version.sh 1.0.0

# 生成图标
./scripts/generate-icons.sh

# 构建 App Store 版本
./scripts/build-appstore.sh 1.0.0
```

### 查看文档
```bash
# 查看发布指南
cat APPSTORE_CONFIG.md

# 查看任务进度
cat TASK_PROGRESS.md

# 查看实施总结
cat IMPLEMENTATION_SUMMARY.md
```

---

## 💡 提示

1. **首次启动引导**: 删除 localStorage 可再次看到
   ```bash
   # 在浏览器控制台运行
   localStorage.removeItem('welcome_shown')
   ```

2. **查看隐私政策**: 打开设置 -> 点击"隐私政策" -> "查看"

3. **测试局域网同步**: 需要两个设备或两个实例

4. **测试文件传输**: 点击顶部"文件"按钮

5. **查看日志**: 实时查看应用运行日志
   ```bash
   tail -f logs/instance1.log
   ```

---

## 🆘 遇到问题？

1. **编译错误**: 运行 `npm run build` 查看详情
2. **构建失败**: 检查 `src-tauri/tauri.conf.json` 配置
3. **签名失败**: 确认 Apple Developer 账号和证书
4. **公证失败**: 确认 App 专用密码正确
5. **功能问题**: 运行 `./test-functional.sh` 全面测试

---

## 📞 联系方式

- GitHub Issues: https://github.com/yourusername/tie-lifang/issues
- 邮箱: your@email.com

---

**准备就绪，开始发布吧！** 🚀
