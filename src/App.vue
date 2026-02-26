<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { save } from '@tauri-apps/plugin-dialog'
import { writeFile } from '@tauri-apps/plugin-fs'
import SyncStatusIndicator from './components/SyncStatusIndicator.vue'
import DevicePanel from './components/DevicePanel.vue'
import FileTransfer from './components/FileTransfer.vue'
import PairRequestDialog from './components/PairRequestDialog.vue'
import PrivacyPolicy from './components/PrivacyPolicy.vue'
import WelcomeGuide from './components/WelcomeGuide.vue'

// 模态对话框状态
const showDevices = ref(false)
const showFileTransfer = ref(false)
const showSettings = ref(false)
const showClearConfirm = ref(false)

// 深色模式
const darkMode = ref(true)

// 历史记录数量限制
const historyLimit = ref(100)

// 自启动状态
const autostartEnabled = ref(false)

// 服务器状态
interface ServerStatus {
  is_running: boolean
  port: number | null
  started_at: string | null
}
const serverStatus = ref<ServerStatus>({
  is_running: false,
  port: null,
  started_at: null
})
const serverOperating = ref(false)

// 自定义端口配置
const customPort = ref<number>(9527)
const portSaving = ref(false)

// 剪贴板数据
interface ClipboardItem {
  id: number
  content_type: string
  content: string
  file_path?: string  // 图片文件路径（仅图片类型）
  created_at: string
  favorite?: boolean   // 是否收藏
}

const items = ref<ClipboardItem[]>([])
const searchQuery = ref('')
const selectedIndex = ref(-1)
const activeTab = ref<'all' | 'text' | 'image'>('all')

// 组件引用
const syncIndicator = ref<InstanceType<typeof SyncStatusIndicator> | null>(null)
const privacyPolicy = ref<InstanceType<typeof PrivacyPolicy> | null>(null)
const welcomeGuide = ref<InstanceType<typeof WelcomeGuide> | null>(null)

// 事件监听器
let unlistenClipboard: UnlistenFn | null = null
let unlistenPairAccepted: UnlistenFn | null = null
let unlistenPairRejected: UnlistenFn | null = null
let unlistenOpenDevices: UnlistenFn | null = null
let unlistenOpenFileTransfer: UnlistenFn | null = null

// 加载所有剪贴板记录
async function loadItems() {
  try {
    const result = await invoke<ClipboardItem[]>('get_all_items')
    console.log('📋 Loaded items:', result)
    items.value = result
    selectedIndex.value = -1
    
    // 预加载图片 URL
    for (const item of result) {
      if (item.content_type === 'image' && item.file_path) {
        loadImage(item)
      }
    }
  } catch (error) {
    console.error('Failed to load items:', error)
  }
}

// 加载单个图片（使用 HTTP URL）
function loadImage(item: ClipboardItem) {
  if (imageUrlCache.value.has(item.id)) {
    return
  }
  
  // 检查服务器是否运行
  if (!serverStatus.value.is_running || !serverPort.value) {
    console.log('📷 Server not running, skipping image load')
    return
  }
  
  if (item.file_path) {
    // 从绝对路径中提取文件名
    const filename = item.file_path.split('/').pop() || item.file_path.split('\\').pop()
    if (filename) {
      // 构建 HTTP URL
      const url = `http://127.0.0.1:${serverPort.value}/images/${filename}`
      imageUrlCache.value.set(item.id, url)
      console.log('📷 Image URL:', url)
    }
  }
}

// 搜索记录
async function searchItems() {
  if (!searchQuery.value.trim()) {
    loadItems()
    return
  }
  try {
    const result = await invoke<ClipboardItem[]>('search_items', { query: searchQuery.value })
    items.value = result
    selectedIndex.value = -1
  } catch (error) {
    console.error('Failed to search items:', error)
  }
}

// 处理图片加载错误
function handleImageError(e: Event) {
  const img = e.target as HTMLImageElement
  console.error('❌ Image load failed!')
  console.error('   src:', img.src)
  console.error('   naturalWidth:', img.naturalWidth)
  console.error('   naturalHeight:', img.naturalHeight)
  
  // 显示错误占位符
  img.style.display = 'none'
  const parent = img.parentElement
  if (parent) {
    parent.innerHTML = `
      <div class="image-error">
        <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="#ff3b30" stroke-width="2">
          <rect x="3" y="3" width="18" height="18" rx="2"/>
          <circle cx="8.5" cy="8.5" r="1.5"/>
          <path d="M21 15l-5-5L5 21"/>
        </svg>
        <span>图片加载失败</span>
        <span style="font-size:10px;color:#999;word-break:break-all;">${img.src}</span>
      </div>
    `
  }
}

// 图片 URL 缓存
const imageUrlCache = ref<Map<number, string>>(new Map())
// 服务器端口
const serverPort = ref<number>(9527)

// 获取服务器端口
async function fetchServerPort() {
  try {
    const port = await invoke<number>('get_server_port')
    if (port) {
      serverPort.value = port
      console.log('🔌 Server port:', port)
    }
  } catch (error) {
    console.error('Failed to get server port:', error)
  }
}

// 获取服务器状态
async function fetchServerStatus() {
  try {
    const status = await invoke<ServerStatus>('get_server_status')
    serverStatus.value = status
    // 同步更新 serverPort
    if (status.port) {
      serverPort.value = status.port
    }
    console.log('🔌 Server status:', status)
  } catch (error) {
    console.error('Failed to get server status:', error)
  }
}

// 获取自定义端口配置
async function fetchCustomPort() {
  try {
    const port = await invoke<number>('get_server_config_port')
    customPort.value = port
    console.log('🔌 Custom port:', port)
  } catch (error) {
    console.error('Failed to get custom port:', error)
  }
}

// 保存自定义端口配置
async function saveCustomPort() {
  if (portSaving.value) return
  
  // 验证端口范围
  if (customPort.value < 1024 || customPort.value > 65535) {
    alert('端口范围必须在 1024-65535 之间')
    return
  }
  
  portSaving.value = true
  try {
    await invoke('set_server_config_port', { port: customPort.value })
    console.log('✓ Port saved:', customPort.value)
    
    // 如果服务正在运行，提示重启
    if (serverStatus.value.is_running) {
      const shouldRestart = confirm('端口配置已保存，是否立即重启服务以应用新端口？')
      if (shouldRestart) {
        await restartServer()
      }
    }
  } catch (error) {
    console.error('Failed to save port:', error)
    alert(`保存失败: ${(error as any).message || error}`)
  } finally {
    portSaving.value = false
  }
}

// 打开图片资源目录
async function openImagesDirectory() {
  try {
    const path = await invoke<string>('open_images_directory')
    console.log('✓ Opened images directory:', path)
  } catch (error) {
    console.error('Failed to open images directory:', error)
    alert(`打开目录失败: ${(error as any).message || error}`)
  }
}

// 切换深色模式
function toggleDarkMode() {
  darkMode.value = !darkMode.value
  localStorage.setItem('darkMode', String(darkMode.value))
  applyDarkMode()
}

// 应用深色模式
function applyDarkMode() {
  if (darkMode.value) {
    document.documentElement.classList.add('dark-mode')
  } else {
    document.documentElement.classList.remove('dark-mode')
  }
}

// 设置历史记录数量限制
function setHistoryLimit(limit: number) {
  historyLimit.value = limit
  localStorage.setItem('historyLimit', String(limit))
}

// 导出数据
async function exportData() {
  try {
    // 打开保存文件对话框
    const filePath = await save({
      defaultPath: `clipboard-backup-${new Date().toISOString().slice(0, 10)}.json`,
      filters: [
        { name: 'JSON', extensions: ['json'] }
      ]
    })
    
    if (!filePath) {
      return // 用户取消了
    }
    
    const data = {
      items: items.value,
      exportTime: new Date().toISOString(),
      version: '1.0'
    }
    
    const content = JSON.stringify(data, null, 2)
    const encoder = new TextEncoder()
    const bytes = encoder.encode(content)
    
    await writeFile(filePath, bytes)
    
    console.log('✓ Data exported to:', filePath)
    alert('导出成功！')
  } catch (error) {
    console.error('Failed to export data:', error)
    alert(`导出失败: ${(error as any).message || error}`)
  }
}

// 启动服务器
async function startServer() {
  if (serverOperating.value) return
  serverOperating.value = true
  
  try {
    const status = await invoke<ServerStatus>('start_server_service')
    serverStatus.value = status
    if (status.port) {
      serverPort.value = status.port
    }
    console.log('✓ Server started:', status)
    
    // 启动mDNS设备发现服务
    try {
      await invoke('start_mdns_discovery')
      console.log('✓ mDNS discovery started')
    } catch (mdnsError) {
      console.warn('Failed to start mDNS discovery:', mdnsError)
    }
  } catch (error) {
    console.error('Failed to start server:', error)
    alert(`启动服务失败: ${(error as any).message || error}`)
  } finally {
    serverOperating.value = false
  }
}

// 停止服务器
async function stopServer() {
  if (serverOperating.value) return
  serverOperating.value = true
  
  try {
    // 先停止mDNS发现
    try {
      await invoke('stop_mdns_discovery')
      console.log('✓ mDNS discovery stopped')
    } catch (e) {
      // 忽略错误
    }
    
    const status = await invoke<ServerStatus>('stop_server_service')
    serverStatus.value = status
    console.log('✓ Server stopped:', status)
  } catch (error) {
    console.error('Failed to stop server:', error)
    alert(`停止服务失败: ${(error as any).message || error}`)
  } finally {
    serverOperating.value = false
  }
}

// 重启服务器
async function restartServer() {
  if (serverOperating.value) return
  serverOperating.value = true
  
  try {
    // 先停止mDNS发现
    try {
      await invoke('stop_mdns_discovery')
    } catch (e) {
      // 忽略错误
    }
    
    const status = await invoke<ServerStatus>('restart_server_service')
    serverStatus.value = status
    if (status.port) {
      serverPort.value = status.port
    }
    console.log('✓ Server restarted:', status)
    
    // 重新启动mDNS设备发现服务
    try {
      await invoke('start_mdns_discovery')
      console.log('✓ mDNS discovery restarted')
    } catch (mdnsError) {
      console.warn('Failed to restart mDNS discovery:', mdnsError)
    }
  } catch (error) {
    console.error('Failed to restart server:', error)
    alert(`重启服务失败: ${(error as any).message || error}`)
  } finally {
    serverOperating.value = false
  }
}

// 复制文本或图片到剪贴板
async function copyText(item: ClipboardItem) {
  try {
    if (item.content_type === 'image' && item.file_path) {
      // 图片：从 HTTP URL 读取并复制到剪贴板
      const filename = item.file_path.split('/').pop() || item.file_path.split('\\').pop()
      const imageUrl = `http://127.0.0.1:${serverPort.value}/images/${filename}`
      const response = await fetch(imageUrl)
      const blob = await response.blob()

      // 使用 image/png 格式复制
      const clipboardItem = new ClipboardItem({ 'image/png': blob })
      await navigator.clipboard.write([clipboardItem])
      console.log('✓ 图片已复制到剪贴板')
    } else {
      // 文本：使用 Rust invoke
      await invoke('copy_text', { text: item.content })
    }
    await hideWindow()
  } catch (error) {
    console.error('Failed to copy:', error)
  }
}

// 保存图片到本地
async function saveImage(item: ClipboardItem) {
  try {
    if (item.content_type !== 'image' || !item.file_path) {
      throw new Error('只能保存图片类型')
    }

    // 从 HTTP URL 读取图片
    const imageFilename = item.file_path.split('/').pop() || item.file_path.split('\\').pop()
    const imageUrl = `http://127.0.0.1:${serverPort.value}/images/${imageFilename}`
    const response = await fetch(imageUrl)
    const blob = await response.blob()

    // 转换为 base64
    const reader = new FileReader()
    const base64Promise = new Promise<string>((resolve, reject) => {
      reader.onloadend = () => {
        const base64 = reader.result as string
        // 移除 data:image/xxx;base64, 前缀
        const base64Data = base64.split(',')[1]
        resolve(base64Data)
      }
      reader.onerror = reject
    })
    reader.readAsDataURL(blob)

    const base64Data = await base64Promise
    // 不传递扩展名，让后端根据图片数据自动检测格式
    const saveFilename = `screenshot_${Date.now()}`

    await invoke('save_image_to_file', {
      base64Data,
      filename: saveFilename
    })

    console.log('✓ 图片已保存')
  } catch (error) {
    console.error('Failed to save image:', error)
    alert(`保存失败: ${(error as any).message || error}`)
  }
}

// 删除记录
async function deleteItem(id: number) {
  try {
    await invoke('delete_item', { id })
    await loadItems()
  } catch (error) {
    console.error('Failed to delete item:', error)
  }
}

// 切换收藏状态
async function toggleFavorite(id: number) {
  try {
    const newState = await invoke<boolean>('toggle_favorite', { id })
    // 更新本地状态
    const item = items.value.find(i => i.id === id)
    if (item) {
      item.favorite = newState
    }
    console.log('✓ Favorite toggled:', id, newState)
  } catch (error) {
    console.error('Failed to toggle favorite:', error)
  }
}

// 清空所有记录 - 打开确认弹窗
function openClearConfirm() {
  showClearConfirm.value = true
}

// 清空所有记录 - 执行清空
async function clearAll() {
  try {
    await invoke('clear_all')
    await loadItems()
    showClearConfirm.value = false
  } catch (error) {
    console.error('Failed to clear items:', error)
  }
}

// 清除哈希缓存状态
const clearingCache = ref(false)
const cacheCleared = ref(false)
const cacheError = ref('')

// 清除哈希缓存
async function clearHashCache() {
  if (clearingCache.value) return

  clearingCache.value = true
  cacheError.value = ''

  try {
    await invoke('clear_hash_cache')
    console.log('✓ Hash cache cleared successfully')

    // 显示成功状态
    cacheCleared.value = true
    
    // 2秒后重置状态
    setTimeout(() => {
      cacheCleared.value = false
    }, 2000)

    // 可选：使用系统通知（需要用户授权）
    if ('Notification' in window && Notification.permission === 'granted') {
      new Notification('缓存已清除', {
        body: '哈希缓存已清除，现在可以重新同步图片了',
        icon: '/icons/32x32.png'
      })
    }
  } catch (error) {
    console.error('Failed to clear hash cache:', error)
    
    // 显示错误状态
    cacheError.value = `${(error as any).message || error}`
    
    // 3秒后清除错误
    setTimeout(() => {
      cacheError.value = ''
    }, 3000)
  } finally {
    clearingCache.value = false
  }
}

// 隐藏窗口
async function hideWindow() {
  try {
    await invoke('hide_window')
  } catch (error) {
    console.error('Failed to hide window:', error)
  }
}

// 检查自启动状态
async function checkAutostartStatus() {
  try {
    console.log('Checking autostart status...')
    const enabled = await invoke<boolean>('plugin:autostart|is_enabled')
    autostartEnabled.value = enabled
    console.log('Autostart status:', enabled ? 'enabled' : 'disabled')
  } catch (error) {
    console.error('Failed to check autostart status:', error)
    console.error('Error details:', JSON.stringify(error))
    autostartEnabled.value = false
  }
}

// 切换自启动
async function toggleAutostart() {
  try {
    console.log('Toggling autostart, current status:', autostartEnabled.value)
    if (autostartEnabled.value) {
      console.log('Disabling autostart...')
      await invoke('plugin:autostart|disable')
      autostartEnabled.value = false
      console.log('Autostart disabled successfully')
    } else {
      console.log('Enabling autostart...')
      await invoke('plugin:autostart|enable')
      autostartEnabled.value = true
      console.log('Autostart enabled successfully')
    }
  } catch (error) {
    console.error('Failed to toggle autostart:', error)
    console.error('Error details:', JSON.stringify(error))

    // 尝试读取错误信息
    const errorMsg = error as any
    if (errorMsg.message) {
      console.error('Error message:', errorMsg.message)
    }
    if (errorMsg.cause) {
      console.error('Error cause:', errorMsg.cause)
    }

    // 回滚状态
    if (!autostartEnabled.value) {
      autostartEnabled.value = false
    } else {
      autostartEnabled.value = true
    }
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

// 过滤后的项目
const filteredItems = computed(() => {
  let result = items.value
  
  // 根据tab筛选
  if (activeTab.value === 'text') {
    result = result.filter(item => item.content_type !== 'image')
  } else if (activeTab.value === 'image') {
    result = result.filter(item => item.content_type === 'image')
  }
  
  // 根据搜索词筛选
  if (searchQuery.value.trim()) {
    result = result.filter(item =>
      item.content.toLowerCase().includes(searchQuery.value.toLowerCase())
    )
  }
  
  return result
})

// 更新连接设备数量
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

// 键盘导航
function handleKeyDown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    if (showDevices.value) {
      showDevices.value = false
    } else if (showFileTransfer.value) {
      showFileTransfer.value = false
    } else {
      hideWindow()
    }
    event.preventDefault()
  } else if (event.key === 'ArrowDown') {
    if (showDevices.value || showFileTransfer.value) return
    event.preventDefault()
    selectedIndex.value = Math.min(selectedIndex.value + 1, filteredItems.value.length - 1)
  } else if (event.key === 'ArrowUp') {
    if (showDevices.value || showFileTransfer.value) return
    event.preventDefault()
    selectedIndex.value = Math.max(selectedIndex.value - 1, 0)
  } else if (event.key === 'Enter' && selectedIndex.value >= 0) {
    if (showDevices.value || showFileTransfer.value) return
    event.preventDefault()
    copyText(filteredItems.value[selectedIndex.value])
  }
}

// 监听剪贴板更新事件
async function setupClipboardListener() {
  try {
    unlistenClipboard = await listen<ClipboardItem>('clipboard-update', (event) => {
      items.value.unshift(event.payload)
    })
  } catch (error) {
    console.error('Failed to setup clipboard listener:', error)
  }
}

// 监听配对事件更新连接数
async function setupPairingListeners() {
  try {
    unlistenPairAccepted = await listen('pair-accepted', async () => {
      await updateConnectedCount()
    })

    unlistenPairRejected = await listen('pair-rejected', async () => {
      await updateConnectedCount()
    })
  } catch (error) {
    console.error('Failed to setup pairing listeners:', error)
  }
}

// 处理设备数量变化
function handleDevicesChanged(count: number) {
  console.log('Devices changed:', count)
  updateConnectedCount()
}

// 监听系统托盘事件
async function setupTrayListeners() {
  try {
    unlistenOpenDevices = await listen('open-devices', () => {
      showDevices.value = true
    })

    unlistenOpenFileTransfer = await listen('open-file-transfer', () => {
      showFileTransfer.value = true
    })
  } catch (error) {
    console.error('Failed to setup tray listeners:', error)
  }
}

onMounted(async () => {
  // 加载设置
  const savedDarkMode = localStorage.getItem('darkMode')
  if (savedDarkMode !== null) {
    darkMode.value = savedDarkMode === 'true'
  }
  applyDarkMode()
  
  const savedHistoryLimit = localStorage.getItem('historyLimit')
  if (savedHistoryLimit) {
    historyLimit.value = parseInt(savedHistoryLimit)
  }
  
  await fetchServerPort()
  await fetchServerStatus()
  await fetchCustomPort()
  await loadItems()
  await updateConnectedCount()
  await setupClipboardListener()
  await setupPairingListeners()
  await setupTrayListeners()
  await checkAutostartStatus()
  window.addEventListener('keydown', handleKeyDown)

  // 监听窗口关闭事件，阻止默认关闭行为
  await getCurrentWindow().onCloseRequested(async (event) => {
    console.log('Window close requested, preventing default and hiding')
    event.preventDefault()
    await invoke('hide_window')
  })
})

onUnmounted(() => {
  if (unlistenClipboard) unlistenClipboard()
  if (unlistenPairAccepted) unlistenPairAccepted()
  if (unlistenPairRejected) unlistenPairRejected()
  if (unlistenOpenDevices) unlistenOpenDevices()
  if (unlistenOpenFileTransfer) unlistenOpenFileTransfer()
  window.removeEventListener('keydown', handleKeyDown)
})
</script>

<template>
  <div class="app">
    <!-- 顶部工具栏 -->
    <div class="header">
      <div class="header-left">
        <div class="app-title">剪贴板同步</div>
      </div>

      <div class="header-right">
        <button
          @click="showDevices = true"
          class="toolbar-btn"
          title="设备管理"
        >
          <svg width="18" height="18" viewBox="0 0 18 18" fill="none" stroke="currentColor" stroke-width="1.5">
            <rect x="2" y="3" width="14" height="11" rx="2"/>
            <path d="M9 14v2"/>
            <circle cx="9" cy="16" r="1"/>
          </svg>
          <span class="btn-label">设备</span>
        </button>
        <button
          @click="showFileTransfer = true"
          class="toolbar-btn"
          title="文件传输"
        >
          <svg width="18" height="18" viewBox="0 0 18 18" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M3 9h12M12 6l3 3-3 3"/>
          </svg>
          <span class="btn-label">文件</span>
        </button>
        <button
          @click="showSettings = true"
          class="toolbar-btn"
          title="设置"
        >
          <svg width="18" height="18" viewBox="0 0 18 18" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M9 1.5a7.5 7.5 0 0 1 0 15 7.5 7.5 0 0 1 0-15z"/>
            <path d="M9 4v5l3.5 2"/>
          </svg>
          <span class="btn-label">设置</span>
        </button>
        <SyncStatusIndicator ref="syncIndicator" />
      </div>
    </div>

    <!-- 主内容区：剪贴板历史 -->
    <div class="main-content">
      <!-- 搜索框 -->
      <div class="search-container">
        <svg class="search-icon" width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="7" cy="7" r="6"/>
          <line x1="11.5" y1="11.5" x2="14.5" y2="14.5"/>
        </svg>
        <input
          v-model="searchQuery"
          @input="searchItems"
          placeholder="搜索剪贴板历史"
          class="search-input"
          autoFocus
        />
        <button v-if="items.length > 0" @click="openClearConfirm" class="clear-btn" title="清空所有记录">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M2.5 5h11M6 9v3M10 9v3M5.5 5V3.5a.5.5 0 0 1 .5-.5h4a.5.5 0 0 1 .5.5V5M3 5l.5 9.5A1 1 0 0 0 4.5 15.5h7a1 1 0 0 0 1-1L13 5"/>
          </svg>
        </button>
      </div>

      <!-- Tab切换 -->
      <div class="tabs-container">
        <button 
          class="tab-btn" 
          :class="{ active: activeTab === 'all' }"
          @click="activeTab = 'all'; selectedIndex = -1"
        >
          全部
          <span class="tab-count">{{ items.length }}</span>
        </button>
        <button 
          class="tab-btn" 
          :class="{ active: activeTab === 'text' }"
          @click="activeTab = 'text'; selectedIndex = -1"
        >
          文本
          <span class="tab-count">{{ items.filter(i => i.content_type !== 'image').length }}</span>
        </button>
        <button 
          class="tab-btn" 
          :class="{ active: activeTab === 'image' }"
          @click="activeTab = 'image'; selectedIndex = -1"
        >
          图片
          <span class="tab-count">{{ items.filter(i => i.content_type === 'image').length }}</span>
        </button>
      </div>

      <!-- 剪贴板历史列表 -->
      <div class="list">
        <div v-if="filteredItems.length === 0" class="empty">
          {{ items.length === 0 ? '暂无剪贴板记录' : '未找到匹配的记录' }}
        </div>

        <div
          v-for="(item, index) in filteredItems"
          :key="item.id"
          class="item"
          :class="{ selected: index === selectedIndex }"
          @click="copyText(item)"
        >
          <div class="item-main">
            <div class="item-header">
              <div class="item-time">{{ formatTime(item.created_at) }}</div>
              <div class="item-type">{{ item.content_type === 'image' ? '图片' : '文本' }}</div>
              <button 
                @click.stop="toggleFavorite(item.id)" 
                class="favorite-btn" 
                :class="{ active: item.favorite }"
                :title="item.favorite ? '取消收藏' : '收藏'"
              >
                <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5">
                  <path d="M7 1l1.5 4.5H13l-3.5 2.5 1.5 4.5L7 10l-4 2.5 1.5-4.5L1 5.5h4.5z"/>
                </svg>
              </button>
              <div v-if="item.content_type === 'image'" class="item-actions">
                <button @click.stop="saveImage(item)" class="save-btn" title="保存到本地">
                  <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5">
                    <path d="M2 3.5a1.5 1.5 0 0 1 1.5-1.5h7a1.5 1.5 0 0 1 1.5 1.5v7a1.5 1.5 0 0 1-1.5 1.5h-7A1.5 1.5 0 0 1 2 10.5v-7z"/>
                    <path d="M4.5 7.5l2.5 2.5 5-6"/>
                  </svg>
                </button>
              </div>
              <button @click.stop="deleteItem(item.id)" class="delete-btn" title="删除">
                <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5">
                  <line x1="3" y1="3" x2="11" y2="11"/>
                  <line x1="11" y1="3" x2="3" y2="11"/>
                </svg>
              </button>
            </div>
            <!-- 图片显示缩略图 -->
            <div v-if="item.content_type === 'image' && item.file_path" class="item-image">
              <img
                v-if="imageUrlCache.get(item.id)"
                :src="imageUrlCache.get(item.id)"
                :alt="`截图-${item.id}`"
                @error="handleImageError"
                loading="lazy"
              />
              <div v-else class="image-loading">
                <span>加载中...</span>
              </div>
            </div>
            <!-- 文本显示内容 -->
            <div v-else class="item-text">{{ item.content }}</div>
          </div>
        </div>
      </div>

      <!-- 底部提示 -->
      <div class="footer">
        <span class="hint">↑↓ 选择 • Enter 复制 • Esc 关闭 • 设备/文件 按钮打开管理</span>
      </div>
    </div>

    <!-- 设备管理模态框 -->
    <Transition name="modal">
      <div v-if="showDevices" class="modal-overlay" @click.self="showDevices = false">
        <div class="modal-content devices-modal">
          <div class="modal-header">
            <h2 class="modal-title">设备管理</h2>
            <button @click="showDevices = false" class="modal-close">
              <svg width="20" height="20" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="4" y1="4" x2="16" y2="16"/>
                <line x1="16" y1="4" x2="4" y2="16"/>
              </svg>
            </button>
          </div>
          <DevicePanel @devices-changed="handleDevicesChanged" />
        </div>
      </div>
    </Transition>

    <!-- 文件传输模态框 -->
    <Transition name="modal">
      <div v-if="showFileTransfer" class="modal-overlay" @click.self="showFileTransfer = false">
        <div class="modal-content files-modal">
          <div class="modal-header">
            <h2 class="modal-title">文件传输</h2>
            <button @click="showFileTransfer = false" class="modal-close">
              <svg width="20" height="20" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="4" y1="4" x2="16" y2="16"/>
                <line x1="16" y1="4" x2="4" y2="16"/>
              </svg>
            </button>
          </div>
          <FileTransfer />
        </div>
      </div>
    </Transition>

    <!-- 设置模态框 -->
    <Transition name="modal">
      <div v-if="showSettings" class="modal-overlay" @click.self="showSettings = false">
        <div class="modal-content settings-modal">
          <div class="modal-header">
            <h2 class="modal-title">设置</h2>
            <button @click="showSettings = false" class="modal-close">
              <svg width="20" height="20" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="4" y1="4" x2="16" y2="16"/>
                <line x1="16" y1="4" x2="4" y2="16"/>
              </svg>
            </button>
          </div>
          <div class="settings-content">
            <!-- 服务器控制 -->
            <div class="setting-item server-control">
              <div class="setting-info">
                <div class="setting-title">图片资源服务</div>
                <div class="setting-desc">
                  <template v-if="serverStatus.is_running">
                    运行中 · 端口: {{ serverStatus.port || '-' }}
                  </template>
                  <template v-else>
                    已停止
                  </template>
                </div>
              </div>
              <div class="server-buttons">
                <button
                  v-if="!serverStatus.is_running"
                  @click="startServer"
                  class="action-btn btn-start"
                  :disabled="serverOperating"
                >
                  {{ serverOperating ? '启动中...' : '启动' }}
                </button>
                <template v-else>
                  <button
                    @click="stopServer"
                    class="action-btn btn-stop"
                    :disabled="serverOperating"
                  >
                    {{ serverOperating ? '停止中...' : '停止' }}
                  </button>
                  <button
                    @click="restartServer"
                    class="action-btn btn-restart"
                    :disabled="serverOperating"
                  >
                    {{ serverOperating ? '重启中...' : '重启' }}
                  </button>
                </template>
              </div>
            </div>
            
            <!-- 端口配置 -->
            <div class="setting-item port-config">
              <div class="setting-info">
                <div class="setting-title">服务端口</div>
                <div class="setting-desc">自定义图片资源服务端口 (1024-65535)</div>
              </div>
              <div class="port-input-group">
                <input
                  type="number"
                  v-model.number="customPort"
                  min="1024"
                  max="65535"
                  class="port-input"
                  :disabled="portSaving"
                />
                <button
                  @click="saveCustomPort"
                  class="action-btn"
                  :disabled="portSaving || customPort === serverStatus.port"
                >
                  {{ portSaving ? '保存中...' : '保存' }}
                </button>
              </div>
            </div>
            
            <div class="setting-item">
              <div class="setting-info">
                <div class="setting-title">开机自启动</div>
                <div class="setting-desc">系统启动时自动运行草果剪贴板</div>
              </div>
              <button
                @click="toggleAutostart"
                class="toggle-btn"
                :class="{ active: autostartEnabled }"
              >
                <span class="toggle-circle"></span>
              </button>
            </div>
            <div class="setting-item">
              <div class="setting-info">
                <div class="setting-title">清除同步缓存</div>
                <div class="setting-desc">
                  {{ cacheError ? `错误: ${cacheError}` : '清除哈希缓存，解决图片重复同步问题' }}
                </div>
              </div>
              <button
                @click="clearHashCache"
                class="action-btn"
                :class="{ 
                  'btn-loading': clearingCache, 
                  'btn-success': cacheCleared,
                  'btn-error': cacheError 
                }"
                :disabled="clearingCache"
              >
                <span v-if="clearingCache">清除中...</span>
                <span v-else-if="cacheCleared">✓ 已清除</span>
                <span v-else-if="cacheError">重试</span>
                <span v-else>清除</span>
              </button>
            </div>
            <div class="setting-item">
              <div class="setting-info">
                <div class="setting-title">隐私政策</div>
                <div class="setting-desc">查看我们的隐私政策和使用条款</div>
              </div>
              <button @click="privacyPolicy?.openPrivacyPolicy()" class="action-btn">
                查看
              </button>
            </div>
            <div class="setting-item">
              <div class="setting-info">
                <div class="setting-title">图片资源目录</div>
                <div class="setting-desc">打开图片资源存放目录</div>
              </div>
              <button @click="openImagesDirectory" class="action-btn">
                打开
              </button>
            </div>
            
            <!-- 深色模式 -->
            <div class="setting-item">
              <div class="setting-info">
                <div class="setting-title">深色模式</div>
                <div class="setting-desc">切换深色/浅色界面主题</div>
              </div>
              <button
                @click="toggleDarkMode"
                class="toggle-btn"
                :class="{ active: darkMode }"
              >
                <span class="toggle-circle"></span>
              </button>
            </div>
            
            <!-- 历史记录数量 -->
            <div class="setting-item">
              <div class="setting-info">
                <div class="setting-title">历史记录数量</div>
                <div class="setting-desc">限制保存的剪贴板历史条数</div>
              </div>
              <select v-model.number="historyLimit" @change="setHistoryLimit(historyLimit)" class="select-input">
                <option :value="50">50 条</option>
                <option :value="100">100 条</option>
                <option :value="200">200 条</option>
                <option :value="500">500 条</option>
              </select>
            </div>
            
            <!-- 数据导出 -->
            <div class="setting-item">
              <div class="setting-info">
                <div class="setting-title">数据导出</div>
                <div class="setting-desc">导出剪贴板历史记录备份</div>
              </div>
              <button @click="exportData" class="action-btn">
                导出
              </button>
            </div>
          </div>
        </div>
      </div>
    </Transition>

    <!-- 配对请求弹窗 -->
    <PairRequestDialog />

    <!-- 隐私政策弹窗 -->
    <PrivacyPolicy ref="privacyPolicy" />

    <!-- 欢迎引导 -->
    <WelcomeGuide ref="welcomeGuide" />

    <!-- 清空确认弹窗 -->
    <Transition name="modal">
      <div v-if="showClearConfirm" class="modal-overlay" @click.self="showClearConfirm = false">
        <div class="modal-content confirm-modal">
          <div class="confirm-content">
            <div class="confirm-icon">
              <svg width="48" height="48" viewBox="0 0 48 48" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d="M2.5 15h43M11 30v3M37 30v3M10 15l1 19.5A2 2 0 0 0 13 36.5h22a2 2 0 0 0 2-2L38 15"/>
              </svg>
            </div>
            <h3 class="confirm-title">清空所有记录</h3>
            <p class="confirm-message">
              确定要清空所有剪贴板记录吗？<br>
              <span class="confirm-warn">此操作不可撤销</span>
            </p>
            <div class="confirm-actions">
              <button @click="showClearConfirm = false" class="btn btn-cancel">
                取消
              </button>
              <button @click="clearAll" class="btn btn-danger">
                确认清空
              </button>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

.app {
  width: 100%;
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: transparent;
  font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Display', 'Segoe UI', sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  overflow: hidden;
  position: relative;
}

.app::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(242, 242, 247, 0.95);
  backdrop-filter: blur(20px) saturate(180%);
  border-radius: 12px;
  z-index: -1;
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.15);
}

/* 顶部工具栏 */
.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  background: rgba(255, 255, 255, 0.9);
  border-bottom: 1px solid rgba(0, 0, 0, 0.08);
  flex-shrink: 0;
  height: 52px;
  user-select: none;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 0 0 auto;
}

.header-center {
  flex: 1;
  display: flex;
  justify-content: center;
  align-items: center;
  min-width: 0;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 0 0 auto;
}

.titlebar-drag-region {
  flex: 1;
  height: 100%;
  min-width: 0;
}

.app-title {
  font-size: 15px;
  font-weight: 600;
  color: #1c1c1e;
}

.toolbar-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 12px;
  border: none;
  background: rgba(0, 0, 0, 0.05);
  border-radius: 8px;
  cursor: pointer;
  color: #86868b;
  transition: all 0.2s ease;
  font-size: 13px;
  -webkit-app-region: no-drag;
  pointer-events: auto;
}

.toolbar-btn:hover {
  cursor: pointer;
}

.toolbar-btn:active {
  cursor: pointer;
}

.toolbar-btn:hover {
  background: rgba(0, 122, 255, 0.1);
  color: #007aff;
}

.btn-label {
  font-size: 12px;
  font-weight: 500;
}

/* 主内容区 */
.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-width: 0;
}

.search-container {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 20px;
  background: rgba(255, 255, 255, 0.8);
  border-bottom: 1px solid rgba(0, 0, 0, 0.08);
  flex-shrink: 0;
}

.search-icon {
  color: #86868b;
  flex-shrink: 0;
}

.search-input {
  flex: 1;
  border: none;
  background: transparent;
  font-size: 15px;
  color: #1c1c1e;
  outline: none;
  height: 28px;
  line-height: 28px;
  -webkit-app-region: no-drag;
  pointer-events: auto;
}

.search-input::placeholder {
  color: #86868b;
}

.clear-btn {
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
  -webkit-app-region: no-drag;
  flex-shrink: 0;
  pointer-events: auto;
}

.clear-btn:hover {
  cursor: pointer;
}

.clear-btn:hover {
  background: rgba(0, 0, 0, 0.08);
  color: #1c1c1e;
}

/* Tab切换 */
.tabs-container {
  display: flex;
  gap: 8px;
  padding: 8px 20px 12px;
  background: rgba(255, 255, 255, 0.8);
  flex-shrink: 0;
}

.tab-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 14px;
  border: none;
  background: rgba(0, 0, 0, 0.05);
  border-radius: 8px;
  cursor: pointer;
  color: #86868b;
  font-size: 13px;
  font-weight: 500;
  transition: all 0.2s ease;
  -webkit-app-region: no-drag;
}

.tab-btn:hover {
  background: rgba(0, 0, 0, 0.08);
  color: #1c1c1e;
}

.tab-btn.active {
  background: #007aff;
  color: white;
}

.tab-count {
  font-size: 11px;
  padding: 1px 6px;
  border-radius: 10px;
  background: rgba(0, 0, 0, 0.08);
  font-weight: 600;
}

.tab-btn.active .tab-count {
  background: rgba(255, 255, 255, 0.25);
}

.list {
  flex: 1;
  overflow-y: auto;
  padding: 8px 20px 20px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.list::-webkit-scrollbar {
  width: 8px;
}

.list::-webkit-scrollbar-track {
  background: transparent;
}

.list::-webkit-scrollbar-thumb {
  background: rgba(0, 0, 0, 0.2);
  border-radius: 4px;
}

.list::-webkit-scrollbar-thumb:hover {
  background: rgba(0, 0, 0, 0.3);
}

.empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 200px;
  color: #86868b;
  font-size: 15px;
  font-weight: 400;
}

.item {
  background: rgba(255, 255, 255, 0.7);
  border-radius: 12px;
  padding: 14px 16px;
  cursor: pointer;
  transition: all 0.2s ease;
  border: 1px solid transparent;
}

.item:hover {
  background: rgba(255, 255, 255, 0.95);
  transform: scale(1.01);
}

.item.selected {
  background: rgba(255, 255, 255, 0.95);
  border-color: #007aff;
  box-shadow: 0 0 0 3px rgba(0, 122, 255, 0.1);
}

.item-main {
  min-width: 0;
}

.item-header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 8px;
}

.item-time {
  font-size: 11px;
  color: #86868b;
  font-weight: 500;
}

.item-type {
  font-size: 10px;
  padding: 2px 7px;
  border-radius: 8px;
  font-weight: 500;
  flex-shrink: 0;
}

.item-type:nth-last-child(2) {
  background: rgba(0, 122, 255, 0.1);
  color: #007aff;
}

.item-actions {
  display: flex;
  gap: 4px;
  align-items: center;
}

.favorite-btn {
  width: 22px;
  height: 22px;
  border: none;
  background: transparent;
  border-radius: 6px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #86868b;
  opacity: 0.5;
  transition: all 0.2s ease;
  flex-shrink: 0;
}

.item:hover .favorite-btn {
  opacity: 1;
}

.favorite-btn:hover {
  color: #ff9500;
}

.favorite-btn.active {
  color: #ff9500;
  opacity: 1;
}

.favorite-btn.active svg {
  fill: #ff9500;
}

.save-btn {
  width: 22px;
  height: 22px;
  border: none;
  background: rgba(52, 199, 89, 0.1);
  border-radius: 6px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #34c759;
  opacity: 0;
  transition: all 0.2s ease;
  flex-shrink: 0;
}

.save-btn {
  width: 22px;
  height: 22px;
  border: none;
  background: rgba(52, 199, 89, 0.1);
  border-radius: 6px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #34c759;
  opacity: 0;
  transition: all 0.2s ease;
  flex-shrink: 0;
}

.item:hover .save-btn {
  opacity: 1;
}

.save-btn:hover {
  background: rgba(52, 199, 89, 0.2);
  color: #34c759;
}

.delete-btn {
  width: 22px;
  height: 22px;
  border: none;
  background: rgba(0, 0, 0, 0.05);
  border-radius: 6px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #86868b;
  opacity: 0;
  transition: all 0.2s ease;
  flex-shrink: 0;
  -webkit-app-region: no-drag;
}

.item:hover .delete-btn {
  opacity: 1;
}

.delete-btn:hover {
  background: #ff3b30;
  color: white;
}

.item-text {
  color: #1c1c1e;
  font-size: 14px;
  line-height: 1.5;
  word-break: break-word;
  max-height: 84px;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 4;
  line-clamp: 4;
  -webkit-box-orient: vertical;
}

.item-image {
  width: 100%;
  height: auto;
  max-height: 200px;
  overflow: hidden;
  border-radius: 8px;
  background: linear-gradient(135deg, #f5f5f5 0%, #e8e8e8 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 80px;
}

.item-image img {
  max-width: 100%;
  max-height: 200px;
  object-fit: contain;
  display: block;
}

.image-error {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 20px;
  color: #ff3b30;
  font-size: 12px;
}

.image-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 80px;
  color: #86868b;
  font-size: 12px;
}

.footer {
  padding: 10px 20px;
  background: rgba(255, 255, 255, 0.8);
  border-top: 1px solid rgba(0, 0, 0, 0.08);
  display: flex;
  justify-content: center;
  flex-shrink: 0;
}

.hint {
  color: #86868b;
  font-size: 11px;
  font-weight: 500;
  letter-spacing: 0.3px;
}

/* 模态框 */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.4);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
}

.modal-content {
  background: white;
  border-radius: 16px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.15);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  max-height: 80vh;
  max-width: 90vw;
}

.devices-modal {
  width: 480px;
}

.files-modal {
  width: 520px;
}

.settings-modal {
  width: 400px;
}

.settings-content {
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  overflow-y: auto;
  flex: 1;
  min-height: 0;
}

.setting-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 0;
  border-bottom: 1px solid rgba(0, 0, 0, 0.05);
}

.setting-item:last-child {
  border-bottom: none;
}

.setting-info {
  flex: 1;
}

.setting-title {
  font-size: 15px;
  font-weight: 500;
  color: #1c1c1e;
  margin-bottom: 4px;
}

.setting-desc {
  font-size: 13px;
  color: #86868b;
}

.toggle-btn {
  width: 48px;
  height: 28px;
  border-radius: 14px;
  background: rgba(0, 0, 0, 0.1);
  border: none;
  cursor: pointer;
  padding: 3px;
  transition: all 0.2s ease;
  -webkit-app-region: no-drag;
}

.toggle-btn.active {
  background: #34c759;
}

.toggle-circle {
  width: 22px;
  height: 22px;
  border-radius: 50%;
  background: white;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.12);
  transition: all 0.2s ease;
}

.toggle-btn.active .toggle-circle {
  transform: translateX(20px);
}

.action-btn {
  padding: 8px 16px;
  border: none;
  background: rgba(0, 122, 255, 0.1);
  border-radius: 8px;
  cursor: pointer;
  color: #007aff;
  font-size: 13px;
  font-weight: 500;
  transition: all 0.2s ease;
  -webkit-app-region: no-drag;
  min-width: 60px;
}

.action-btn:hover {
  background: rgba(0, 122, 255, 0.2);
  color: #0066cc;
}

.action-btn:active {
  transform: scale(0.98);
}

.select-input {
  padding: 8px 12px;
  border: 1px solid rgba(0, 0, 0, 0.1);
  border-radius: 8px;
  font-size: 13px;
  color: #1c1c1e;
  background: rgba(255, 255, 255, 0.8);
  cursor: pointer;
  transition: all 0.2s ease;
  -webkit-app-region: no-drag;
}

.select-input:focus {
  outline: none;
  border-color: #007aff;
  box-shadow: 0 0 0 3px rgba(0, 122, 255, 0.1);
}

/* 深色模式样式 */
.dark-mode .app::before {
  background: rgba(28, 28, 30, 0.95) !important;
}

.dark-mode .header {
  background: rgba(44, 44, 46, 0.9) !important;
  border-bottom-color: rgba(255, 255, 255, 0.1) !important;
}

.dark-mode .app-title {
  color: #fff !important;
}

.dark-mode .search-container {
  background: rgba(44, 44, 46, 0.8) !important;
  border-bottom-color: rgba(255, 255, 255, 0.1) !important;
}

.dark-mode .search-input {
  color: #fff !important;
}

.dark-mode .search-input::placeholder {
  color: rgba(255, 255, 255, 0.5) !important;
}

.dark-mode .item {
  background: rgba(58, 58, 60, 0.7) !important;
  border-color: transparent !important;
}

.dark-mode .item:hover {
  background: rgba(72, 72, 74, 0.95) !important;
}

.dark-mode .item.selected {
  background: rgba(72, 72, 74, 0.95) !important;
  border-color: #007aff !important;
}

.dark-mode .item-time {
  color: rgba(255, 255, 255, 0.6) !important;
}

.dark-mode .item-text {
  color: #fff !important;
}

.dark-mode .footer {
  background: rgba(44, 44, 46, 0.8) !important;
  border-top-color: rgba(255, 255, 255, 0.1) !important;
}

.dark-mode .hint {
  color: rgba(255, 255, 255, 0.4) !important;
}

.dark-mode .modal-content {
  background: #2c2c2e !important;
}

.dark-mode .modal-header {
  border-bottom-color: rgba(255, 255, 255, 0.1) !important;
}

.dark-mode .modal-title {
  color: #fff !important;
}

.dark-mode .modal-close {
  background: rgba(255, 255, 255, 0.1) !important;
  color: rgba(255, 255, 255, 0.6) !important;
}

.dark-mode .modal-close:hover {
  background: rgba(255, 255, 255, 0.15) !important;
  color: #fff !important;
}

.dark-mode .setting-title {
  color: #fff !important;
}

.dark-mode .setting-desc {
  color: rgba(255, 255, 255, 0.5) !important;
}

.dark-mode .setting-item {
  border-bottom-color: rgba(255, 255, 255, 0.1) !important;
}

.dark-mode .select-input {
  background: rgba(58, 58, 60, 0.8) !important;
  border-color: rgba(255, 255, 255, 0.2) !important;
  color: #fff !important;
}

.dark-mode .clear-btn {
  background: rgba(255, 255, 255, 0.1) !important;
  color: rgba(255, 255, 255, 0.6) !important;
}

.dark-mode .clear-btn:hover {
  background: rgba(255, 255, 255, 0.15) !important;
  color: #fff !important;
}

/* Tab深色模式 */
.dark-mode .tabs-container {
  background: rgba(44, 44, 46, 0.8) !important;
}

.dark-mode .tab-btn {
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.6);
}

.dark-mode .tab-btn:hover {
  background: rgba(255, 255, 255, 0.15);
  color: #fff;
}

.dark-mode .tab-btn.active {
  background: #007aff;
  color: white;
}

.dark-mode .tab-count {
  background: rgba(255, 255, 255, 0.15);
}

/* 设备管理面板深色模式 */
.dark-mode .device-panel,
.dark-mode .file-transfer {
  background: rgba(28, 28, 30, 0.95) !important;
}

.dark-mode .device-panel .panel-header,
.dark-mode .file-transfer .panel-header {
  background: rgba(44, 44, 46, 0.9) !important;
  border-bottom-color: rgba(255, 255, 255, 0.1) !important;
}

.dark-mode .device-panel .panel-title,
.dark-mode .file-transfer .panel-title {
  color: #fff !important;
}

.dark-mode .device-count {
  color: rgba(255, 255, 255, 0.5) !important;
}

.dark-mode .device-card,
.dark-mode .transfer-item,
.dark-mode .file-item {
  background: rgba(58, 58, 60, 0.7) !important;
}

.dark-mode .device-card:hover,
.dark-mode .transfer-item:hover,
.dark-mode .file-item:hover {
  background: rgba(72, 72, 74, 0.95) !important;
}

.dark-mode .device-name,
.dark-mode .transfer-file,
.dark-mode .file-name {
  color: #fff !important;
}

.dark-mode .device-ip,
.dark-mode .device-last-seen,
.dark-mode .progress-text,
.dark-mode .file-size,
.dark-mode .file-time {
  color: rgba(255, 255, 255, 0.5) !important;
}

.dark-mode .empty-text,
.dark-mode .section-title {
  color: rgba(255, 255, 255, 0.5) !important;
}

.dark-mode .btn-secondary {
  background: rgba(255, 255, 255, 0.1) !important;
  color: #fff !important;
}

.dark-mode .btn-secondary:hover {
  background: rgba(255, 255, 255, 0.15) !important;
}

.dark-mode .btn-open,
.dark-mode .btn-close,
.dark-mode .btn-clear {
  background: rgba(255, 255, 255, 0.1) !important;
  color: rgba(255, 255, 255, 0.6) !important;
}

.dark-mode .btn-open:hover,
.dark-mode .btn-close:hover {
  background: rgba(255, 255, 255, 0.15) !important;
  color: #fff !important;
}

.dark-mode .selected-file {
  background: rgba(0, 122, 255, 0.2) !important;
}

.dark-mode .selected-file .label {
  color: rgba(255, 255, 255, 0.5) !important;
}

.dark-mode .no-devices {
  color: rgba(255, 255, 255, 0.5) !important;
}

.dark-mode .device-item {
  background: rgba(255, 255, 255, 0.05) !important;
}

.dark-mode .device-item:hover {
  background: rgba(0, 122, 255, 0.2) !important;
}

.dark-mode .modal {
  background: rgba(44, 44, 46, 0.95) !important;
}

.dark-mode .modal-header {
  border-bottom-color: rgba(255, 255, 255, 0.1) !important;
}

.dark-mode .modal-title {
  color: #fff !important;
}

/* 列表项图片预览深色模式 */
.dark-mode .item-image {
  background: rgba(58, 58, 60, 0.5) !important;
}

.dark-mode .item-image img {
  border-color: rgba(255, 255, 255, 0.1) !important;
}

.action-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.action-btn.btn-loading {
  background: rgba(0, 122, 255, 0.15);
  color: #007aff;
  cursor: wait;
}

.action-btn.btn-success {
  background: rgba(52, 199, 89, 0.15);
  color: #34c759;
}

.action-btn.btn-error {
  background: rgba(255, 59, 48, 0.15);
  color: #ff3b30;
}

.action-btn.btn-error:hover {
  background: rgba(255, 59, 48, 0.2);
  color: #d70015;
}

/* 服务器控制样式 */
.server-control {
  flex-direction: column;
  align-items: flex-start;
  gap: 12px;
}

.server-buttons {
  display: flex;
  gap: 8px;
  width: 100%;
}

.server-buttons .action-btn {
  flex: 1;
}

.btn-start {
  background: rgba(52, 199, 89, 0.15);
  color: #34c759;
}

.btn-start:hover {
  background: rgba(52, 199, 89, 0.25);
}

.btn-stop {
  background: rgba(255, 59, 48, 0.15);
  color: #ff3b30;
}

.btn-stop:hover {
  background: rgba(255, 59, 48, 0.25);
}

.btn-restart {
  background: rgba(255, 149, 0, 0.15);
  color: #ff9500;
}

.btn-restart:hover {
  background: rgba(255, 149, 0, 0.25);
}

/* 端口配置样式 */
.port-config {
  flex-direction: column;
  align-items: flex-start;
  gap: 12px;
}

.port-input-group {
  display: flex;
  gap: 8px;
  width: 100%;
}

.port-input {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid rgba(0, 0, 0, 0.1);
  border-radius: 8px;
  font-size: 14px;
  color: #1c1c1e;
  background: rgba(255, 255, 255, 0.8);
  transition: all 0.2s ease;
  -webkit-app-region: no-drag;
}

.port-input:focus {
  outline: none;
  border-color: #007aff;
  box-shadow: 0 0 0 3px rgba(0, 122, 255, 0.1);
}

.port-input:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.port-input-group .action-btn {
  min-width: 60px;
}

.toggle-btn.active .toggle-circle {
  transform: translateX(20px);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid rgba(0, 0, 0, 0.08);
  flex-shrink: 0;
}

.modal-title {
  font-size: 16px;
  font-weight: 600;
  color: #1c1c1e;
}

.modal-close {
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
}

.modal-close:hover {
  background: rgba(0, 0, 0, 0.1);
  color: #1c1c1e;
}

/* Modal transitions */
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.2s ease;
}

/* 确认弹窗 */
.confirm-modal {
  width: 380px;
}

.confirm-content {
  padding: 32px 24px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
  text-align: center;
}

.confirm-icon {
  width: 64px;
  height: 64px;
  border-radius: 50%;
  background: rgba(255, 59, 48, 0.1);
  display: flex;
  align-items: center;
  justify-content: center;
  color: #ff3b30;
}

.confirm-icon svg {
  flex-shrink: 0;
}

.confirm-title {
  font-size: 18px;
  font-weight: 600;
  color: #1c1c1e;
  margin: 0;
}

.confirm-message {
  font-size: 14px;
  color: #86868b;
  line-height: 1.5;
  margin: 0;
}

.confirm-warn {
  color: #ff3b30;
  font-weight: 500;
}

.confirm-actions {
  display: flex;
  gap: 12px;
  width: 100%;
  margin-top: 8px;
}

.confirm-actions .btn {
  flex: 1;
  padding: 12px 20px;
  border-radius: 10px;
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
  border: none;
  -webkit-app-region: no-drag;
}

.btn-cancel {
  background: rgba(0, 0, 0, 0.08);
  color: #1c1c1e;
}

.btn-cancel:hover {
  background: rgba(0, 0, 0, 0.12);
}

.btn-danger {
  background: #ff3b30;
  color: white;
}

.btn-danger:hover {
  background: #d70015;
}


.modal-enter-active .modal-content,
.modal-leave-active .modal-content {
  transition: transform 0.2s ease, opacity 0.2s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from .modal-content,
.modal-leave-to .modal-content {
  transform: scale(0.95);
  opacity: 0;
}

/* 响应式 */
@media (max-width: 768px) {
  .devices-modal {
    width: 100%;
    max-width: 100%;
  }

  .files-modal {
    width: 100%;
    max-width: 100%;
  }
}
</style>
