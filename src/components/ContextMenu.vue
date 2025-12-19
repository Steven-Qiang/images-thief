<template>
  <div v-if="visible" class="context-menu" :style="{ top: `${y}px`, left: `${x}px` }" @click.stop>
    <button v-for="item in menuItems" :key="item.label" :disabled="item.disabled" class="menu-item" @click="handleClick(item)">
      <span class="menu-icon">{{ item.icon }}</span>
      <span class="menu-label">{{ item.label }}</span>
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';

interface MenuItem {
  label: string;
  icon: string;
  action: string;
  disabled?: boolean;
}

const emit = defineEmits<{
  action: [action: string, item: any];
}>();
const visible = ref(false);
const x = ref(0);
const y = ref(0);
const menuItems = ref<MenuItem[]>([]);
const currentItem = ref<any>(null);

function show(event: MouseEvent, item: any, items: MenuItem[]) {
  event.preventDefault();
  x.value = event.clientX;
  y.value = event.clientY;
  currentItem.value = item;
  menuItems.value = items;
  visible.value = true;

  document.addEventListener('click', hide);
}

function hide() {
  visible.value = false;
  document.removeEventListener('click', hide);
}

function handleClick(menuItem: MenuItem) {
  if (!menuItem.disabled) {
    emit('action', menuItem.action, currentItem.value);
  }
  hide();
}

defineExpose({ show, hide });
</script>

<style scoped>
.context-menu {
  position: fixed;
  z-index: 1000;
  background: #fff;
  border: 1px solid #e0e0e0;
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  padding: 8px 0;
  min-width: 180px;
}

.menu-item {
  width: 100%;
  padding: 10px 16px;
  border: none;
  background: none;
  text-align: left;
  font-size: 13px;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 8px;
  transition: background 0.15s;
}

.menu-item:hover:not(:disabled) {
  background: #f5f5f5;
}

.menu-item:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.menu-icon {
  flex-shrink: 0;
}

.menu-label {
  flex: 1;
}
</style>
