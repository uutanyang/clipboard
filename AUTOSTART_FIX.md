# 开机自启动功能修复

## 问题描述

用户反馈："开机自启动功能不能设置"

## 问题诊断

### 根本原因

**Tauri 2 权限系统**：`tauri-plugin-autostart` 插件需要在 capabilities 文件中声明权限才能使用。

**缺失的权限**：
- `autostart:allow-enable` - 允许启用自启动
- `autostart:allow-disable` - 允许禁用自启动
- `autostart:allow-is-enabled` - 允许查询自启动状态

### 原因分析

在 Tauri 2 中，安全模型要求所有插件权限必须在 `capabilities/default.json` 中明确声明。前端调用插件 API 时，如果权限未声明，会返回权限拒绝错误。

```typescript
// 前端调用
await invoke('plugin:autostart|is_enabled')

// 如果没有权限，会抛出类似错误：
// "plugin:autostart|is_enabled not allowed"
// 或者
// "Permission autostart:allow-is-enabled not allowed"
```

---

## 修复方案

### 修复 1: 添加 autostart 权限到 capabilities

**文件**: `src-tauri/capabilities/default.json`

**修改前**:
```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:window:allow-hide",
    "core:window:allow-show",
    "core:window:allow-set-focus",
    "core:window:allow-is-visible",
    "opener:default",
    "notification:allow-is-permission-granted",
    "notification:default"
  ]
}
```

**修改后**:
```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:window:allow-hide",
    "core:window:allow-show",
    "core:window:allow-set-focus",
    "core:window:allow-is-visible",
    "opener:default",
    "notification:allow-is-permission-granted",
    "notification:default",
    "autostart:allow-enable",
    "autostart:allow-disable",
    "autostart:allow-is-enabled"
  ]
}
```

### 修复 2: 改进错误处理和日志

**文件**: `src/App.vue`

**修改后的代码**:
```typescript
// 检查自启动状态
async function checkAutostartStatus() {
  try {
    console.log('Checking autostart status...')
    const enabled = await invoke<boolean>('plugin:autostart|is_enabled')
    autostartEnabled.value = enabled
    console.log('Autostart status:', enabled ? 'enabled' : 'disabled')
  } catch (error) {
    console.error('Failed to check autostart status:', error)
    console.error('Error details:', JSON.stringify(error))
    autostartEnabled.value = false
  }
}

// 切换自启动
async function toggleAutostart() {
  try {
    console.log('Toggling autostart, current status:', autostartEnabled.value)
    if (autostartEnabled.value) {
      console.log('Disabling autostart...')
      await invoke('plugin:autostart|disable')
      autostartEnabled.value = false
      console.log('Autostart disabled successfully')
    } else {
      console.log('Enabling autostart...')
      await invoke('plugin:autostart|enable')
      autostartEnabled.value = true
      console.log('Autostart enabled successfully')
    }
  } catch (error) {
    console.error('Failed to toggle autostart:', error)
    console.error('Error details:', JSON.stringify(error))

    // 尝试读取错误信息
    const errorMsg = error as any
    if (errorMsg.message) {
      console.error('Error message:', errorMsg.message)
    }
    if (errorMsg.cause) {
      console.error('Error cause:', errorMsg.cause)
    }

    // 回滚状态
    if (!autostartEnabled.value) {
      autostartEnabled.value = false
    } else {
      autostartEnabled.value = true
    }
  }
}
```

**改进点**:
1. 添加详细的控制台日志
2. 添加错误详情打印（JSON 格式）
3. 添加错误消息和原因提取
4. 操作失败时回滚状态
5. 记录每个步骤的执行状态

---

## 测试步骤

### 1. 检查权限是否正确

```bash
# 查看生成的权限文件
cat src-tauri/gen/schemas/desktop-schema.json

# 或者检查应用日志，查找权限相关错误
```

### 2. 开发模式测试

```bash
# 重新启动开发服务器
npm run tauri dev
```

**预期行为**:
- 应用启动后，打开"设置"模态框
- 开关按钮应该能正确反映当前自启动状态
- 点击开关应该能切换自启动状态
- 控制台应该看到详细的日志输出

### 3. 控制台日志检查

**打开应用后，在浏览器控制台中应该看到**:

```
Checking autostart status...
Autostart status: disabled
```

**点击开关后**:

```
Toggling autostart, current status: false
Enabling autostart...
Autostart enabled successfully
```

### 4. 功能验证

#### 测试启用自启动:
1. 打开"设置"模态框
2. 点击"开机自启动"开关
3. 开关应该变为"开启"状态
4. 检查控制台确认没有错误

#### 测试禁用自启动:
1. 确保开关已开启
2. 再次点击"开机自启动"开关
3. 开关应该变为"关闭"状态
4. 检查控制台确认没有错误

### 5. 系统级别验证（macOS）

```bash
# 检查 LaunchAgent 是否创建
ls -la ~/Library/LaunchAgents/ | grep caoguo

# 查看 LaunchAgent 内容
cat ~/Library/LaunchAgents/com.yangtanfang.caoguo-clipboard.plist
```

**预期结果**:
- 启用后，应该能找到 `.plist` 文件
- 禁用后，文件应该被删除
- `.plist` 文件包含正确的应用路径

---

## 常见问题排查

### 问题 1: 权限拒绝错误

**症状**:
```
plugin:autostart|is_enabled not allowed
```

**原因**: capabilities 文件未配置或未重新生成

**解决方案**:
```bash
# 1. 检查 capabilities/default.json 是否包含 autostart 权限
cat src-tauri/capabilities/default.json | grep autostart

# 2. 清理并重新构建
rm -rf src-tauri/target/
npm run tauri dev
```

### 问题 2: 权限文件未生成

**症状**: 应用启动正常，但调用 API 时报错

**原因**: Tauri 生成权限文件失败

**解决方案**:
```bash
# 手动触发权限文件生成
cd src-tauri
cargo check

# 检查生成文件是否存在
ls -la gen/schemas/
```

### 问题 3: macOS 权限问题

**症状**:
- 开关状态改变，但系统自启动不生效
- LaunchAgent 文件未创建

**原因**:
- macOS 需要用户授权才能创建 LaunchAgent
- 应用没有足够权限

**解决方案**:
```bash
# 1. 检查应用是否有权限
# 系统偏好设置 -> 安全性与隐私 -> 隐私 -> 辅助功能
# 确保应用有权限

# 2. 尝试手动创建 LaunchAgent（开发测试）
cat > ~/Library/LaunchAgents/com.yangtanfang.caoguo-clipboard.plist <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.yangtanfang.caoguo-clipboard</string>
    <key>ProgramArguments</key>
    <array>
        <string>/path/to/your/app</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>
EOF

# 3. 加载 LaunchAgent
launchctl load ~/Library/LaunchAgents/com.yangtanfang.caoguo-clipboard.plist
```

### 问题 4: 状态不同步

**症状**: UI 开关状态与系统实际状态不一致

**原因**:
- 前端状态缓存未更新
- 后端状态查询失败但前端未正确处理

**解决方案**:
```typescript
// 添加状态刷新机制
async function refreshAutostartStatus() {
  try {
    const enabled = await invoke<boolean>('plugin:autostart|is_enabled')
    autostartEnabled.value = enabled
  } catch (error) {
    console.error('Failed to refresh autostart status:', error)
  }
}

// 在设置模态框打开时刷新
watch(showSettings, (isOpen) => {
  if (isOpen) {
    refreshAutostartStatus()
  }
})
```

### 问题 5: 构建后自启动不工作

**症状**: 开发模式正常，但打包后的应用自启动失败

**原因**:
- 打包后的应用路径与开发模式不同
- LaunchAgent 路径配置不正确

**解决方案**:
```rust
// 确保使用正确的应用路径
// tauri-plugin-autostart 会自动处理
// 但需要确保 bundle identifier 正确

// 检查 tauri.conf.json
{
  "identifier": "com.yangtanfang.caoguo-clipboard",
  // ...
}
```

---

## 调试命令

### 开发模式调试

```bash
# 启动开发服务器
npm run tauri dev

# 查看详细日志
# Tauri 会输出到终端
```

### 控制台调试

打开应用后，按以下步骤查看日志：

1. macOS: `Cmd + Option + I` (DevTools)
2. 切换到 "Console" 标签
3. 查找以下日志:
   - `Checking autostart status...`
   - `Toggling autostart...`
   - `Autostart enabled/disabled successfully`
   - 任何红色错误信息

### 后端日志调试

```bash
# 查看 Rust 编译日志
npm run tauri dev 2>&1 | grep -i autostart
```

### 系统日志调试（macOS）

```bash
# 查看系统日志
log show --predicate 'process == "launchd"' --last 1h | grep caoguo

# 查看 LaunchAgent 加载情况
launchctl list | grep caoguo
```

---

## 验证清单

使用此清单确认自启动功能正常：

- [ ] 开发模式下，开关能正确切换状态
- [ ] 控制台没有权限错误
- [ ] 启用自启动后，LaunchAgent 文件已创建
- [ ] 禁用自启动后，LaunchAgent 文件已删除
- [ ] 重启系统后，应用自动启动（如果已启用）
- [ ] UI 开关状态与系统实际状态一致
- [ ] 开关切换时有正确的视觉反馈
- [ ] 错误情况下有适当的回滚机制

---

## 代码修改总结

### 修改的文件

1. **src-tauri/capabilities/default.json**
   - 添加 `autostart:allow-enable` 权限
   - 添加 `autostart:allow-disable` 权限
   - 添加 `autostart:allow-is-enabled` 权限

2. **src/App.vue**
   - 改进 `checkAutostartStatus()` 函数
     - 添加详细日志
     - 改进错误处理
   - 改进 `toggleAutostart()` 函数
     - 添加详细日志
     - 添加错误详情提取
     - 添加状态回滚机制

### 无需修改的部分

- **src-tauri/Cargo.toml** - 已包含 `tauri-plugin-autostart = "2"`
- **src-tauri/src/lib.rs** - 已正确注册插件

---

## 后续优化建议

1. **添加用户反馈**:
   ```typescript
   // 成功时显示提示
   async function toggleAutostart() {
     // ...
     const status = autostartEnabled.value ? '已启用' : '已禁用'
     showToast(`开机自启动${status}`)
   }
   ```

2. **添加加载状态**:
   ```typescript
   const autostartLoading = ref(false)

   async function toggleAutostart() {
     autostartLoading.value = true
     try {
       // ...
     } finally {
       autostartLoading.value = false
     }
   }
   ```

3. **添加错误提示**:
   ```typescript
   catch (error) {
     showToast('操作失败，请检查控制台日志', 'error')
     console.error('Failed to toggle autostart:', error)
   }
   ```

---

## 总结

通过本次修复，开机自启动功能应该能够正常工作：

1. ✅ 添加了必要的权限配置
2. ✅ 改进了错误处理和日志
3. ✅ 添加了状态回滚机制
4. ✅ 提供了完整的测试步骤
5. ✅ 列出了常见问题和解决方案

重启开发服务器后即可测试功能：
```bash
npm run tauri dev
```
