# 发布前检查清单

## ✅ 代码检查

- [x] Rust 代码编译通过 (`cargo check`)
- [x] 前端代码编译通过 (`npm run build`)
- [x] TypeScript 类型检查通过
- [x] 无编译错误和警告
- [x] Linter 检查通过（除缓存错误）

## ✅ 功能测试

### 剪贴板监听
- [ ] 能够监听到剪贴板变化
- [ ] 自动保存到数据库
- [ ] 重复内容不重复保存
- [ ] 支持中英文文本

### 历史记录
- [ ] 记录按时间倒序排列
- [ ] 显示相对时间（刚刚/X分钟前）
- [ ] 内容预览正确显示
- [ ] 最多保存 100 条记录

### 搜索功能
- [ ] 实时搜索响应
- [ ] 模糊匹配正确
- [ ] 搜索结果即时显示
- [ ] 清空搜索恢复所有记录

### 快捷键
- [ ] Cmd+Shift+V 呼出面板
- [ ] Cmd+Shift+V 隐藏面板
- [ ] ↑↓ 键盘导航
- [ ] Enter 复制选中内容
- [ ] Esc 隐藏面板

### UI/UX
- [ ] 渐变背景显示正常
- [ ] 玻璃拟态效果正常
- [ ] 悬停效果流畅
- [ ] 响应式布局正常
- [ ] 滚动条样式美观

### 数据管理
- [ ] 删除单条记录
- [ ] 清空所有记录
- [ ] 数据持久化保存
- [ ] 数据库文件创建正确

## ✅ 权限检查

- [ ] 辅助功能权限提示
- [ ] 权限授予后功能正常
- [ ] 隐私信息提示清晰
- [ ] 无过度权限请求

## ✅ 性能测试

- [ ] 启动时间 < 2 秒
- [ ] 面板呼出响应 < 200ms
- [ ] 搜索响应 < 100ms
- [ ] 内存占用 < 100MB
- [ ] CPU 占用 < 5% (空闲时)

## ✅ 兼容性测试

- [ ] macOS 10.15 (Catalina)
- [ ] macOS 11 (Big Sur)
- [ ] macOS 12 (Monterey)
- [ ] macOS 13 (Ventura)
- [ ] macOS 14 (Sonoma)
- [ ] Apple Silicon (M1/M2/M3) - 需要单独构建

## ✅ 安全检查

- [ ] 无敏感信息泄露
- [ ] 数据加密存储（如需要）
- [ ] SQL 注入防护
- [ ] 输入验证完善

## ✅ 打包检查

- [x] DMG 文件生成
- [x] 应用图标正确
- [x] 应用名称正确
- [x] 版本号正确
- [x] 文件大小合理

## ✅ 文档检查

- [x] README.md 完整
- [x] USAGE.md 详细
- [x] BUILD.md 创建
- [x] 代码注释充分
- [x] 使用说明清晰

## 📋 发布准备

### 代码签名（可选但推荐）
```bash
# 查看证书
security find-identity -v -p codesigning

# 签名应用
codesign --deep --force --verify --verbose --sign "Developer ID Application: Your Name" 草果剪贴板.app

# 验证签名
codesign -dv --verbose=4 草果剪贴板.app
```

### 公证（App Store 分发需要）
```bash
# 上传到 Apple Notary Service
xcrun notarytool submit 草果剪贴板_0.1.0_x64.dmg \
  --apple-id "your@email.com" \
  --password "app-specific-password" \
  --team-id "YOUR_TEAM_ID" \
  --wait

# 装订公证票据
xcrun stapler staple 草果剪贴板_0.1.0_x64.dmg
```

### 版本号更新
- [ ] 在 `package.json` 中更新版本号
- [ ] 在 `src-tauri/Cargo.toml` 中更新版本号
- [ ] 在 `src-tauri/tauri.conf.json` 中更新版本号
- [ ] 更新 CHANGELOG.md

### 发布渠道
- [ ] GitHub Releases
- [ ] 官网下载
- [ ] App Store (需额外配置)

## 🎯 快速测试命令

```bash
# 启动测试
npm run tauri dev

# 查看数据库
sqlite3 clipboard.db "SELECT * FROM clipboard_items ORDER BY created_at DESC LIMIT 10;"

# 查看日志
log stream --predicate 'process == "草果剪贴板"' --level debug

# 检查签名
codesign -dv --verbose=4 草果剪贴板.app
```

## 📊 当前状态

- **版本**: 0.1.0
- **平台**: macOS (Intel x64)
- **构建状态**: ✅ 成功
- **DMG 大小**: 4.0 MB
- **编译时间**: ~10 分钟
- **测试状态**: 待用户测试

## ⚠️ 已知限制

1. 仅支持文本剪贴板（图片支持待开发）
2. 仅支持 macOS Intel 架构（Apple Silicon 需单独构建）
3. 需要 macOS 10.15+
4. 需要授予辅助功能权限

## 🔄 后续优化

- [ ] 添加图片支持
- [ ] 支持 Apple Silicon (M1/M2/M3)
- [ ] 添加 Windows/Linux 构建
- [ ] 实现云同步功能
- [ ] 添加更多主题
- [ ] 优化性能和内存占用

---

**发布前请完成以上所有检查项！**
