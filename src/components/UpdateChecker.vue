<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { check } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import { getVersion } from '@tauri-apps/api/app'

interface UpdateInfo {
  version: string
  currentVersion: string
  date?: string
  body?: string
}

const showUpdateDialog = ref(false)
const showNoUpdateDialog = ref(false)
const updateInfo = ref<UpdateInfo | null>(null)
const currentVersion = ref('')
const isDownloading = ref(false)
const downloadProgress = ref(0)
const isInstalling = ref(false)
const errorMessage = ref('')
const lastCheckTime = ref<number | null>(null)

// 获取当前版本
async function fetchCurrentVersion() {
  try {
    currentVersion.value = await getVersion()
  } catch (error) {
    console.error('获取版本失败:', error)
    currentVersion.value = '1.0.0'
  }
}

// 检查更新
async function checkForUpdate(silent = false) {
  try {
    errorMessage.value = ''
    showNoUpdateDialog.value = false

    const update = await check()

    lastCheckTime.value = Date.now()

    if (update) {
      updateInfo.value = {
        version: update.version,
        currentVersion: update.currentVersion,
        date: update.date,
        body: update.body
      }
      currentVersion.value = update.currentVersion
      showUpdateDialog.value = true
      console.log('📦 发现新版本:', update.version)
    } else {
      if (!silent) {
        showNoUpdateDialog.value = true
        setTimeout(() => {
          showNoUpdateDialog.value = false
        }, 2000)
      }
      console.log('✓ 当前已是最新版本')
    }
  } catch (error) {
    console.error('检查更新失败:', error)
    if (!silent) {
      errorMessage.value = `检查更新失败: ${(error as Error).message}`
      alert(errorMessage.value)
    }
  }
}

// 下载并安装更新
async function downloadAndInstall() {
  if (!updateInfo.value) return
  
  isDownloading.value = true
  downloadProgress.value = 0
  errorMessage.value = ''
  
  try {
    const update = await check()
    
    if (!update) {
      errorMessage.value = '未找到更新'
      isDownloading.value = false
      return
    }
    
    let downloaded = 0
    let contentLength = 0
    
    await update.downloadAndInstall((event) => {
      switch (event.event) {
        case 'Started':
          contentLength = event.data.contentLength || 0
          console.log(`开始下载，总大小: ${contentLength} 字节`)
          break
        case 'Progress':
          downloaded += event.data.chunkLength
          if (contentLength > 0) {
            downloadProgress.value = Math.round((downloaded / contentLength) * 100)
          }
          break
        case 'Finished':
          console.log('下载完成')
          break
      }
    })
    
    isDownloading.value = false
    isInstalling.value = true
    
    // 安装完成后重启应用
    await relaunch()
  } catch (error) {
    console.error('下载更新失败:', error)
    errorMessage.value = `下载更新失败: ${(error as Error).message}`
    isDownloading.value = false
  }
}

// 稍后提醒
function remindLater() {
  showUpdateDialog.value = false
}

// 跳过此版本
function skipVersion() {
  if (updateInfo.value) {
    localStorage.setItem('skipVersion', updateInfo.value.version)
  }
  showUpdateDialog.value = false
}

// 检查是否应该跳过此版本
function shouldSkipVersion(version: string): boolean {
  const skippedVersion = localStorage.getItem('skipVersion')
  return skippedVersion === version
}

// 格式化更新说明
function formatBody(body?: string): string {
  if (!body) return '暂无更新说明'
  return body.replace(/\\n/g, '\n')
}

// 自动检查更新（静默模式）
onMounted(async () => {
  await fetchCurrentVersion()
  // 应用启动 5 秒后静默检查更新
  setTimeout(async () => {
    const update = await check()
    if (update && !shouldSkipVersion(update.version)) {
      updateInfo.value = {
        version: update.version,
        currentVersion: update.currentVersion,
        date: update.date,
        body: update.body
      }
      currentVersion.value = update.currentVersion
      showUpdateDialog.value = true
    }
  }, 5000)
})

// 暴露方法供外部调用
defineExpose({
  checkForUpdate,
  currentVersion
})
</script>

<template>
  <!-- 已是最新版本提示 -->
  <Transition name="toast">
    <div v-if="showNoUpdateDialog" class="toast-message">
      <svg width="20" height="20" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M10 18a8 8 0 100-16 8 8 0 000 16z"/>
        <path d="M6 10l3 3 6-6"/>
      </svg>
      <span>当前已是最新版本 v{{ currentVersion }}</span>
    </div>
  </Transition>

  <!-- 更新提示弹窗 -->
  <Transition name="modal">
    <div v-if="showUpdateDialog" class="modal-overlay" @click.self="remindLater">
      <div class="modal-content update-modal">
        <div class="modal-header">
          <div class="update-icon">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
              <polyline points="7 10 12 15 17 10"/>
              <line x1="12" y1="15" x2="12" y2="3"/>
            </svg>
          </div>
          <h3 class="modal-title">发现新版本</h3>
        </div>

        <div class="modal-body">
          <div v-if="updateInfo" class="update-info">
            <div class="version-info">
              <span class="current">当前版本: v{{ updateInfo.currentVersion }}</span>
              <span class="arrow">→</span>
              <span class="new">v{{ updateInfo.version }}</span>
            </div>
            
            <div v-if="updateInfo.date" class="release-date">
              发布日期: {{ new Date(updateInfo.date).toLocaleDateString('zh-CN') }}
            </div>

            <div v-if="updateInfo.body" class="release-notes">
              <div class="notes-title">更新内容:</div>
              <pre class="notes-content">{{ formatBody(updateInfo.body) }}</pre>
            </div>
          </div>

          <!-- 下载进度 -->
          <div v-if="isDownloading" class="download-progress">
            <div class="progress-bar">
              <div class="progress-fill" :style="{ width: downloadProgress + '%' }"></div>
            </div>
            <div class="progress-text">正在下载... {{ downloadProgress }}%</div>
          </div>

          <!-- 安装中 -->
          <div v-if="isInstalling" class="installing">
            <div class="spinner"></div>
            <span>正在安装，请稍候...</span>
          </div>

          <!-- 错误信息 -->
          <div v-if="errorMessage" class="error-message">
            {{ errorMessage }}
          </div>
        </div>

        <div class="modal-footer">
          <template v-if="!isDownloading && !isInstalling">
            <button @click="remindLater" class="btn btn-secondary">
              稍后提醒
            </button>
            <button @click="skipVersion" class="btn btn-secondary">
              跳过此版本
            </button>
            <button @click="downloadAndInstall" class="btn btn-primary">
              立即更新
            </button>
          </template>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
/* Toast 提示 */
.toast-message {
  position: fixed;
  top: 20px;
  left: 50%;
  transform: translateX(-50%);
  background: #34c759;
  color: white;
  padding: 12px 24px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  font-weight: 500;
  box-shadow: 0 4px 20px rgba(52, 199, 89, 0.3);
  z-index: 3000;
}

.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s ease;
}

.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(-20px);
}

.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
  padding: 20px;
}

.update-modal {
  background: white;
  border-radius: 16px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.2);
  max-width: 420px;
  width: 100%;
  overflow: hidden;
}

.modal-header {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 24px 24px 16px;
  background: linear-gradient(135deg, #007aff 0%, #5856d6 100%);
  color: white;
}

.update-icon {
  width: 48px;
  height: 48px;
  background: rgba(255, 255, 255, 0.2);
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 12px;
}

.modal-title {
  font-size: 20px;
  font-weight: 600;
  margin: 0;
}

.modal-body {
  padding: 20px 24px;
}

.update-info {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.version-info {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  font-size: 15px;
}

.current {
  color: #86868b;
}

.arrow {
  color: #34c759;
}

.new {
  color: #007aff;
  font-weight: 600;
}

.release-date {
  text-align: center;
  font-size: 13px;
  color: #86868b;
}

.release-notes {
  background: #f5f5f7;
  border-radius: 8px;
  padding: 12px;
}

.notes-title {
  font-size: 13px;
  font-weight: 500;
  color: #1c1c1e;
  margin-bottom: 8px;
}

.notes-content {
  font-size: 13px;
  color: #3c3c43;
  margin: 0;
  white-space: pre-wrap;
  word-break: break-word;
  font-family: inherit;
  line-height: 1.5;
}

.download-progress {
  margin-top: 16px;
}

.progress-bar {
  height: 6px;
  background: #e5e5ea;
  border-radius: 3px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #007aff, #5856d6);
  border-radius: 3px;
  transition: width 0.3s ease;
}

.progress-text {
  text-align: center;
  font-size: 13px;
  color: #86868b;
  margin-top: 8px;
}

.installing {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 16px;
  background: #f5f5f7;
  border-radius: 8px;
  margin-top: 16px;
}

.spinner {
  width: 20px;
  height: 20px;
  border: 2px solid #e5e5ea;
  border-top-color: #007aff;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.error-message {
  margin-top: 12px;
  padding: 10px;
  background: #fff5f5;
  border-radius: 8px;
  color: #ff3b30;
  font-size: 13px;
  text-align: center;
}

.modal-footer {
  display: flex;
  gap: 12px;
  padding: 16px 24px 24px;
  justify-content: flex-end;
}

.btn {
  padding: 10px 20px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  border: none;
}

.btn-primary {
  background: #007aff;
  color: white;
}

.btn-primary:hover {
  background: #0062cc;
}

.btn-secondary {
  background: #f5f5f7;
  color: #1c1c1e;
}

.btn-secondary:hover {
  background: #e5e5ea;
}

/* Transition */
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.2s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

/* 深色模式 */
.dark-mode .update-modal {
  background: #2c2c2e;
}

.dark-mode .release-notes {
  background: #3a3a3c;
}

.dark-mode .notes-title {
  color: #fff;
}

.dark-mode .notes-content {
  color: rgba(255, 255, 255, 0.7);
}

.dark-mode .progress-bar {
  background: #3a3a3c;
}

.dark-mode .progress-text {
  color: rgba(255, 255, 255, 0.5);
}

.dark-mode .installing {
  background: #3a3a3c;
}

.dark-mode .btn-secondary {
  background: #3a3a3c;
  color: #fff;
}

.dark-mode .btn-secondary:hover {
  background: #48484a;
}
</style>
