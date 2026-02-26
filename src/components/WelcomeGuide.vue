<script setup lang="ts">
import { ref, onMounted } from 'vue'

const showWelcome = ref(false)
const currentStep = ref(0)
const steps = [
  {
    title: '欢迎使用贴立方',
    description: '一款现代化的剪贴板管理工具，支持文本和图片记录、局域网同步、文件传输等功能。',
    icon: '📋'
  },
  {
    title: '剪贴板权限',
    description: '贴立方需要访问剪贴板来监听和管理您的剪贴板历史记录。所有数据都存储在本地，不会上传到任何服务器。',
    icon: '🔒'
  },
  {
    title: '网络权限',
    description: '贴立方使用局域网网络来发现其他设备并进行剪贴板同步和文件传输。仅在您信任的设备之间传输数据。',
    icon: '🌐'
  },
  {
    title: '文件权限',
    description: '贴立方需要访问下载文件夹来保存接收的文件和图片。您可以随时在设置中管理这些权限。',
    icon: '📁'
  },
  {
    title: '开始使用',
    description: '配置完成！现在您可以开始使用贴立方了。按 Cmd+Shift+V 随时呼出剪贴板面板。',
    icon: '🚀'
  }
]

function nextStep() {
  if (currentStep.value < steps.length - 1) {
    currentStep.value++
  } else {
    closeWelcome()
  }
}

function prevStep() {
  if (currentStep.value > 0) {
    currentStep.value--
  }
}

function closeWelcome() {
  showWelcome.value = false
  // 保存到 localStorage，下次不再显示
  localStorage.setItem('welcome_shown', 'true')
}

onMounted(() => {
  // 检查是否首次启动
  const hasShown = localStorage.getItem('welcome_shown')
  if (!hasShown) {
    showWelcome.value = true
  }
})

defineExpose({
  showWelcome: () => { showWelcome.value = true }
})
</script>

<template>
  <Transition name="modal">
    <div v-if="showWelcome" class="welcome-overlay">
      <div class="welcome-content">
        <div class="welcome-icon">{{ steps[currentStep].icon }}</div>

        <h2 class="welcome-title">{{ steps[currentStep].title }}</h2>

        <p class="welcome-description">{{ steps[currentStep].description }}</p>

        <div class="welcome-steps">
          <div
            v-for="(_step, index) in steps"
            :key="index"
            class="step-dot"
            :class="{ active: index === currentStep, completed: index < currentStep }"
          ></div>
        </div>

        <div class="welcome-actions">
          <button
            v-if="currentStep > 0"
            @click="prevStep"
            class="btn btn-secondary"
          >
            上一步
          </button>
          <button
            @click="nextStep"
            class="btn btn-primary"
          >
            {{ currentStep === steps.length - 1 ? '开始使用' : '下一步' }}
          </button>
        </div>

        <button v-if="currentStep === 0" @click="closeWelcome" class="skip-btn">
          跳过引导
        </button>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.welcome-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
  padding: 20px;
}

.welcome-content {
  background: white;
  border-radius: 20px;
  padding: 40px;
  max-width: 480px;
  width: 100%;
  text-align: center;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
}

.welcome-icon {
  font-size: 64px;
  margin-bottom: 24px;
  animation: bounce 2s infinite;
}

@keyframes bounce {
  0%, 100% {
    transform: translateY(0);
  }
  50% {
    transform: translateY(-10px);
  }
}

.welcome-title {
  font-size: 24px;
  font-weight: 700;
  color: #1c1c1e;
  margin: 0 0 16px 0;
}

.welcome-description {
  font-size: 15px;
  line-height: 1.6;
  color: #3a3a3c;
  margin: 0 0 32px 0;
  min-height: 72px;
}

.welcome-steps {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  margin-bottom: 32px;
}

.step-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #e5e5ea;
  transition: all 0.3s ease;
}

.step-dot.active {
  width: 24px;
  background: #007aff;
}

.step-dot.completed {
  background: #34c759;
}

.welcome-actions {
  display: flex;
  gap: 12px;
  justify-content: center;
}

.btn {
  padding: 12px 28px;
  border: none;
  border-radius: 10px;
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
  min-width: 100px;
}

.btn-primary {
  background: #007aff;
  color: white;
}

.btn-primary:hover {
  background: #0066cc;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 122, 255, 0.3);
}

.btn-secondary {
  background: rgba(0, 0, 0, 0.08);
  color: #1c1c1e;
}

.btn-secondary:hover {
  background: rgba(0, 0, 0, 0.12);
}

.skip-btn {
  margin-top: 20px;
  background: none;
  border: none;
  color: #86868b;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.skip-btn:hover {
  color: #007aff;
}

/* Modal transitions */
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
</style>
