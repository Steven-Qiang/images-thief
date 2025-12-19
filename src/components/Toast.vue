<template>
  <Teleport to="body">
    <div class="toast-container">
      <div v-for="toast in toasts" :key="toast.id" class="toast-item" :class="toast.type">
        <div class="toast-icon">
          <span v-if="toast.type === 'success'">✅</span>
          <span v-else-if="toast.type === 'error'">❌</span>
          <span v-else-if="toast.type === 'warning'">⚠️</span>
          <span v-else>ℹ️</span>
        </div>
        <div class="toast-message">
          {{ toast.message }}
        </div>
        <button class="toast-close" @click="removeToast(toast.id)">
          ✕
        </button>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref } from 'vue';

interface Toast {
  id: string;
  type: 'success' | 'error' | 'warning' | 'info';
  message: string;
}

const toasts = ref<Toast[]>([]);

function addToast(type: Toast['type'], message: string) {
  const id = Date.now().toString();
  toasts.value.push({ id, type, message });

  setTimeout(() => {
    removeToast(id);
  }, 5000);
}

function removeToast(id: string) {
  const index = toasts.value.findIndex((t) => t.id === id);
  if (index > -1) {
    toasts.value.splice(index, 1);
  }
}

defineExpose({ addToast });
</script>

<style scoped>
.toast-container {
  position: fixed;
  top: 16px;
  right: 16px;
  z-index: 1000;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.toast-item {
  padding: 12px 16px;
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 320px;
  max-width: 400px;
  animation: slideIn 0.3s;
  color: #fff;
}

@keyframes slideIn {
  from {
    transform: translateX(100%);
    opacity: 0;
  }
  to {
    transform: translateX(0);
    opacity: 1;
  }
}

.toast-item.success {
  background: #388e3c;
}
.toast-item.error {
  background: #d32f2f;
}
.toast-item.warning {
  background: #f57c00;
}
.toast-item.info {
  background: #1976d2;
}

.toast-icon {
  flex-shrink: 0;
}

.toast-message {
  flex: 1;
  font-size: 13px;
}

.toast-close {
  background: none;
  border: none;
  color: rgba(255, 255, 255, 0.8);
  cursor: pointer;
  font-size: 18px;
  padding: 0;
  line-height: 1;
  flex-shrink: 0;
}

.toast-close:hover {
  color: #fff;
}
</style>
