<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { listen, UnlistenFn } from '@tauri-apps/api/event'

type SyncStatus = 'server_off' | 'server_on' | 'connected' | 'syncing'

interface ClipboardSyncedEvent {
  device_id: string
  received_count: number
  errors: string[]
}

const status = ref<SyncStatus>('server_off')
const connectedDeviceCount = ref(0)
const lastSyncTime = ref<Date | null>(null)
const lastSyncDevice = ref<string | null>(null)
const isAnimating = ref(false)
let unlisten: UnlistenFn | null = null
let animationTimeout: number | null = null

// 状态文本
const statusText = computed(() => {
  switch (status.value) {
    case 'server_off':
      return '服务未开启'
    case 'server_on':
      return '服务已开启'
    case 'connected':
      return `已连接 ${connectedDeviceCount.value} 台设备`
    case 'syncing':
      return '同步中...'
  }
})

// 状态颜色
const statusColor = computed(() => {
  switch (status.value) {
    case 'server_off':
      return '#8e8e93'
    case 'server_on':
      return '#ff9500'
    case 'connected':
      return '#34c759'
    case 'syncing':
      return '#007aff'
  }
})

// 背景颜色
const bgColor = computed(() => {
  switch (status.value) {
    case 'server_off':
      return 'rgba(142, 142, 147, 0.1)'
    case 'server_on':
      return 'rgba(255, 149, 0, 0.1)'
    case 'connected':
      return 'rgba(52, 199, 89, 0.1)'
    case 'syncing':
      return 'rgba(0, 122, 255, 0.1)'
  }
})

// 格式化时间
function formatTime(date: Date | null): string {
  if (!date) return '从未'

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

// 触发同步动画
function triggerSyncAnimation() {
  status.value = 'syncing'
  isAnimating.value = true

  if (animationTimeout) {
    clearTimeout(animationTimeout)
  }

  animationTimeout = window.setTimeout(() => {
    isAnimating.value = false
    status.value = connectedDeviceCount.value > 0 ? 'connected' : 'server_on'
  }, 2000)
}

// 更新连接设备数和服务器状态
function updateConnectedCount(count: number, isServerRunning: boolean = true) {
  connectedDeviceCount.value = count

  if (!isServerRunning) {
    status.value = 'server_off'
  } else if (count > 0) {
    status.value = 'connected'
  } else {
    status.value = 'server_on'
  }
}

// 监听剪贴板同步事件
async function setupEventListener() {
  try {
    unlisten = await listen<ClipboardSyncedEvent>('clipboard-synced', (event) => {
      console.log('Clipboard synced:', event.payload)

      const { device_id } = event.payload

      // 更新最后同步信息
      lastSyncTime.value = new Date()
      lastSyncDevice.value = device_id

      // 触发同步动画
      triggerSyncAnimation()
    })
  } catch (error) {
    console.error('Failed to setup event listener:', error)
  }
}

onMounted(async () => {
  await setupEventListener()
})

onUnmounted(() => {
  if (unlisten) {
    unlisten()
  }
  if (animationTimeout) {
    clearTimeout(animationTimeout)
  }
})

// 暴露方法给父组件
defineExpose({
  updateConnectedCount,
  triggerSyncAnimation
})
</script>

<template>
  <div class="sync-indicator">
    <!-- 状态指示器 -->
    <div class="status-badge" :style="{ backgroundColor: bgColor }">
      <div
        class="status-dot"
        :class="{
          'server-off': status === 'server_off',
          'server-on': status === 'server_on',
          'connected': status === 'connected',
          'syncing': isAnimating
        }"
      >
        <svg v-if="isAnimating" class="spin" width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M6 2v4M8.5 4.5L6 2 3.5 4.5"/>
        </svg>
      </div>
      <span class="status-text" :style="{ color: statusColor }">
        {{ statusText }}
      </span>
    </div>

    <!-- 最近同步时间 -->
    <div v-if="lastSyncTime" class="sync-time">
      <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.5">
        <circle cx="6" cy="6" r="5"/>
        <path d="M6 3v3l2 2"/>
      </svg>
      <span>
        {{ lastSyncDevice ? `来自 ${lastSyncDevice}` : '' }}
        {{ formatTime(lastSyncTime) }}
      </span>
    </div>
  </div>
</template>

<style scoped>
.sync-indicator {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 4px;
}

.status-badge {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  border-radius: 20px;
  transition: all 0.3s ease;
}

.status-dot {
  width: 14px;
  height: 14px;
  border-radius: 50%;
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 8px;
  transition: all 0.3s ease;
}

.status-dot.server-off {
  background: #8e8e93;
}

.status-dot.server-on {
  background: #ff9500;
  box-shadow: 0 0 0 3px rgba(255, 149, 0, 0.2);
}

.status-dot.connected {
  background: #34c759;
  box-shadow: 0 0 0 3px rgba(52, 199, 89, 0.2);
}

.status-dot.syncing {
  background: #007aff;
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% {
    box-shadow: 0 0 0 0 rgba(0, 122, 255, 0.4);
  }
  50% {
    box-shadow: 0 0 0 8px rgba(0, 122, 255, 0);
  }
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.status-text {
  font-size: 13px;
  font-weight: 500;
  transition: color 0.3s ease;
}

.sync-time {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: #8e8e93;
  padding-right: 0;
}

.sync-time svg {
  flex-shrink: 0;
  opacity: 0.6;
}

.sync-time span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
