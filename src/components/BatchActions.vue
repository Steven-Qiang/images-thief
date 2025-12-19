<template>
  <div class="batch-card">
    <div class="batch-content">
      <div class="batch-left">
        <label class="select-label">
          <input type="checkbox" :checked="isAllSelected" :indeterminate="isIndeterminate" @change="toggleSelectAll">
          <span>{{ selectedCount > 0 ? `已选择 ${selectedCount} 项` : '全选' }}</span>
        </label>

        <div class="quick-filters">
          <button class="filter-btn failed" @click="selectByStatus('Failed')">
            选择失败项
          </button>
          <button class="filter-btn completed" @click="selectByStatus('Completed')">
            选择完成项
          </button>
        </div>
      </div>

      <div v-if="selectedCount > 0" class="batch-actions">
        <button class="action-btn primary" @click="$emit('batch-retry')">
          重新下载 ({{ selectedCount }})
        </button>
        <button class="action-btn danger" @click="$emit('batch-delete')">
          删除记录 ({{ selectedCount }})
        </button>
        <button class="action-btn success" @click="$emit('batch-export')">
          导出选中
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

interface DownloadItem {
  id: string;
  status: string;
}

const props = defineProps<{
  items: DownloadItem[];
  selectedIds: string[];
}>();

const emit = defineEmits<{
  'update-selection': [ids: string[]];
  'batch-retry': [];
  'batch-delete': [];
  'batch-export': [];
}>();

const selectedCount = computed(() => props.selectedIds.length);
const isAllSelected = computed(() =>
  props.items.length > 0 && props.selectedIds.length === props.items.length
);
const isIndeterminate = computed(() =>
  props.selectedIds.length > 0 && props.selectedIds.length < props.items.length
);

function toggleSelectAll() {
  if (isAllSelected.value) {
    emit('update-selection', []);
  } else {
    emit('update-selection', props.items.map((item) => item.id));
  }
}

function selectByStatus(status: string) {
  const ids = props.items
    .filter((item) => item.status === status)
    .map((item) => item.id);
  emit('update-selection', ids);
}
</script>

<style scoped>
.batch-card {
  background: #fff;
  border: 1px solid #e0e0e0;
  border-radius: 6px;
  padding: 16px;
  margin-bottom: 24px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08);
}

.batch-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.batch-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.select-label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  cursor: pointer;
}

.select-label input[type='checkbox'] {
  width: 16px;
  height: 16px;
  cursor: pointer;
  accent-color: #1976d2;
}

.quick-filters {
  display: flex;
  gap: 8px;
}

.filter-btn {
  padding: 6px 12px;
  border: none;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
}

.filter-btn.failed {
  background: #ffebee;
  color: #c62828;
}

.filter-btn.failed:hover {
  background: #ffcdd2;
}

.filter-btn.completed {
  background: #e8f5e9;
  color: #2e7d32;
}

.filter-btn.completed:hover {
  background: #c8e6c9;
}

.batch-actions {
  display: flex;
  gap: 8px;
}

.action-btn {
  padding: 8px 12px;
  border: none;
  border-radius: 4px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
}

.action-btn.primary {
  background: #1976d2;
  color: #fff;
}

.action-btn.primary:hover {
  background: #1565c0;
}

.action-btn.danger {
  background: #d32f2f;
  color: #fff;
}

.action-btn.danger:hover {
  background: #c62828;
}

.action-btn.success {
  background: #388e3c;
  color: #fff;
}

.action-btn.success:hover {
  background: #2e7d32;
}
</style>
