# 图片存储架构重构

## 问题背景

之前的图片存储方式是将图片转换为 base64 并存储在 SQLite 数据库中。这种方式存在以下问题：

1. **数据库膨胀**: 图片 base64 数据量大，导致数据库文件快速增长
2. **性能问题**: 每次查询都需要传输大量数据，前端需要解码 base64
3. **加载困难**: 大图片的 base64 编码很长，浏览器加载困难

## 新的架构设计

### 1. 文件存储结构

```
{app_data_dir}/
└── images/
    ├── img_20260217_123456_12345.jpg
    ├── img_20260217_123457_67890.jpg
    └── ...
```

### 2. 数据库结构变更

**旧结构**:
```sql
CREATE TABLE clipboard_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content_type TEXT NOT NULL,
    content TEXT NOT NULL,  -- 图片存 base64
    created_at TEXT NOT NULL
)
```

**新结构**:
```sql
CREATE TABLE clipboard_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content_type TEXT NOT NULL,
    content TEXT,              -- 文本内容，图片为空
    file_path TEXT,            -- 图片文件路径（相对路径）
    created_at TEXT NOT NULL
)
```

### 3. 数据流程

#### 图片保存流程

```
1. 用户截图/复制图片
   ↓
2. 后端监听到剪贴板变化
   ↓
3. 将图片编码为 JPEG
   ↓
4. 生成唯一文件名 (时间戳 + 随机数)
   ↓
5. 保存到 {app_data_dir}/images/
   ↓
6. 数据库存储相对路径 (images/xxx.jpg)
```

#### 图片显示流程

```
1. 前端请求获取记录
   ↓
2. 后端将相对路径转换为绝对路径
   ↓
3. 前端使用 convertFileSrc 转换为 URL
   ↓
4. 浏览器通过 asset:// 协议加载图片
```

## 代码变更

### 后端 (Rust)

#### 1. 数据结构

```rust
pub struct ClipboardItem {
    pub id: i64,
    pub content_type: String,
    pub content: String,
    pub file_path: Option<String>,  // 新增
    pub created_at: String,
}
```

#### 2. 图片保存

```rust
// 生成唯一文件名
let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
let random_suffix: u32 = rand::random();
let filename = format!("img_{}_{}.jpg", timestamp, random_suffix);
let relative_path = format!("images/{}", filename);

// 保存图片文件
let app_data_dir = handle.path().app_data_dir()?;
let images_dir = app_data_dir.join("images");
std::fs::create_dir_all(&images_dir)?;
std::fs::write(images_dir.join(&filename), &jpeg_bytes)?;

// 保存到数据库
db.insert_item("image", "", Some(&relative_path))?;
```

#### 3. 图片删除

```rust
pub fn delete_item(&self, id: i64) -> SqlResult<Option<String>> {
    // 先获取文件路径
    let file_path = self.conn.query_row(
        "SELECT file_path FROM clipboard_items WHERE id = ?1",
        [id],
        |row| row.get(0),
    ).ok();

    // 删除数据库记录
    self.conn.execute("DELETE FROM clipboard_items WHERE id = ?1", [id])?;

    Ok(file_path)  // 返回文件路径供外部删除
}
```

### 前端 (Vue)

#### 1. 图片 URL 转换

```typescript
import { convertFileSrc } from '@tauri-apps/api/core'

// 获取图片的可访问 URL
function getImageSrc(item: ClipboardItem): string {
  if (item.file_path) {
    // 使用 Tauri 的 convertFileSrc 将文件路径转换为可访问的 URL
    return convertFileSrc(item.file_path)
  }
  return ''
}
```

#### 2. 图片显示

```vue
<div v-if="item.content_type === 'image' && item.file_path" class="item-image">
  <img
    :src="getImageSrc(item)"
    :alt="`截图-${item.id}`"
    @error="handleImageError"
    loading="lazy"
  />
</div>
```

### Tauri 配置

```json
{
  "app": {
    "security": {
      "csp": "default-src 'self'; img-src 'self' asset: https://asset.localhost data:; style-src 'self' 'unsafe-inline'",
      "assetProtocol": {
        "enable": true,
        "scope": ["**"]
      }
    }
  }
}
```

## 优势

### 1. 性能提升
- **数据库大小**: 大幅减少，不再存储图片数据
- **查询速度**: 更快，数据量小
- **内存占用**: 更低，不需要加载大 base64

### 2. 图片加载
- **浏览器原生支持**: 使用 `asset://` 协议直接加载文件
- **懒加载**: 支持 `loading="lazy"` 属性
- **缓存**: 浏览器自动缓存图片

### 3. 可扩展性
- **图片管理**: 方便批量处理、清理旧图片
- **格式转换**: 可以存储不同格式（JPEG、PNG、WebP）
- **缩略图**: 未来可以生成缩略图提高性能

## 兼容性处理

### 数据库迁移

```rust
// 尝试添加 file_path 列（如果不存在）
let _ = conn.execute(
    "ALTER TABLE clipboard_items ADD COLUMN file_path TEXT",
    [],
);
```

### 旧数据处理

对于已存在的 base64 图片数据，可以：
1. 迁移脚本：将 base64 转为文件
2. 渐进式迁移：访问时转换
3. 清空历史：直接清空旧数据

## 测试要点

### 1. 基本功能
- [ ] 截图后图片正常显示
- [ ] 复制图片后正常显示
- [ ] 图片缩略图正确缩放
- [ ] 点击图片可复制到剪贴板
- [ ] 保存图片到下载目录

### 2. 数据管理
- [ ] 删除记录时图片文件被删除
- [ ] 清空所有时图片目录被清空
- [ ] 数据库大小合理

### 3. 性能
- [ ] 大图片加载流畅
- [ ] 多图片滚动流畅
- [ ] 内存占用正常

### 4. 兼容性
- [ ] 旧数据库可正常打开
- [ ] 应用数据目录正确
- [ ] 文件权限正确

## 后续优化

1. **缩略图生成**: 保存时生成缩略图，提高列表显示性能
2. **图片压缩**: 自动压缩大图片
3. **格式优化**: 支持保存为 WebP 等高效格式
4. **定时清理**: 自动清理过期的图片文件
5. **图片去重**: 基于哈希去重，节省存储空间

---

**更新日期**: 2026-02-17
**版本**: v0.1.0
