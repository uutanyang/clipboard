<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, UnlistenFn } from '@tauri-apps/api/event'

// 定义 emits
const emit = defineEmits<{
  'devices-changed': [count: number]
}>()

interface NetworkDevice {
  device_id: string
  name: string
  hostname: string
  ip: string
  port: number
  last_seen: string
}

interface TrustedDevice {
  device_id: string
  device_name: string
  paired_at: string
  last_seen: string
}

interface PairingState {
  device_id: string
  device_name: string
  status: 'pending' | 'accepted' | 'rejected' | 'failed' | 'cancelled'
  created_at: string
}

type PairingStatus = '未配对' | '配对中...' | '已配对' | '已拒绝' | '配对失败'

const devices = ref<NetworkDevice[]>([])
const trustedDevices = ref<TrustedDevice[]>([])
const pairings = ref<Map<string, PairingState>>(new Map())
let unlistenDiscovered: UnlistenFn | null = null
let unlistenRemoved: UnlistenFn | null = null
let unlistenPairRequested: UnlistenFn | null = null
let unlistenPairAccepted: UnlistenFn | null = null
let unlistenPairRejected: UnlistenFn | null = null
let unlistenPairCancelled: UnlistenFn | null = null

// 获取配对状态
function getPairingStatus(device: NetworkDevice): PairingStatus {
  // 检查是否在信任设备列表中
  const trusted = trustedDevices.value.find(d => d.device_id === device.device_id)
  if (trusted) {
    return '已配对'
  }

  // 检查配对状态
  const pairing = pairings.value.get(device.device_id)
  if (!pairing) {
    return '未配对'
  }

  switch (pairing.status) {
    case 'pending':
      return '配对中...'
    case 'accepted':
      return '已配对'
    case 'rejected':
      return '已拒绝'
    case 'failed':
      return '配对失败'
    case 'cancelled':
      return '已拒绝'
    default:
      return '未配对'
  }
}

// 获取在线状态
function getOnlineStatus(lastSeen: string): boolean {
  const diff = Date.now() - new Date(lastSeen).getTime()
  return diff < 30000 // 30秒内视为在线
}

// 加载信任设备列表
async function loadTrustedDevices() {
  try {
    const result = await invoke<TrustedDevice[]>('get_trusted_devices')
    trustedDevices.value = result
  } catch (error) {
    console.error('Failed to load trusted devices:', error)
  }
}

// 加载配对状态
async function loadPairings() {
  try {
    const result = await invoke<Array<PairingState>>('get_all_pairings')
    pairings.value = new Map(result.map(p => [p.device_id, p]))
  } catch (error) {
    console.error('Failed to load pairings:', error)
  }
}

// 请求配对
async function requestPair(deviceId: string) {
  try {
    await invoke('request_pair', { targetDeviceId: deviceId })
    console.log('Pair request sent:', deviceId)
  } catch (error) {
    console.error('Failed to request pair:', error)
  }
}

// 取消配对
async function cancelPair(deviceId: string) {
  try {
    await invoke('cancel_pair', { targetDeviceId: deviceId })
    console.log('Pair cancelled:', deviceId)
  } catch (error) {
    console.error('Failed to cancel pair:', error)
  }
}

// 移除信任设备
async function removeTrustedDevice(deviceId: string) {
  if (!confirm('确定要移除此设备吗？')) {
    return
  }
  try {
    await invoke('remove_trusted_device', { deviceId })
    console.log('Device removed:', deviceId)
    await loadTrustedDevices()
  } catch (error) {
    console.error('Failed to remove trusted device:', error)
  }
}

// 格式化时间
function formatTime(dateString: string) {
  const date = new Date(dateString)
  const now = new Date()
  const diff = now.getTime() - date.getTime()

  if (diff < 60000) {
    return '刚刚'
  } else if (diff < 3600000) {
    return `${Math.floor(diff / 60000)} 分钟前`
  } else if (diff < 86400000) {
    return `${Math.floor(diff / 3600000)} 小时前`
  } else {
    return date.toLocaleDateString('zh-CN')
  }
}

// 监听设备发现事件
async function setupEventListeners() {
  try {
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

    unlistenRemoved = await listen<string>('device-removed', (event) => {
      const deviceId = event.payload
      devices.value = devices.value.filter(d => d.device_id !== deviceId)
      // 通知父组件更新连接状态
      emit('devices-changed', devices.value.length)
    })

    unlistenPairRequested = await listen<any>('pair-request-sent', (event) => {
      const pairing = event.payload
      pairings.value.set(pairing.device_id, pairing)
    })

    unlistenPairAccepted = await listen<any>('pair-accepted', (event) => {
      const { device_id, device_name } = event.payload
      pairings.value.set(device_id, {
        device_id,
        device_name,
        status: 'accepted',
        created_at: new Date().toISOString()
      })
      loadTrustedDevices()
    })

    unlistenPairRejected = await listen<string>('pair-rejected', (event) => {
      const deviceId = event.payload
      const pairing = pairings.value.get(deviceId)
      if (pairing) {
        pairings.value.set(deviceId, {
          ...pairing,
          status: 'rejected'
        })
      }
    })

    unlistenPairCancelled = await listen<string>('pair-cancelled', (event) => {
      const deviceId = event.payload
      pairings.value.delete(deviceId)
    })
  } catch (error) {
    console.error('Failed to setup event listeners:', error)
  }
}

onMounted(async () => {
  await loadTrustedDevices()
  await loadPairings()
  await setupEventListeners()
})

onUnmounted(() => {
  if (unlistenDiscovered) unlistenDiscovered()
  if (unlistenRemoved) unlistenRemoved()
  if (unlistenPairRequested) unlistenPairRequested()
  if (unlistenPairAccepted) unlistenPairAccepted()
  if (unlistenPairRejected) unlistenPairRejected()
  if (unlistenPairCancelled) unlistenPairCancelled()
})
</script>

<template>
  <div class="device-panel">
    <div class="panel-header">
      <h2 class="panel-title">设备列表</h2>
      <span class="device-count">{{ devices.length }} 台设备</span>
    </div>

    <div class="device-list">
      <div v-if="devices.length === 0" class="empty">
        <div class="empty-icon">📡</div>
        <div class="empty-text">未发现设备</div>
        <div class="empty-hint">确保设备在同一局域网内</div>
      </div>

      <div
        v-for="device in devices"
        :key="device.device_id"
        class="device-card"
      >
        <div class="device-main">
          <div class="device-info">
            <div class="device-name-line">
              <span class="device-name">{{ device.name }}</span>
              <div
                class="status-dot"
                :class="{ online: getOnlineStatus(device.last_seen) }"
                :title="getOnlineStatus(device.last_seen) ? '在线' : '离线'"
              />
            </div>
            <div class="device-details">
              <span class="device-ip">{{ device.ip }}</span>
              <span class="device-status" :class="getPairingStatus(device)">
                {{ getPairingStatus(device) }}
              </span>
            </div>
            <div class="device-last-seen">
              最后活跃: {{ formatTime(device.last_seen) }}
            </div>
          </div>

          <div class="device-actions">
            <template v-if="getPairingStatus(device) === '未配对'">
              <button @click="requestPair(device.device_id)" class="btn btn-primary">
                请求配对
              </button>
            </template>
            <template v-else-if="getPairingStatus(device) === '配对中...'">
              <button @click="cancelPair(device.device_id)" class="btn btn-secondary">
                取消配对
              </button>
            </template>
            <template v-else-if="getPairingStatus(device) === '已配对'">
              <button @click="removeTrustedDevice(device.device_id)" class="btn btn-danger">
                取消配对
              </button>
            </template>
            <template v-else-if="getPairingStatus(device) === '已拒绝' || getPairingStatus(device) === '配对失败'">
              <button @click="requestPair(device.device_id)" class="btn btn-secondary">
                重新配对
              </button>
            </template>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.device-panel {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: rgba(242, 242, 247, 0.9);
  backdrop-filter: blur(20px) saturate(180%);
  font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Display', 'Segoe UI', sans-serif;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px;
  background: rgba(255, 255, 255, 0.8);
  border-bottom: 1px solid rgba(0, 0, 0, 0.08);
}

.panel-title {
  font-size: 18px;
  font-weight: 600;
  color: #1c1c1e;
  margin: 0;
}

.device-count {
  font-size: 14px;
  color: #86868b;
  font-weight: 500;
}

.device-list {
  flex: 1;
  overflow-y: auto;
  padding: 16px 20px 20px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.device-list::-webkit-scrollbar {
  width: 8px;
}

.device-list::-webkit-scrollbar-track {
  background: transparent;
}

.device-list::-webkit-scrollbar-thumb {
  background: rgba(0, 0, 0, 0.2);
  border-radius: 4px;
}

.device-list::-webkit-scrollbar-thumb:hover {
  background: rgba(0, 0, 0, 0.3);
}

.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 300px;
  gap: 12px;
}

.empty-icon {
  font-size: 48px;
  opacity: 0.5;
}

.empty-text {
  font-size: 16px;
  color: #86868b;
  font-weight: 500;
}

.empty-hint {
  font-size: 14px;
  color: #aeaeb2;
  font-weight: 400;
}

.device-card {
  background: rgba(255, 255, 255, 0.7);
  border-radius: 12px;
  padding: 16px;
  transition: all 0.2s ease;
  border: 1px solid transparent;
}

.device-card:hover {
  background: rgba(255, 255, 255, 0.95);
  transform: scale(1.01);
}

.device-main {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.device-info {
  flex: 1;
  min-width: 0;
}

.device-name-line {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.device-name {
  font-size: 15px;
  font-weight: 600;
  color: #1c1c1e;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #c7c7cc;
  flex-shrink: 0;
  transition: background-color 0.2s ease;
}

.status-dot.online {
  background: #34c759;
  box-shadow: 0 0 0 3px rgba(52, 199, 89, 0.2);
}

.device-details {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 4px;
}

.device-ip {
  font-size: 13px;
  color: #86868b;
  font-weight: 400;
  font-family: 'SF Mono', Monaco, 'Courier New', monospace;
}

.device-status {
  font-size: 12px;
  font-weight: 600;
  padding: 4px 10px;
  border-radius: 12px;
}

.device-status.未配对 {
  background: rgba(142, 142, 147, 0.15);
  color: #8e8e93;
}

.device-status.配对中 {
  background: rgba(0, 122, 255, 0.15);
  color: #007aff;
}

.device-status.已配对 {
  background: rgba(52, 199, 89, 0.15);
  color: #34c759;
}

.device-status.已拒绝,
.device-status.配对失败 {
  background: rgba(255, 59, 48, 0.15);
  color: #ff3b30;
}

.device-last-seen {
  font-size: 12px;
  color: #aeaeb2;
  font-weight: 400;
}

.device-actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

.btn {
  padding: 8px 16px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
  border: none;
  white-space: nowrap;
  -webkit-app-region: no-drag;
}

.btn-primary {
  background: #007aff;
  color: white;
}

.btn-primary:hover {
  background: #0062cc;
}

.btn-secondary {
  background: rgba(0, 0, 0, 0.08);
  color: #1c1c1e;
}

.btn-secondary:hover {
  background: rgba(0, 0, 0, 0.12);
}

.btn-danger {
  background: #ff3b30;
  color: white;
}

.btn-danger:hover {
  background: #d70015;
}
</style>
