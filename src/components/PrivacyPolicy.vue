<script setup lang="ts">
import { ref } from 'vue'

const showPrivacyPolicy = ref(false)

function openPrivacyPolicy() {
  showPrivacyPolicy.value = true
}

function closePrivacyPolicy() {
  showPrivacyPolicy.value = false
}

defineExpose({
  openPrivacyPolicy
})
</script>

<template>
  <Transition name="modal">
    <div v-if="showPrivacyPolicy" class="modal-overlay" @click.self="closePrivacyPolicy">
      <div class="modal-content privacy-modal">
        <div class="modal-header">
          <h2 class="modal-title">隐私政策</h2>
          <button @click="closePrivacyPolicy" class="modal-close">
            <svg width="20" height="20" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="4" y1="4" x2="16" y2="16"/>
              <line x1="16" y1="4" x2="4" y2="16"/>
            </svg>
          </button>
        </div>

        <div class="modal-body">
          <div class="privacy-content">
            <h3>1. 数据收集</h3>
            <p>本应用不会收集任何个人数据。所有剪贴板内容仅存储在您的本地设备上，不会上传到任何服务器。</p>

            <h3>2. 数据存储</h3>
            <p>剪贴板数据存储在您设备的本地 SQLite 数据库中，只有您本人可以访问。数据文件位置：</p>
            <code>~/Library/Application Support/com.yangtanfang.tie-lifang/clipboard.db</code>

            <h3>3. 数据删除</h3>
            <p>您可以通过以下方式删除数据：</p>
            <ul>
              <li>在应用内点击"清空所有记录"按钮删除所有历史剪贴板记录</li>
              <li>删除单个剪贴板记录</li>
              <li>卸载应用后，所有数据也会被删除</li>
            </ul>

            <h3>4. 数据共享</h3>
            <p>本应用不会与任何第三方共享您的数据。仅在您主动启用局域网同步功能时，数据会在您信任的设备之间直接传输，不会经过任何服务器。</p>

            <h3>5. 权限使用说明</h3>
            <ul>
              <li><strong>剪贴板访问权限</strong>：用于监听和管理您的剪贴板内容</li>
              <li><strong>网络访问权限</strong>：用于局域网设备发现和数据同步</li>
              <li><strong>文件访问权限</strong>：用于保存图片和接收文件到下载目录</li>
            </ul>

            <h3>6. 局域网同步说明</h3>
            <p>当您启用局域网同步功能时：</p>
            <ul>
              <li>数据仅在您信任的设备之间传输</li>
              <li>使用 mDNS 协议发现局域网设备</li>
              <li>所有数据传输都在局域网内进行，不会经过互联网</li>
              <li>您可以随时移除已信任的设备</li>
            </ul>

            <h3>7. 数据安全</h3>
            <ul>
              <li>所有数据存储在本地，离线也可使用</li>
              <li>局域网传输使用点对点直接连接</li>
              <li>配对过程使用设备 ID 和配对令牌进行验证</li>
              <li>您可以随时清除同步缓存和信任设备列表</li>
            </ul>

            <h3>8. 儿童隐私</h3>
            <p>本应用不针对儿童设计，也不会故意收集任何儿童的个人信息。</p>

            <h3>9. 隐私政策更新</h3>
            <p>我们可能会不时更新本隐私政策。更新后的政策将在应用中发布，请您定期查看。</p>

            <h3>10. 联系我们</h3>
            <p>如果您对本隐私政策有任何疑问或建议，请通过以下方式联系我们：</p>
            <ul>
              <li>邮箱：uutan@qq.com</li>
              <li>GitHub：https://github.com/uutanyang/tie-lifang/issues</li>
            </ul>

            <div class="policy-footer">
              <p>最后更新时间：2026年2月</p>
              <p>本政策自发布之日起生效</p>
            </div>
          </div>
        </div>

        <div class="modal-footer">
          <button @click="closePrivacyPolicy" class="btn btn-primary">
            我已了解
          </button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
/* 模态框背景遮罩 */
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

/* 模态框内容容器 */
.modal-content {
  background: white;
  border-radius: 16px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.15);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  animation: modalSlideIn 0.3s ease;
}

@keyframes modalSlideIn {
  from {
    transform: scale(0.95);
    opacity: 0;
  }
  to {
    transform: scale(1);
    opacity: 1;
  }
}

.privacy-modal {
  width: 600px;
  max-width: 90vw;
  max-height: 80vh;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 24px;
  border-bottom: 1px solid rgba(0, 0, 0, 0.08);
  flex-shrink: 0;
}

.modal-title {
  font-size: 18px;
  font-weight: 600;
  color: #1c1c1e;
  margin: 0;
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

.modal-body {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
}

.privacy-content {
  color: #1c1c1e;
  line-height: 1.6;
}

.privacy-content h3 {
  font-size: 16px;
  font-weight: 600;
  color: #1c1c1e;
  margin: 20px 0 8px 0;
  padding-bottom: 4px;
  border-bottom: 1px solid rgba(0, 0, 0, 0.08);
}

.privacy-content h3:first-child {
  margin-top: 0;
}

.privacy-content p {
  font-size: 14px;
  color: #3a3a3c;
  margin: 8px 0;
}

.privacy-content ul {
  margin: 8px 0;
  padding-left: 20px;
}

.privacy-content li {
  font-size: 14px;
  color: #3a3a3c;
  margin: 4px 0;
}

.privacy-content code {
  display: block;
  background: rgba(0, 0, 0, 0.05);
  padding: 8px 12px;
  border-radius: 6px;
  font-size: 12px;
  color: #007aff;
  font-family: 'Monaco', 'Menlo', 'Courier New', monospace;
  margin: 8px 0;
  word-break: break-all;
}

.privacy-content strong {
  color: #1c1c1e;
  font-weight: 600;
}

.policy-footer {
  margin-top: 24px;
  padding-top: 16px;
  border-top: 1px solid rgba(0, 0, 0, 0.08);
}

.policy-footer p {
  font-size: 12px;
  color: #86868b;
  margin: 4px 0;
}

.modal-footer {
  padding: 16px 24px;
  border-top: 1px solid rgba(0, 0, 0, 0.08);
  display: flex;
  justify-content: flex-end;
  flex-shrink: 0;
}

.btn {
  padding: 10px 20px;
  border: none;
  border-radius: 8px;
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-primary {
  background: #007aff;
  color: white;
}

.btn-primary:hover {
  background: #0066cc;
}

/* Modal transitions */
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.2s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

/* Scrollbar styling */
.modal-body::-webkit-scrollbar {
  width: 8px;
}

.modal-body::-webkit-scrollbar-track {
  background: transparent;
}

.modal-body::-webkit-scrollbar-thumb {
  background: rgba(0, 0, 0, 0.2);
  border-radius: 4px;
}

.modal-body::-webkit-scrollbar-thumb:hover {
  background: rgba(0, 0, 0, 0.3);
}
</style>
