# 应用测试说明

## 当前问题诊断

### 已确认的问题
1. **前端资源缺失**: 应用包的 Resources 目录中没有前端文件
2. **应用进程运行**: `tauri-app` 进程正常运行
3. **窗口不可见**: 由于前端资源缺失，窗口无法正常渲染

### 根本原因
Tauri 2.0 将前端资源嵌入到二进制文件中，而不是作为单独的文件。如果前端资源没有正确嵌入，应用将无法显示任何内容。

## 解决方案

### 方案一：使用开发模式（推荐用于测试）

```bash
npm run tauri dev
```

这将：
1. 启动 Vite 开发服务器（http://localhost:1421）
2. 启动 Tauri 应用
3. 应用会从开发服务器加载前端

### 方案二：完整重新打包

```bash
# 清理旧构建
rm -rf src-tauri/target/release

# 重新构建
npm run tauri build
```

### 方案三：检查二进制文件中嵌入的资源

```bash
# 查看二进制文件中的资源
cd src-tauri/target/release
strings tauri-app | grep -i "index.html\|vite" | head -20
```

## 测试步骤

### 1. 开发模式测试

```bash
# 启动开发服务器
npm run tauri dev
```

预期结果：
- Vite 服务器在 1421 端口启动
- Tauri 窗口应该出现
- 按 Cmd+Shift+V 可以显示/隐藏窗口

### 2. 检查窗口可见性

开发模式启动后：

```bash
# 列出所有窗口
osascript -e 'tell application "System Events" to get name of every window'

# 切换到应用
osascript -e 'tell application "System Events" to set frontmost of process "tauri-app" to true'
```

### 3. 测试快捷键

1. 确保应用正在运行
2. 按 `Cmd + Shift + V`
3. 应该看到剪贴板面板

### 4. 测试剪贴板功能

1. 复制一些文本（Cmd+C）
2. 按 Cmd+Shift+V 呼出面板
3. 应该看到刚复制的文本
4. 可以搜索、选择、复制

## 调试工具

### 查看应用日志

```bash
# 实时查看日志
tail -f ~/Library/Logs/com.yangtanfang.caoguo-clipboard/tauri-app.log

# 或使用 Console.app
open /Applications/Utilities/Console.app
```

### 检查进程

```bash
# 查看应用进程
ps aux | grep tauri-app

# 终止应用
pkill -9 tauri-app
```

### 检查网络连接（开发模式）

```bash
# 查看端口 1421 是否被监听
lsof -i:1421

# 测试开发服务器
curl http://localhost:1421
```

## 临时工作区

如果需要快速测试而不重新打包：

### 使用开发服务器
```bash
# 终端 1: 启动 Vite
cd /Users/yangtanfang/project/2026/4cool/clipboard-caoguo
npm run dev

# 终端 2: 启动 Tauri
cd /Users/yangtanfang/project/2026/4cool/clipboard-caoguo/src-tauri
cargo run
```

### 直接测试前端
```bash
# 在浏览器中打开前端
open dist/index.html
```

## 已知限制

1. **无边框窗口**: 窗口没有标题栏和边框，可能难以发现
2. **alwaysOnTop**: 窗口始终在最上层
3. **窗口隐藏**: 默认隐藏，需要快捷键呼出

## 修改建议

如果您想更容易地看到窗口，可以修改 `src-tauri/tauri.conf.json`:

```json
{
  "app": {
    "windows": [
      {
        "title": "草果剪贴板",
        "width": 700,
        "height": 500,
        "resizable": true,        // 允许调整大小
        "center": true,
        "decorations": true,      // 显示标题栏和边框
        "alwaysOnTop": false,     // 不始终在最上层
        "skipTaskbar": false,     // 在 Dock 中显示
        "visible": true          // 启动时可见
      }
    ]
  }
}
```

修改后需要重新编译。

## 下一步

1. **先使用开发模式测试**: `npm run tauri dev`
2. **验证功能是否正常**: 测试快捷键、剪贴板、搜索等
3. **确认前端资源问题**: 查看日志确认具体错误
4. **重新打包**: 确保前端资源正确嵌入

## 联系支持

如果以上方案都无法解决问题，请提供以下信息：

1. 控制台错误输出
2. 应用日志文件内容
3. 系统版本（`sw_vers`）
4. 完整的构建输出

---

**祝您测试顺利！** 🍎
