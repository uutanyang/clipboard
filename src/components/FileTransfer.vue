<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { listen, UnlistenFn } from '@tauri-apps/api/event'

interface NetworkDevice {
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

interface ReceivedFile {
  file_name: string
  file_size: number
  file_path: string
  source_device: string
  timestamp: number
}

interface TransferProgress {
  device_id: string
  file_name: string
  progress: number
  total: number
  status: 'uploading' | 'completed' | 'error'
  error?: string
}

const devices = ref<NetworkDevice[]>([])
const trustedDevices = ref<TrustedDevice[]>([])
const receivedFiles = ref<ReceivedFile[]>([])
const transfers = ref<Map<string, TransferProgress>>(new Map())
const showDeviceSelector = ref(false)
const selectedFile = ref<string | null>(null)
let unlisten: UnlistenFn | null = null

// 获取在线状态
function getOnlineStatus(lastSeen: string): boolean {
  const diff = Date.now() - new Date(lastSeen).getTime()
  return diff < 30000 // 30秒内视为在线
}

// 加载已发现的设备
async function loadDevices() {
  try {
    const result = await invoke<NetworkDevice[]>('get_discovered_devices')
    devices.value = result
  } catch (error) {
    console.error('Failed to load devices:', error)
  }
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

// 格式化文件大小
function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`
}

// 格式化时间
function formatTime(timestamp: number): string {
  const date = new Date(timestamp)
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

// 选择文件
async function selectFile() {
  try {
    const selected = await openDialog({
      multiple: false,
      directory: false,
      title: '选择要发送的文件'
    })

    if (selected && typeof selected === 'string') {
      console.log('Selected file:', selected)
      selectedFile.value = selected

      // 重新加载设备列表
      await loadDevices()
      await loadTrustedDevices()

      // 检查是否有可用设备
      if (availableDevices.value.length === 0) {
        alert('没有可用的设备。请确保目标设备已启动、在线且已完成配对。')
        return
      }

      showDeviceSelector.value = true
    }
  } catch (error) {
    console.error('Failed to open file dialog:', error)
    alert('打开文件选择器失败')
  }
}

// 发送文件到指定设备
async function sendFileToDevice(deviceId: string) {
  if (!selectedFile.value) return

  const device = devices.value.find(d => d.hostname === deviceId)
  if (!device) {
    console.error('Device not found:', deviceId)
    alert('未找到目标设备')
    return
  }

  const fileName = selectedFile.value.split(/[/\\]/).pop() || 'unknown'
  const transferId = `${deviceId}-${Date.now()}`

  // 添加传输进度
  transfers.value.set(transferId, {
    device_id: deviceId,
    file_name: fileName,
    progress: 0,
    total: 0,
    status: 'uploading'
  })

  try {
    console.log('Sending file to device:', deviceId)
    console.log('File path:', selectedFile.value)

    // 调用 Tauri 命令发送文件
    const result = await invoke<any>('send_file_to_device', {
      deviceId,
      filePath: selectedFile.value
    })

    console.log('File sent successfully:', result)

    // 更新传输状态为完成
    const transfer = transfers.value.get(transferId)
    if (transfer) {
      transfers.value.set(transferId, {
        ...transfer,
        progress: result.file_size || 0,
        total: result.file_size || 0,
        status: 'completed'
      })
    }

    // 3秒后移除传输记录
    setTimeout(() => {
      transfers.value.delete(transferId)
    }, 3000)

    // 关闭设备选择器
    showDeviceSelector.value = false
    selectedFile.value = null
  } catch (error) {
    console.error('Failed to send file:', error)

    const errorMessage = String(error)
    alert(`文件传输失败：${errorMessage}`)

    const transfer = transfers.value.get(transferId)
    if (transfer) {
      transfers.value.set(transferId, {
        ...transfer,
        status: 'error',
        error: errorMessage
      })
    }
  }
}

// 打开文件所在目录
async function openFileDirectory(filePath: string) {
  try {
    // 提取目录路径
    const match = /[/\\]/.exec(filePath)
    const separator = match ? match[0] : ''
    const directory = filePath.substring(0, filePath.lastIndexOf(separator))
    // 使用 Tauri 的 shell API
    await invoke('plugin:shell|open', { path: directory })
  } catch (error) {
    console.error('Failed to open file directory:', error)
  }
}

// 获取传输进度百分比
function getProgressPercentage(transfer: TransferProgress): number {
  if (transfer.total === 0) return 0
  return Math.round((transfer.progress / transfer.total) * 100)
}

// 获取可用的目标设备（在线且已配对）
const availableDevices = computed(() => {
  return devices.value.filter(device => {
    const isOnline = getOnlineStatus(device.last_seen)
    const isPaired = trustedDevices.value.some(d => d.device_id === device.hostname)
    return isOnline && isPaired
  })
})

// 清空接收记录
function clearReceivedFiles() {
  if (confirm('确定要清空所有接收文件记录吗？')) {
    receivedFiles.value = []
  }
}

// 监听文件接收事件
async function setupEventListener() {
  try {
    unlisten = await listen<ReceivedFile>('file-received', (event) => {
      console.log('File received:', event.payload)
      receivedFiles.value.unshift(event.payload)

      // 限制列表数量
      if (receivedFiles.value.length > 50) {
        receivedFiles.value = receivedFiles.value.slice(0, 50)
      }
    })
  } catch (error) {
    console.error('Failed to setup event listener:', error)
  }
}

onMounted(async () => {
  await loadDevices()
  await loadTrustedDevices()
  await setupEventListener()
})

onUnmounted(() => {
  if (unlisten) {
    unlisten()
  }
})
</script>

<template>
  <div class="file-transfer">
    <div class="panel-header">
      <h2 class="panel-title">文件传输</h2>
      <button @click="selectFile" class="btn-send">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M3 8h10M10 5l3 3-3 3"/>
        </svg>
        <span>发送文件</span>
      </button>
    </div>

    <!-- 传输进度 -->
    <div v-if="transfers.size > 0" class="transfers-section">
      <div class="section-title">传输中</div>
      <div
        v-for="([id, transfer]) in transfers"
        :key="id"
        class="transfer-item"
      >
        <div class="transfer-info">
          <div class="transfer-file">{{ transfer.file_name }}</div>
          <div
            class="transfer-status"
            :class="transfer.status"
          >
            <template v-if="transfer.status === 'uploading'">
              <svg class="spin" width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M7 2v5M9.5 5.5L7 3 4.5 5.5"/>
              </svg>
              <span>上传中...</span>
            </template>
            <template v-else-if="transfer.status === 'completed'">
              <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M3 7l3 3 5-5"/>
              </svg>
              <span>传输完成</span>
            </template>
            <template v-else-if="transfer.status === 'error'">
              <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M3 3l8 8M11 3l-8 8"/>
              </svg>
              <span>传输失败: {{ transfer.error }}</span>
            </template>
          </div>
        </div>
        <div
          v-if="transfer.status === 'uploading'"
          class="progress-bar"
        >
          <div
            class="progress-fill"
            :style="{ width: getProgressPercentage(transfer) + '%' }"
          />
        </div>
        <div v-if="transfer.status === 'uploading'" class="progress-text">
          {{ formatFileSize(transfer.progress) }} / {{ formatFileSize(transfer.total) }}
        </div>
      </div>
    </div>

    <!-- 接收的文件列表 -->
    <div class="files-section">
      <div class="section-header">
        <div class="section-title">接收的文件</div>
        <button
          v-if="receivedFiles.length > 0"
          @click="clearReceivedFiles"
          class="btn-clear"
          title="清空所有记录"
        >
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M2.5 4h9M3.5 8h7M4.5 12h5"/>
          </svg>
        </button>
      </div>

      <div v-if="receivedFiles.length === 0" class="empty">
        <div class="empty-icon">📁</div>
        <div class="empty-text">暂无接收的文件</div>
      </div>

      <div v-else class="file-list">
        <div
          v-for="file in receivedFiles"
          :key="file.file_path"
          class="file-item"
        >
          <div class="file-main">
            <div class="file-icon">📄</div>
            <div class="file-info">
              <div class="file-name">{{ file.file_name }}</div>
              <div class="file-meta">
                <span class="file-size">{{ formatFileSize(file.file_size) }}</span>
                <span class="file-source">来自 {{ file.source_device }}</span>
                <span class="file-time">{{ formatTime(file.timestamp) }}</span>
              </div>
            </div>
          </div>
          <button
            @click="openFileDirectory(file.file_path)"
            class="btn-open"
            title="打开文件所在目录"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M3 7v6a2 2 0 002 2h6a2 2 0 002-2V9M3 7h10M3 7l4-4 4 4"/>
            </svg>
          </button>
        </div>
      </div>
    </div>

    <!-- 设备选择器弹窗 -->
    <Transition name="fade">
      <div v-if="showDeviceSelector" class="overlay">
        <div class="modal">
          <div class="modal-header">
            <h3 class="modal-title">选择目标设备</h3>
            <button @click="showDeviceSelector = false" class="btn-close">
              <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d="M3 3l10 10M13 3l-10 10"/>
              </svg>
            </button>
          </div>

          <div class="modal-content">
            <div v-if="selectedFile" class="selected-file">
              <span class="label">已选择文件:</span>
              <span class="filename">{{ selectedFile.split(/[/\\]/).pop() }}</span>
            </div>

            <div v-if="availableDevices.length === 0" class="no-devices">
              没有可用的设备（需要在线且已配对）
            </div>

            <div v-else class="device-list">
              <div
                v-for="device in availableDevices"
                :key="device.hostname"
                class="device-item"
                @click="sendFileToDevice(device.hostname)"
              >
                <div class="device-info">
                  <div class="device-name">{{ device.name }}</div>
                  <div class="device-ip">{{ device.ip }}</div>
                </div>
                <div class="device-status online">
                  <svg width="8" height="8" viewBox="0 0 8 8">
                    <circle cx="4" cy="4" r="3" fill="#34c759"/>
                  </svg>
                  <span>在线</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.file-transfer {
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

.btn-send {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 18px;
  background: #007aff;
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
  -webkit-app-region: no-drag;
}

.btn-send:hover {
  background: #0062cc;
  transform: translateY(-1px);
}

.transfers-section,
.files-section {
  flex: 1;
  overflow-y: auto;
  padding: 16px 20px 20px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.transfers-section {
  border-bottom: 1px solid rgba(0, 0, 0, 0.08);
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: #86868b;
  margin-bottom: 4px;
}

.transfers-section::-webkit-scrollbar,
.files-section::-webkit-scrollbar {
  width: 8px;
}

.transfers-section::-webkit-scrollbar-track,
.files-section::-webkit-scrollbar-track {
  background: transparent;
}

.transfers-section::-webkit-scrollbar-thumb,
.files-section::-webkit-scrollbar-thumb {
  background: rgba(0, 0, 0, 0.2);
  border-radius: 4px;
}

.transfers-section::-webkit-scrollbar-thumb:hover,
.files-section::-webkit-scrollbar-thumb:hover {
  background: rgba(0, 0, 0, 0.3);
}

.transfer-item,
.file-item {
  background: rgba(255, 255, 255, 0.7);
  border-radius: 12px;
  padding: 16px;
  transition: all 0.2s ease;
}

.transfer-item:hover,
.file-item:hover {
  background: rgba(255, 255, 255, 0.95);
  transform: scale(1.01);
}

.transfer-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.transfer-file {
  font-size: 15px;
  font-weight: 500;
  color: #1c1c1e;
}

.transfer-status {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  font-weight: 500;
}

.transfer-status.uploading {
  color: #007aff;
}

.transfer-status.completed {
  color: #34c759;
}

.transfer-status.error {
  color: #ff3b30;
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.progress-bar {
  height: 4px;
  background: rgba(0, 0, 0, 0.1);
  border-radius: 2px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #007aff, #5856d6);
  border-radius: 2px;
  transition: width 0.3s ease;
}

.progress-text {
  font-size: 12px;
  color: #86868b;
  text-align: right;
  margin-top: 4px;
}

.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 200px;
  gap: 12px;
}

.empty-icon {
  font-size: 48px;
  opacity: 0.5;
}

.empty-text {
  font-size: 15px;
  color: #86868b;
  font-weight: 500;
}

.file-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.file-main {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}

.file-icon {
  font-size: 32px;
  flex-shrink: 0;
}

.file-info {
  flex: 1;
  min-width: 0;
}

.file-name {
  font-size: 15px;
  font-weight: 500;
  color: #1c1c1e;
  margin-bottom: 6px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.file-size,
.file-source,
.file-time {
  font-size: 12px;
  color: #86868b;
}

.file-source {
  padding: 2px 8px;
  background: rgba(0, 122, 255, 0.1);
  color: #007aff;
  border-radius: 10px;
}

.btn-open {
  width: 32px;
  height: 32px;
  border: none;
  background: rgba(0, 0, 0, 0.05);
  border-radius: 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #86868b;
  transition: all 0.2s ease;
  flex-shrink: 0;
  -webkit-app-region: no-drag;
}

.btn-open:hover {
  background: rgba(0, 0, 0, 0.1);
  color: #1c1c1e;
}

/* 弹窗样式 */
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

.modal {
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(20px);
  border-radius: 16px;
  min-width: 400px;
  max-width: 600px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px;
  border-bottom: 1px solid rgba(0, 0, 0, 0.08);
}

.modal-title {
  font-size: 18px;
  font-weight: 600;
  color: #1c1c1e;
  margin: 0;
}

.btn-close {
  width: 32px;
  height: 32px;
  border: none;
  background: rgba(0, 0, 0, 0.05);
  border-radius: 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #86868b;
  transition: all 0.2s ease;
  -webkit-app-region: no-drag;
}

.btn-close:hover {
  background: rgba(0, 0, 0, 0.1);
  color: #1c1c1e;
}

.modal-content {
  padding: 20px;
  overflow-y: auto;
}

.selected-file {
  background: rgba(0, 122, 255, 0.1);
  border-radius: 8px;
  padding: 12px;
  margin-bottom: 16px;
}

.selected-file .label {
  display: block;
  font-size: 12px;
  color: #86868b;
  margin-bottom: 4px;
}

.selected-file .filename {
  font-size: 14px;
  color: #007aff;
  font-weight: 500;
}

.no-devices {
  text-align: center;
  padding: 40px 20px;
  color: #86868b;
  font-size: 14px;
}

.device-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.device-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px;
  background: rgba(0, 0, 0, 0.03);
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.2s ease;
  -webkit-app-region: no-drag;
}

.device-item:hover {
  background: rgba(0, 122, 255, 0.1);
  transform: translateX(4px);
}

.device-info {
  flex: 1;
}

.device-name {
  font-size: 15px;
  font-weight: 500;
  color: #1c1c1e;
  margin-bottom: 4px;
}

.device-ip {
  font-size: 13px;
  color: #86868b;
  font-family: 'SF Mono', Monaco, 'Courier New', monospace;
}

.device-status {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: #34c759;
  font-weight: 500;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.btn-clear {
  width: 28px;
  height: 28px;
  border: none;
  background: rgba(0, 0, 0, 0.05);
  border-radius: 6px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #86868b;
  transition: all 0.2s ease;
  padding: 0;
  -webkit-app-region: no-drag;
}

.btn-clear:hover {
  background: rgba(255, 59, 48, 0.1);
  color: #ff3b30;
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
