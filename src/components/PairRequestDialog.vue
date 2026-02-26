<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, UnlistenFn } from '@tauri-apps/api/event'

interface PairRequestData {
  device_id: string
  device_name: string
}

const visible = ref(false)
const currentRequest = ref<PairRequestData | null>(null)
const countdown = ref(10)
let unlisten: UnlistenFn | null = null
let countdownTimer: number | null = null

// 格式化设备 ID
function formatDeviceId(deviceId: string): string {
  if (deviceId.length <= 16) {
    return deviceId
  }
  return `${deviceId.substring(0, 8)}...${deviceId.substring(deviceId.length - 8)}`
}

// 显示配对请求
function showPairRequest(data: PairRequestData) {
  currentRequest.value = data
  visible.value = true
  countdown.value = 10
  startCountdown()
}

// 开始倒计时
function startCountdown() {
  if (countdownTimer) {
    clearInterval(countdownTimer)
  }

  countdownTimer = window.setInterval(() => {
    countdown.value--
    if (countdown.value <= 0) {
      stopCountdown()
      if (visible.value && currentRequest.value) {
        // 超时自动拒绝
        rejectPair(currentRequest.value.device_id)
      }
    }
  }, 1000)
}

// 停止倒计时
function stopCountdown() {
  if (countdownTimer) {
    clearInterval(countdownTimer)
    countdownTimer = null
  }
}

// 接受配对
async function acceptPair() {
  if (!currentRequest.value) return

  const { device_id, device_name } = currentRequest.value

  try {
    await invoke('accept_pair', { deviceId: device_id, deviceName: device_name })
    console.log('Pair accepted:', device_id)
    hide()
  } catch (error) {
    console.error('Failed to accept pair:', error)
  }
}

// 拒绝配对
async function rejectPair(deviceId?: string) {
  const id = deviceId || currentRequest.value?.device_id

  if (!id) return

  try {
    await invoke('reject_pair', { deviceId: id })
    console.log('Pair rejected:', id)
    hide()
  } catch (error) {
    console.error('Failed to reject pair:', error)
  }
}

// 隐藏弹窗
function hide() {
  stopCountdown()
  visible.value = false
  currentRequest.value = null
}

// 监听配对请求事件
async function setupEventListener() {
  try {
    unlisten = await listen<PairRequestData>('pair-request-received', (event) => {
      console.log('Pair request received:', event.payload)
      showPairRequest(event.payload)
    })
  } catch (error) {
    console.error('Failed to setup event listener:', error)
  }
}

onMounted(async () => {
  await setupEventListener()
})

onUnmounted(() => {
  stopCountdown()
  if (unlisten) {
    unlisten()
  }
})
</script>

<template>
  <Transition name="fade">
    <div v-if="visible" class="overlay">
      <div class="dialog">
        <!-- 对话框头部 -->
        <div class="dialog-header">
          <div class="icon-wrapper">
            <svg class="icon" width="32" height="32" viewBox="0 0 32 32" fill="none">
              <circle cx="16" cy="16" r="14" fill="url(#gradient)" />
              <path d="M16 9v14M9 16h14" stroke="white" stroke-width="3" stroke-linecap="round"/>
              <defs>
                <linearGradient id="gradient" x1="0%" y1="0%" x2="100%" y2="100%">
                  <stop offset="0%" style="stop-color:#007AFF"/>
                  <stop offset="100%" style="stop-color:#5856D6"/>
                </linearGradient>
              </defs>
            </svg>
          </div>
          <h3 class="dialog-title">配对请求</h3>
        </div>

        <!-- 对话框内容 -->
        <div class="dialog-content">
          <div v-if="currentRequest" class="request-info">
            <div class="message">
              设备 <strong>{{ currentRequest.device_name }}</strong> 请求与您配对
            </div>
            <div class="device-id">
              <span class="label">设备 ID:</span>
              <span class="id">{{ formatDeviceId(currentRequest.device_id) }}</span>
            </div>
          </div>

          <!-- 倒计时提示 -->
          <div class="countdown-hint">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5">
              <circle cx="7" cy="7" r="6"/>
              <path d="M7 4v3l2 2"/>
            </svg>
            <span>{{ countdown }} 秒后自动拒绝</span>
          </div>
        </div>

        <!-- 对话框按钮 -->
        <div class="dialog-footer">
          <button @click="rejectPair()" class="btn btn-reject">
            拒绝
          </button>
          <button @click="acceptPair()" class="btn btn-accept">
            接受
          </button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(10px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
}

.dialog {
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(20px);
  border-radius: 16px;
  padding: 0;
  min-width: 360px;
  max-width: 480px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  animation: slideUp 0.3s ease;
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(20px) scale(0.95);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

.dialog-header {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 24px 24px 16px;
  border-bottom: 1px solid rgba(0, 0, 0, 0.08);
}

.icon-wrapper {
  margin-bottom: 12px;
}

.icon {
  filter: drop-shadow(0 2px 8px rgba(0, 122, 255, 0.3));
}

.dialog-title {
  font-size: 18px;
  font-weight: 600;
  color: #1c1c1e;
  margin: 0;
}

.dialog-content {
  padding: 20px 24px;
}

.request-info {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.message {
  font-size: 15px;
  color: #1c1c1e;
  text-align: center;
  line-height: 1.5;
}

.message strong {
  font-weight: 600;
  color: #007aff;
}

.device-id {
  display: flex;
  align-items: center;
  gap: 8px;
  background: rgba(0, 0, 0, 0.05);
  padding: 10px 14px;
  border-radius: 8px;
}

.label {
  font-size: 12px;
  color: #86868b;
  font-weight: 500;
  flex-shrink: 0;
}

.id {
  font-size: 13px;
  color: #1c1c1e;
  font-family: 'SF Mono', Monaco, 'Courier New', monospace;
  font-weight: 500;
}

.countdown-hint {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  margin-top: 16px;
  padding: 8px;
  background: rgba(255, 59, 48, 0.1);
  border-radius: 6px;
  color: #ff3b30;
  font-size: 13px;
  font-weight: 500;
}

.countdown-hint svg {
  flex-shrink: 0;
}

.dialog-footer {
  display: flex;
  gap: 12px;
  padding: 16px 24px 24px;
}

.btn {
  flex: 1;
  padding: 12px 20px;
  border-radius: 10px;
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
  border: none;
  outline: none;
  -webkit-app-region: no-drag;
}

.btn:hover {
  transform: translateY(-1px);
}

.btn:active {
  transform: translateY(0);
}

.btn-reject {
  background: rgba(0, 0, 0, 0.08);
  color: #1c1c1e;
}

.btn-reject:hover {
  background: rgba(0, 0, 0, 0.12);
}

.btn-accept {
  background: linear-gradient(135deg, #007aff 0%, #5856d6 100%);
  color: white;
  box-shadow: 0 4px 12px rgba(0, 122, 255, 0.3);
}

.btn-accept:hover {
  background: linear-gradient(135deg, #0062cc 0%, #4a48c5 100%);
  box-shadow: 0 6px 16px rgba(0, 122, 255, 0.4);
}

/* Transition animations */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
