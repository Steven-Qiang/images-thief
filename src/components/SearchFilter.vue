<template>
  <div class="filter-card">
    <div class="filter-content">
      <div class="search-box">
        <input v-model="searchQuery" type="text" placeholder="搜索文件名..." class="filter-input">
      </div>

      <select v-model="statusFilter" class="filter-select">
        <option value="">
          全部状态
        </option>
        <option
          v-for="option in statusOptions"
          :key="option.value"
          :value="option.value"
        >
          {{ option.label }}
        </option>
      </select>

      <select v-model="sortBy" class="filter-select">
        <option value="start_time">
          按时间
        </option>
        <option value="filename">
          按文件名
        </option>
        <option value="size">
          按大小
        </option>
        <option value="status">
          按状态
        </option>
      </select>

      <button class="sort-btn" @click="toggleSortOrder">
        {{ sortOrder === 'asc' ? '↑' : '↓' }}
      </button>

      <button class="clear-btn" @click="clearFilters">
        清空
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { DownloadStatus } from '../utils/downloadStatus';
import { ref, watch } from 'vue';
import { DownloadStatusUtils } from '../utils/downloadStatus';

const emit = defineEmits<{
  filter: [filters: {
    search: string;
    status: DownloadStatus | '';
    sortBy: string;
    sortOrder: 'asc' | 'desc';
  }];
}>();
const searchQuery = ref('');
const statusFilter = ref<DownloadStatus | ''>('');
const sortBy = ref('start_time');
const sortOrder = ref<'asc' | 'desc'>('desc');

// 获取状态选项
const statusOptions = DownloadStatusUtils.getOptions();

function emitFilters() {
  emit('filter', {
    search: searchQuery.value,
    status: statusFilter.value,
    sortBy: sortBy.value,
    sortOrder: sortOrder.value
  });
}

function toggleSortOrder() {
  sortOrder.value = sortOrder.value === 'asc' ? 'desc' : 'asc';
}

function clearFilters() {
  searchQuery.value = '';
  statusFilter.value = '';
  sortBy.value = 'start_time';
  sortOrder.value = 'desc';
}

watch([searchQuery, statusFilter, sortBy, sortOrder], emitFilters);
</script>

<style scoped>
.filter-card {
  background: #fff;
  border: 1px solid #e0e0e0;
  border-radius: 6px;
  padding: 16px;
  margin-bottom: 24px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08);
}

.filter-content {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  align-items: center;
}

.search-box {
  flex: 1;
  min-width: 250px;
}

.filter-input,
.filter-select {
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 13px;
  transition:
    border-color 0.2s,
    box-shadow 0.2s;
  font-family: inherit;
  background-color: #fff;
  cursor: pointer;
}

.filter-input {
  width: 100%;
}

.filter-select {
  appearance: none;
  background-image: url("data:image/svg+xml,%3csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 20 20'%3e%3cpath stroke='%236b7280' stroke-linecap='round' stroke-linejoin='round' stroke-width='1.5' d='M6 8l4 4 4-4'/%3e%3c/svg%3e");
  background-position: right 12px center;
  background-repeat: no-repeat;
  background-size: 16px;
  padding-right: 36px;
  min-width: 120px;
}

.filter-input:focus,
.filter-select:focus {
  outline: none;
  border-color: #1976d2;
  box-shadow: 0 0 0 3px rgba(25, 118, 210, 0.1);
}

.filter-select:hover {
  background-color: #f8f8f8;
  border-color: #1976d2;
}

.sort-btn,
.clear-btn {
  padding: 8px 12px;
  border: 1px solid #ddd;
  background: #fff;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
  transition: all 0.15s;
  font-family: inherit;
}

.sort-btn:hover,
.clear-btn:hover {
  background: #f8f8f8;
  border-color: #1976d2;
}

.clear-btn {
  color: #666;
}

.sort-btn {
  min-width: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
}
</style>
