# 异常情况测试与修复方案

## 测试概述

本文档描述了 5 种异常情况的测试方法，以及针对每个测试可能遇到的问题的修复方案。

---

## 测试 1: 设备离线后重连

### 测试目标
验证设备离线后重连时，连接状态能否自动恢复。

### 测试步骤
1. 启动两个实例并配对
2. 验证连接状态为"已连接"
3. 关闭实例 2
4. 等待 30-60 秒，观察状态变为"未连接"
5. 重新启动实例 2
6. 观察连接状态自动恢复为"已连接"

### 预期结果
- ✓ 设备离线后，30 秒内状态变为"未连接"
- ✓ 设备重新上线后，状态自动恢复为"已连接"
- ✓ 配对状态保持，无需重新配对

### 可能的问题与修复

#### 问题 1: 设备离线后状态不更新
**症状**: 设备离线后，连接状态仍显示"已连接"

**原因**:
- `last_seen` 时间戳没有正确更新
- 前端没有定期检查设备在线状态

**修复方案**:

```rust
// src-tauri/src/mdns/mod.rs
// 添加心跳机制，定期更新 last_seen

pub async fn start_heartbeat() {
    let interval = Duration::from_secs(10);
    loop {
        // 更新所有设备的 last_seen 时间
        update_device_last_seen().await;
        tokio::time::sleep(interval).await;
    }
}

pub async fn update_device_last_seen() {
    // 获取所有信任设备
    let trusted_devices = get_trusted_devices().await;

    for device in trusted_devices {
        // 尝试 ping 设备
        if ping_device(&device.ip, device.port).await {
            update_device_seen_time(&device.device_id).await;
        }
    }
}
```

```typescript
// src/App.vue
// 添加定期检查设备在线状态

let checkOnlineInterval: number | null = null

onMounted(async () => {
  // 每 10 秒检查一次设备在线状态
  checkOnlineInterval = window.setInterval(async () => {
    await updateConnectedCount()
  }, 10000)
})

onUnmounted(() => {
  if (checkOnlineInterval) {
    clearInterval(checkOnlineInterval)
  }
})
```

#### 问题 2: 设备重连后不自动恢复连接
**症状**: 设备重新上线后，需要手动刷新才能看到连接状态恢复

**原因**: 没有监听设备上线事件

**修复方案**:

```typescript
// src/App.vue
// 监听设备上线/下线事件

async function setupDeviceOnlineListeners() {
  try {
    await listen('device-online', async () => {
      console.log('Device came online')
      await updateConnectedCount()
    })

    await listen('device-offline', async () => {
      console.log('Device went offline')
      await updateConnectedCount()
    })
  } catch (error) {
    console.error('Failed to setup device online listeners:', error)
  }
}
```

```rust
// src-tauri/src/mdns/mod.rs
// 触发设备上线/下线事件

async fn handle_device_discovery(device: NetworkDevice) {
    let old_device = get_device_by_id(&device.device_id).await;

    // 如果设备之前离线，现在在线，触发上线事件
    if let Some(old) = old_device {
        if is_device_offline(&old.last_seen) && !is_device_offline(&device.last_seen) {
            emit_event('device-online', &device).await;
        } else if !is_device_offline(&old.last_seen) && is_device_offline(&device.last_seen) {
            emit_event('device-offline', &device.device_id).await;
        }
    }
}
```

---

## 测试 2: 同时复制多个内容

### 测试目标
验证快速复制多个内容时，同步是否正常，没有重复或丢失。

### 测试步骤
1. 在 3 秒内快速复制 10 段不同的文本
2. 观察两端剪贴板历史
3. 检查是否有重复内容
4. 验证时间戳是否正确

### 预期结果
- ✓ 两端都包含全部 10 段文本
- ✓ 没有重复内容
- ✓ 时间戳正确且递增
- ✓ 没有内容丢失

### 可能的问题与修复

#### 问题 1: 内容丢失
**症状**: 快速复制时，部分内容没有同步

**原因**: 同步是异步的，新请求覆盖了旧的请求

**修复方案**:

```rust
// src-tauri/src/clipboard/mod.rs
// 添加剪贴板队列，确保顺序同步

use std::collections::VecDeque;
use tokio::sync::Mutex;

static CLIPBOARD_QUEUE: Mutex<VecDeque<ClipboardItem>> = Mutex::const_new(VecDeque::new());

pub async fn queue_clipboard_sync(item: ClipboardItem) {
    // 添加到队列
    CLIPBOARD_QUEUE.lock().await.push_back(item);

    // 如果队列中有任务，启动处理
    process_queue().await;
}

async fn process_queue() {
    while let Some(item) = CLIPBOARD_QUEUE.lock().await.pop_front() {
        // 同步到所有配对设备
        sync_to_devices(item).await;
        // 添加延迟，避免过快同步
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
```

#### 问题 2: 内容重复
**症状**: 同一段内容出现多次

**原因**: 回声问题（A 同步到 B，B 又同步回 A）

**修复方案**:

```rust
// src-tauri/src/clipboard/mod.rs
// 添加同步 ID 避免回声

use uuid::Uuid;

#[derive(Clone)]
pub struct ClipboardItem {
    pub content: String,
    pub sync_id: Uuid,  // 添加同步 ID
}

pub async fn handle_remote_clipboard(item: ClipboardItem) {
    // 检查是否已经处理过此同步
    if is_already_processed(&item.sync_id) {
        return;
    }

    // 标记为已处理
    mark_as_processed(&item.sync_id).await;

    // 更新剪贴板
    update_local_clipboard(&item.content).await;
}
```

---

## 测试 3: 大文件传输（100MB+）

### 测试目标
验证大文件传输的稳定性、进度显示和内存占用。

### 测试步骤
1. 传输 50MB 文件
2. 传输 100MB 文件
3. 传输 200MB 文件（可选）
4. 观察传输进度
5. 检查内存占用
6. 验证文件完整性

### 预期结果
- ✓ 所有文件传输成功
- ✓ 进度条实时更新
- ✓ 内存占用合理（不持续增长）
- ✓ 文件完整性验证通过

### 可能的问题与修复

#### 问题 1: 内存溢出
**症状**: 传输大文件时应用崩溃

**原因**: 一次性加载整个文件到内存

**修复方案**:

```rust
// src-tauri/src/file_transfer/mod.rs
// 使用流式传输，分块处理

pub async fn transfer_file_streaming(
    file_path: &Path,
    target_device: &Device,
    progress_sender: tokio::sync::mpsc::Sender<TransferProgress>
) -> Result<(), TransferError> {
    const CHUNK_SIZE: usize = 1024 * 1024; // 1MB chunks

    let file = File::open(file_path).await?;
    let file_size = file.metadata().await?.len();

    let mut reader = BufReader::with_capacity(CHUNK_SIZE, file);
    let mut buffer = vec![0u8; CHUNK_SIZE];
    let mut sent_bytes: u64 = 0;

    loop {
        let bytes_read = reader.read(&mut buffer).await?;

        if bytes_read == 0 {
            break; // 文件传输完成
        }

        // 发送分块
        send_chunk(&target_device, &buffer[..bytes_read]).await?;

        sent_bytes += bytes_read as u64;

        // 更新进度
        let progress = TransferProgress {
            bytes_sent: sent_bytes,
            total_bytes: file_size,
            percentage: (sent_bytes as f32 / file_size as f32) * 100.0,
        };

        progress_sender.send(progress).await?;

        // 释放内存
        drop(buffer);
        buffer = vec![0u8; CHUNK_SIZE];
    }

    Ok(())
}
```

#### 问题 2: 进度显示不流畅
**症状**: 进度条更新不频繁

**原因**: 前端没有正确处理进度事件

**修复方案**:

```typescript
// src/components/FileTransfer.vue
// 添加进度更新节流

let lastProgressUpdate = 0
const PROGRESS_UPDATE_THROTTLE = 100 // 100ms

function handleTransferProgress(event: TransferProgress) {
  const now = Date.now()

  // 节流更新，避免过于频繁
  if (now - lastProgressUpdate < PROGRESS_UPDATE_THROTTLE) {
    return
  }

  lastProgressUpdate = now

  transferProgress.value = event.percentage
  bytesTransferred.value = event.bytes_sent
  totalBytes.value = event.total_bytes
}
```

#### 问题 3: 文件完整性验证失败
**症状**: 传输完成后文件损坏

**原因**: 网络传输导致数据损坏

**修复方案**:

```rust
// src-tauri/src/file_transfer/mod.rs
// 添加文件校验和验证

use sha2::{Sha256, Digest};

pub async fn calculate_file_hash(file_path: &Path) -> Result<String, io::Error> {
    let mut file = File::open(file_path).await?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

// 发送前计算哈希
let file_hash = calculate_file_hash(file_path).await?;
send_with_hash(&file_path, &file_hash).await?;

// 接收后验证哈希
let received_hash = calculate_file_hash(received_file_path).await?;
if received_hash != original_hash {
    return Err(TransferError::ChecksumMismatch);
}
```

---

## 测试 4: 网络断开处理

### 测试目标
验证网络断开时应用的稳定性，以及网络恢复后的自动恢复。

### 测试步骤
1. 正常使用应用
2. 断开网络连接（WiFi/以太网）
3. 复制一段文本
4. 观察应用是否崩溃或卡顿
5. 重新连接网络
6. 验证应用恢复正常

### 预期结果
- ✓ 网络断开时应用不崩溃
- ✓ 应用可以继续本地使用
- ✓ 网络恢复后自动重连
- ✓ 同步功能恢复正常

### 可能的问题与修复

#### 问题 1: 网络断开时应用崩溃
**症状**: 断网后应用立即崩溃

**原因**: 未处理网络错误，导致 panic

**修复方案**:

```rust
// src-tauri/src/sync/mod.rs
// 添加错误处理和重试机制

pub async fn sync_with_retry(
    device: &Device,
    max_retries: u32,
    retry_delay: Duration
) -> Result<(), SyncError> {
    let mut retries = 0;

    loop {
        match sync_to_device(device).await {
            Ok(_) => return Ok(()),
            Err(e) if e.is_network_error() => {
                if retries >= max_retries {
                    return Err(e);
                }
                retries += 1;
                println!("Sync failed (attempt {}/{}), retrying...", retries, max_retries);
                tokio::time::sleep(retry_delay).await;
            }
            Err(e) => return Err(e),
        }
    }
}

impl SyncError {
    fn is_network_error(&self) -> bool {
        matches!(self, SyncError::ConnectionFailed(_) | SyncError::Timeout)
    }
}
```

#### 问题 2: 网络恢复后不自动重连
**症状**: 网络恢复后需要手动重启应用

**原因**: 没有监听网络状态变化

**修复方案**:

```rust
// src-tauri/src/network/mod.rs
// 监听网络状态变化

use notify::{RecommendedWatcher, RecursiveMode, Watcher, Event, EventKind};
use std::path::Path;

pub async fn watch_network_status() {
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);

    let mut watcher: RecommendedWatcher = Watcher::new(
        move |res: Result<Event, _>| {
            if let Ok(event) = res {
                if event.kind == EventKind::Any {
                    let _ = tx.blocking_send(event);
                }
            }
        },
        notify::Config::default(),
    ).unwrap();

    watcher.watch(Path::new("/etc/resolv.conf"), RecursiveMode::NonRecursive).unwrap();

    while let Some(event) = rx.recv().await {
        println!("Network status changed: {:?}", event);
        // 触发重连
        reconnect_devices().await;
    }
}
```

```typescript
// 前端显示网络状态提示
const networkStatus = ref<'online' | 'offline' | 'connecting'>('online')

function showNetworkStatus() {
  if (networkStatus.value === 'offline') {
    // 显示提示
    console.log('网络已断开，同步功能暂时不可用')
  }
}
```

---

## 测试 5: 快速连续同步回声

### 测试目标
验证快速连续复制时，是否存在回声问题（内容来回同步）。

### 测试步骤
1. 在 1 秒内快速复制 5 段文本
2. 观察同步日志
3. 检查是否有重复的同步事件
4. 计算发送/接收比率

### 预期结果
- ✓ 发送次数 ≈ 接收次数（比率接近 100%）
- ✓ 没有无限循环同步
- ✓ 没有重复内容

### 可能的问题与修复

#### 问题 1: 回声问题（无限循环）
**症状**: 接收次数远大于发送次数（>200%）

**原因**: A 同步到 B 后，B 更新剪贴板，又触发同步回 A

**修复方案**:

```rust
// src-tauri/src/clipboard/mod.rs
// 添加来源标记，避免回声

#[derive(Clone, Debug)]
pub struct ClipboardItem {
    pub id: String,
    pub content: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub source: ClipboardSource,  // 来源标记
}

#[derive(Clone, Debug)]
pub enum ClipboardSource {
    Local,       // 本地复制
    Remote(String),  // 远程同步，包含设备 ID
}

pub async fn handle_local_copy(content: String) {
    let item = ClipboardItem {
        id: Uuid::new_v4().to_string(),
        content,
        created_at: Utc::now(),
        source: ClipboardSource::Local,
    };

    save_to_db(&item).await;

    // 只同步本地复制的内容
    sync_to_trusted_devices(&item).await;
}

pub async fn handle_remote_copy(item: ClipboardItem) {
    // 远程同步的内容不再同步出去
    if matches!(item.source, ClipboardSource::Local) {
        item.source = ClipboardSource::Remote(item.source_device.clone());
        save_to_db(&item).await;
        update_local_clipboard(&item.content).await;
    }
    // 如果是远程来源，忽略（避免回声）
}
```

```rust
// 使用版本号或时间戳避免重复

pub async fn sync_to_trusted_devices(item: &ClipboardItem) {
    let trusted_devices = get_trusted_devices().await;

    for device in trusted_devices {
        // 检查设备是否已经收到此内容
        if !device_already_received(&device.device_id, &item.id).await {
            send_to_device(device, item).await;
            mark_as_received(&device.device_id, &item.id).await;
        }
    }
}
```

#### 问题 2: 并发同步导致的数据竞争
**症状**: 快速复制时数据不一致

**原因**: 多个同步请求同时执行

**修复方案**:

```rust
// src-tauri/src/sync/mod.rs
// 添加互斥锁保护同步操作

use tokio::sync::Mutex;

static SYNC_MUTEX: Mutex<()> = Mutex::const_new(());

pub async fn sync_to_trusted_devices(item: &ClipboardItem) {
    // 获取锁，确保串行同步
    let _guard = SYNC_MUTEX.lock().await;

    let trusted_devices = get_trusted_devices().await;

    for device in trusted_devices {
        send_to_device(&device, item).await?;
    }

    Ok(())
}
```

---

## 测试脚本使用

### 启动测试

```bash
chmod +x test-exceptional.sh
./test-exceptional.sh
```

### 测试脚本功能

1. **自动创建测试文件**：1MB、50MB、100MB、200MB
2. **指导手动测试**：每个测试都有详细的操作指引
3. **日志分析**：自动分析日志检测问题
4. **生成测试报告**：记录所有测试结果

---

## 性能优化建议

### 1. 剪贴板同步优化

```rust
// 使用内存数据库缓存最近同步内容
// 避免重复处理

use lru::LruCache;

let mut sync_cache = LruCache::new(100);

if sync_cache.contains(&content_hash) {
    return; // 已同步，跳过
}
```

### 2. 文件传输优化

```rust
// 使用并行传输提高大文件速度

use tokio::task::JoinSet;

let mut tasks = JoinSet::new();

for device in devices {
    tasks.spawn(transfer_file_to_device(device.clone(), file_path.clone()));
}

// 等待所有传输完成
while let Some(task) = tasks.join_next().await {
    task??
}
```

### 3. 网络优化

```rust
// 使用连接池复用 HTTP 连接

use reqwest::Client;

let http_client = Client::builder()
    .pool_max_idle_per_host(5)
    .timeout(Duration::from_secs(30))
    .build()?;
```

---

## 监控和日志

### 添加性能监控

```rust
// src-tauri/src/monitoring/mod.rs

pub struct PerformanceMetrics {
    pub sync_count: AtomicU64,
    pub avg_sync_time: AtomicU64,
    pub error_count: AtomicU64,
    pub memory_usage: AtomicU64,
}

impl PerformanceMetrics {
    pub async fn report(&self) -> MetricsReport {
        MetricsReport {
            sync_count: self.sync_count.load(Ordering::Relaxed),
            avg_sync_time: self.avg_sync_time.load(Ordering::Relaxed),
            error_count: self.error_count.load(Ordering::Relaxed),
            memory_usage: self.memory_usage.load(Ordering::Relaxed),
        }
    }
}
```

### 日志级别配置

```rust
// 开发环境使用 DEBUG
// 生产环境使用 INFO

let log_level = if cfg!(debug_assertions) {
    "debug"
} else {
    "info"
};

env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level))
    .init();
```

---

## 总结

通过以上测试和修复方案，可以确保应用在各种异常情况下都能稳定运行：

1. ✓ 设备离线/重连：自动检测并恢复连接
2. ✓ 多内容同步：队列机制确保不丢失不重复
3. ✓ 大文件传输：流式传输避免内存溢出
4. ✓ 网络断开：错误处理和自动重连
5. ✓ 快速同步：来源标记避免回声问题

建议在生产环境中持续监控这些指标，及时发现和解决问题。
