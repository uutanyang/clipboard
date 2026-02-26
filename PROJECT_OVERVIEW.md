# 草果剪贴板 - 项目概览

## 项目简介

草果剪贴板是一个现代化的剪贴板管理工具，采用 Tauri 2.0 + TypeScript + Vue 3 技术栈开发。它提供了类似 Alfred/Raycast 的快捷键呼出面板，支持文本历史记录、本地 SQLite 存储和搜索功能。

## 核心功能

### ✅ 已实现功能

1. **剪贴板监听**
   - 后台自动监听剪贴板变化
   - 实时保存到本地数据库
   - 500ms 轮询间隔，平衡性能与响应速度

2. **历史记录管理**
   - 支持文本历史记录（最多 100 条）
   - 按时间倒序排列
   - 显示相对时间（刚刚、X分钟前、X小时前等）
   - 支持删除单条记录
   - 支持清空所有记录

3. **搜索功能**
   - 实时搜索历史记录
   - 模糊匹配
   - 搜索框即时反馈

4. **快捷键系统**
   - `Cmd/Ctrl + Shift + V`: 呼出/隐藏面板
   - `↑/↓`: 键盘导航选择记录
   - `Enter`: 复制选中内容
   - `Esc`: 隐藏面板

5. **现代化 UI**
   - 渐变背景设计（紫色渐变）
   - 玻璃拟态效果（backdrop-filter）
   - 响应式布局
   - 平滑动画过渡
   - 悬停效果和交互反馈

## 技术架构

### 前端架构

```
Vue 3 + TypeScript
├── Composition API
├── 响应式数据管理
├── 事件监听
└── Tauri API 集成
```

**关键文件**:
- `src/App.vue`: 主应用组件（~300 行）
- `src/main.ts`: 应用入口
- `src/style.css`: 全局样式

### 后端架构

```
Tauri 2.0 + Rust
├── 数据库层（rusqlite）
├── 剪贴板监听（arboard）
├── 全局快捷键（global-shortcut）
└── 异步任务（tokio）
```

**关键文件**:
- `src-tauri/src/lib.rs`: 主逻辑（~275 行）
- `src-tauri/Cargo.toml`: 依赖配置
- `src-tauri/tauri.conf.json`: 应用配置

### 数据库设计

```sql
CREATE TABLE clipboard_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content_type TEXT NOT NULL,  -- 'text' or 'image'
    content TEXT NOT NULL,       -- 内容（base64 for images）
    created_at TEXT NOT NULL      -- ISO 8601 格式
)
```

**索引**: 在 `created_at` 字段上自动创建索引（主键）

## 技术栈详情

### 依赖库

#### 前端依赖
- `vue@^3.5.13`: Vue 3 框架
- `@tauri-apps/api@^2`: Tauri API
- `@tauri-apps/plugin-opener@^2`: URL 打开插件

#### 后端依赖
- `tauri@2`: Tauri 框架
- `tauri-plugin-global-shortcut@2`: 全局快捷键插件
- `rusqlite@0.32`: SQLite 数据库
- `arboard@3.4`: 跨平台剪贴板访问
- `chrono@0.4`: 时间处理
- `tokio@1`: 异步运行时
- `serde@1`: 序列化/反序列化
- `base64@0.22`: Base64 编码
- `image@0.25`: 图像处理

### 开发工具

- `Vite@6.0`: 前端构建工具
- `TypeScript@5.6`: 类型检查
- `vue-tsc@2.1`: Vue TypeScript 编译器
- `Cargo`: Rust 包管理器

## 文件结构

```
clipboard-caoguo/
├── src/                          # 前端源码
│   ├── App.vue                  # 主应用组件
│   ├── main.ts                  # 应用入口
│   ├── style.css                # 全局样式
│   └── vite-env.d.ts            # Vite 类型定义
├── src-tauri/                    # 后端源码
│   ├── src/
│   │   └── lib.rs              # Tauri 命令和业务逻辑
│   ├── capabilities/
│   │   └── default.json        # 权限配置
│   ├── Cargo.toml              # Rust 依赖
│   ├── tauri.conf.json         # Tauri 配置
│   └── icons/                  # 应用图标
├── public/                      # 静态资源
├── dist/                        # 构建输出
├── package.json                 # Node.js 配置
├── tsconfig.json               # TypeScript 配置
├── vite.config.ts              # Vite 配置
├── README.md                   # 项目说明
├── USAGE.md                    # 使用说明
└── PROJECT_OVERVIEW.md         # 项目概览（本文件）
```

## 核心实现

### 剪贴板监听机制

```rust
// 每 500ms 检查一次剪贴板
loop {
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        if let Ok(text) = clipboard.get_text() {
            if !text.is_empty() && text != last_text {
                // 保存到数据库并通知前端
            }
        }
    }
}
```

### Tauri 命令

后端提供了以下 Tauri 命令供前端调用：

| 命令 | 功能 |
|------|------|
| `get_all_items` | 获取所有剪贴板记录 |
| `search_items` | 搜索剪贴板记录 |
| `delete_item` | 删除单条记录 |
| `clear_all` | 清空所有记录 |
| `copy_text` | 复制文本到剪贴板 |
| `hide_window` | 隐藏窗口 |

### 事件通信

后端通过事件向前端推送更新：

```rust
window.emit("clipboard-update", item)
```

前端监听事件：

```typescript
listen<ClipboardItem>('clipboard-update', (event) => {
  items.value.unshift(event.payload)
})
```

## 性能优化

1. **数据库查询优化**
   - 使用索引加速查询
   - 限制返回结果数量（100 条）
   - 使用预处理语句防止 SQL 注入

2. **内存管理**
   - 使用 `Arc<Mutex<>>` 共享状态
   - 避免不必要的数据克隆

3. **UI 优化**
   - 虚拟滚动（待实现）
   - 懒加载图片（待实现）
   - 防抖搜索

## 安全性

1. **数据隐私**
   - 所有数据存储在本地
   - 不上传到任何服务器
   - SQLite 文件加密（可选）

2. **输入验证**
   - 数据库参数化查询
   - Rust 类型安全
   - TypeScript 类型检查

3. **权限管理**
   - 最小权限原则
   - 通过 Tauri capabilities 控制权限

## 未来计划

### 短期目标
- [ ] 支持图片剪贴板记录
- [ ] 支持富文本格式
- [ ] 添加深色模式
- [ ] 优化搜索性能

### 中期目标
- [ ] 云同步功能
- [ ] 导入/导出功能
- [ ] 更多主题选择
- [ ] 快捷键自定义

### 长期目标
- [ ] 插件系统
- [ ] AI 智能分类
- [ ] 跨设备同步
- [ ] 移动端支持

## 开发指南

### 添加新的 Tauri 命令

1. 在 `src-tauri/src/lib.rs` 中定义命令函数
2. 使用 `#[tauri::command]` 宏标记
3. 在 `generate_handler!` 宏中注册命令
4. 在前端使用 `invoke()` 调用

### 添加新功能

1. **后端**: 在 `lib.rs` 中实现 Rust 逻辑
2. **数据库**: 添加必要的表和索引
3. **前端**: 在 `App.vue` 中添加 UI 和交互
4. **测试**: 手动测试功能完整性

### 调试技巧

- **前端**: 使用浏览器 DevTools
- **后端**: 使用 `println!` 或 Rust 调试器
- **数据库**: 使用 SQLite 客户端查看数据

## 构建和部署

### 开发构建

```bash
npm run tauri dev
```

### 生产构建

```bash
npm run tauri build
```

输出位置: `src-tauri/target/release/bundle/`

### 支持平台

- ✅ macOS (Intel & Apple Silicon)
- ✅ Windows
- ✅ Linux

## 贡献指南

欢迎贡献代码、报告问题或提出建议！

1. Fork 项目
2. 创建特性分支
3. 提交更改
4. 发起 Pull Request

## 许可证

MIT License - 详见 LICENSE 文件

## 联系方式

- 项目地址: https://github.com/yourusername/caoguo-clipboard
- 问题反馈: GitHub Issues

---

**草果剪贴板** - 让剪贴板管理更简单、更高效！
