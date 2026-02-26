# 修复说明：未连接状态问题

## 问题描述

用户反馈"未连接状态"显示不正确，可能存在以下问题：

1. **即使设备已配对且在线，仍显示"未连接"**
2. **设备离线后，状态没有及时更新**
3. **设备重新上线后，连接状态没有自动恢复**

## 问题根因分析

### 1. 连接状态计算不准确

**原代码：**
```javascript
// App.vue
async function updateConnectedCount() {
  try {
    const trustedDevices = await invoke<any[]>('get_trusted_devices')
    if (syncIndicator.value) {
      syncIndicator.value.updateConnectedCount(trustedDevices.length)
    }
  } catch (error) {
    console.error('Failed to update connected count:', error)
  }
}
```

**问题：**
- 只统计信任设备总数，不检查设备是否在线
- 即使设备离线，只要在信任列表中，就显示"已连接"
- 没有基于 `last_seen` 时间判断设备是否真正在线

### 2. DevicePanel 配对状态判断错误

**原代码：**
```typescript
// DevicePanel.vue
function getPairingStatus(deviceId: string): PairingStatus {
  const trusted = trustedDevices.value.find(d => d.device_id === deviceId)
  // ...
}

interface NetworkDevice {
  name: string
  hostname: string
  ip: string
  port: number
  last_seen: string
  // 缺少 device_id
}
```

**问题：**
- `NetworkDevice` 接口缺少 `device_id` 字段
- `getPairingStatus()` 接收 `deviceId` 参数，但在模板中传递的是 `hostname`
- 导致配对状态判断不正确

### 3. 缺少设备变化通知

**原代码：**
```typescript
// DevicePanel.vue
unlistenDiscovered = await listen<NetworkDevice>('device-discovered', (event) => {
  const device = event.payload
  const existingIndex = devices.value.findIndex(d => d.hostname === device.hostname)
  if (existingIndex >= 0) {
    devices.value[existingIndex] = device
  } else {
    devices.value.push(device)
  }
  // 缺少通知父组件更新连接状态
})
```

**问题：**
- 设备发现/移除时没有通知父组件
- 连接状态不会实时更新
- 需要手动刷新或重启应用才能看到状态变化

## 修复方案

### 修复 1：改进连接状态计算，基于设备在线状态

**修改后：**
```javascript
// App.vue
async function updateConnectedCount() {
  try {
    const trustedDevices = await invoke<any[]>('get_trusted_devices')

    // 检查在线状态（最近30秒内活跃视为在线）
    const now = Date.now()
    const onlineThreshold = 30000 // 30秒
    const onlineCount = trustedDevices.filter((device: any) => {
      if (!device.last_seen) return false
      const lastSeen = new Date(device.last_seen).getTime()
      return (now - lastSeen) < onlineThreshold
    }).length

    if (syncIndicator.value) {
      syncIndicator.value.updateConnectedCount(onlineCount)
    }
  } catch (error) {
    console.error('Failed to update connected count:', error)
  }
}
```

**改进：**
- 添加了在线状态检查（30秒阈值）
- 只统计真正在线的信任设备
- 离线设备不计入连接数

### 修复 2：添加设备 ID 字段

**修改后：**
```typescript
// DevicePanel.vue
interface NetworkDevice {
  device_id: string  // 添加此字段
  name: string
  hostname: string
  ip: string
  port: number
  last_seen: string
}

function getPairingStatus(device: NetworkDevice): PairingStatus {
  const trusted = trustedDevices.value.find(d => d.device_id === device.device_id)
  // ...
}
```

**改进：**
- 添加了 `device_id` 字段到接口
- `getPairingStatus()` 现在接收完整的 `device` 对象
- 配对状态判断基于 `device_id` 而不是 `hostname`

### 修复 3：添加设备变化通知机制

**修改后：**
```typescript
// DevicePanel.vue
// 定义 emits
const emit = defineEmits<{
  devicesChanged: [count: number]
}>()

unlistenDiscovered = await listen<NetworkDevice>('device-discovered', (event) => {
  const device = event.payload
  const existingIndex = devices.value.findIndex(d => d.device_id === device.device_id)
  if (existingIndex >= 0) {
    devices.value[existingIndex] = device
  } else {
    devices.value.push(device)
  }
  // 通知父组件更新连接状态
  emit('devices-changed', devices.value.length)
})
```

**App.vue:**
```javascript
// 处理设备数量变化
function handleDevicesChanged(count: number) {
  console.log('Devices changed:', count)
  updateConnectedCount()
}

// 模板中添加监听
<DevicePanel @devices-changed="handleDevicesChanged" />
```

**改进：**
- 设备发现/移除时自动触发连接状态更新
- 实时反映设备在线状态
- 无需手动刷新

### 修复 4：更新模板中的设备 key 和事件处理

**修改后：**
```vue
<!-- DevicePanel.vue -->
<div
  v-for="device in devices"
  :key="device.device_id"  <!-- 使用 device_id 而不是 hostname -->
  class="device-card"
>
  <button @click="requestPair(device.device_id)" class="btn btn-primary">
    请求配对
  </button>
  <span class="device-status" :class="getPairingStatus(device)">
    {{ getPairingStatus(device) }}  <!-- 传递整个 device 对象 -->
  </span>
</div>
```

**改进：**
- 使用 `device_id` 作为 key（更稳定的标识符）
- 传递完整的 `device` 对象到 `getPairingStatus()`
- 所有事件处理使用 `device_id`

## 修复后的行为

### 1. 正常连接状态
- 设备配对且在线（最近30秒活跃）：显示"已连接 N 台设备"
- 设备配对但离线：显示"未连接"
- 没有配对设备：显示"未连接"

### 2. 实时状态更新
- 设备上线：立即更新连接状态
- 设备离线：30秒后自动更新为未连接
- 配对/取消配对：立即更新连接状态

### 3. 配对流程
- 发送配对请求：状态变为"配对中..."
- 接受配对：状态变为"已配对"，连接数+1
- 拒绝配对：状态变为"已拒绝"
- 取消配对：移除信任设备，连接数-1

## 测试验证

### 测试步骤 1：设备连接状态
1. 启动两个应用实例
2. 配对两个设备
3. 观察连接状态显示"已连接 1 台设备"
4. 关闭一个实例
5. 等待 30 秒
6. 观察连接状态变为"未连接"
7. 重新启动实例
8. 观察连接状态恢复"已连接 1 台设备"

### 测试步骤 2：多设备场景
1. 启动三个应用实例
2. 实例 A 与 B、C 分别配对
3. 观察连接状态显示"已连接 2 台设备"
4. 关闭实例 B
5. 观察连接状态显示"已连接 1 台设备"
6. 关闭实例 C
7. 观察连接状态显示"未连接"

### 测试步骤 3：配对流程
1. 实例 A 发送配对请求到实例 B
2. 状态显示"配对中..."
3. 实例 B 接受配对
4. 状态变为"已配对"
5. 观察连接状态更新

## 注意事项

1. **在线阈值**：当前设置为 30 秒，可根据实际情况调整
2. **设备 ID**：后端需要在 `device-discovered` 事件中包含 `device_id` 字段
3. **last_seen**：后端需要正确维护和更新设备的 `last_seen` 时间戳

## 后续优化建议

1. **添加心跳机制**：定期检查设备在线状态
2. **显示在线/离线详情**：在设备列表中明确标识在线状态
3. **优化更新频率**：避免频繁更新导致性能问题
4. **添加重连机制**：设备离线后自动尝试重连
