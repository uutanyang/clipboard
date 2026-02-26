# 草果剪贴板

一个基于 Tauri 2.0 + TypeScript + Vue 3 的剪贴板管理应用。

## 功能特性

- ✅ 自动监听剪贴板变化
- ✅ 支持文本历史记录
- ✅ 本地 SQLite 存储
- ✅ 支持搜索剪贴板内容
- ✅ 快捷键呼出面板（类似 Alfred/Raycast）
- ✅ 键盘导航支持
- ✅ 美观的渐变 UI 界面

## 技术栈

- **前端**: Vue 3 + TypeScript + Vite
- **后端**: Tauri 2.0 + Rust
- **数据库**: SQLite (rusqlite)
- **剪贴板**: arboard

## 快捷键

- **呼出/隐藏面板**: `Cmd/Ctrl + Shift + V`
- **选择记录**: `↑ / ↓` 箭头键
- **复制选中内容**: `Enter`
- **隐藏面板**: `Esc`

## 开发

### 安装依赖

```bash
npm install
```

### 运行开发环境

```bash
npm run tauri dev
```

### 构建生产版本

```bash
npm run tauri build
```

## 项目结构

```
clipboard-caoguo/
├── src/                 # Vue 前端源码
│   ├── App.vue         # 主应用组件
│   ├── main.ts         # 入口文件
│   └── style.css       # 全局样式
├── src-tauri/          # Rust 后端源码
│   ├── src/
│   │   └── lib.rs      # Tauri 命令和逻辑
│   ├── Cargo.toml      # Rust 依赖配置
│   └── tauri.conf.json # Tauri 配置
└── package.json        # Node.js 依赖配置
```

## 数据存储

剪贴板数据存储在本地的 `clipboard.db` SQLite 数据库中，包含以下字段：

- `id`: 记录 ID
- `content_type`: 内容类型（text/image）
- `content`: 内容内容（文本为纯文本，图片为 base64）
- `created_at`: 创建时间（ISO 8601 格式）

## 未来计划

- [ ] 支持图片剪贴板记录
- [ ] 支持富文本格式
- [ ] 云同步功能
- [ ] 更多自定义快捷键
- [ ] 导入/导出功能
- [ ] 深色模式
- [ ] 更多主题选择

## 许可证

MIT License
