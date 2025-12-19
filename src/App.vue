<template>
  <div class="app">
    <!-- Header -->
    <header class="header">
      <div class="header-content">
        <div>
          <h1 class="header-title">
            Images Thief <span class="header-version">v{{ appVersion }}</span>
          </h1>
          <p class="header-subtitle">
            批量下载随机图片接口的所有图片
          </p>
        </div>
        <div class="header-right">
          <button type="button" class="btn btn-secondary" @click="checkForUpdate">
            检查更新
          </button>
          <div class="status-indicator">
            <div class="status-dot" :class="{ running: status.is_running }" />
            <span>{{ status.is_running ? '运行中' : '已停止' }}</span>
          </div>
        </div>
      </div>
    </header>

    <!-- Main Content -->
    <main class="main">
      <!-- Configuration Form -->
      <div class="card">
        <h2 class="card-title">
          下载配置
        </h2>

        <form @submit.prevent="handleStart">
          <!-- API URL -->
          <div class="form-group">
            <label class="form-label">接口地址</label>
            <input
              v-model="config.api_url"
              type="url"
              required
              class="form-input"
              :class="{ error: config.api_url && !isValidUrl(config.api_url) }"
              placeholder="https://example.com/api/random-image"
            >
            <span v-if="config.api_url && !isValidUrl(config.api_url)" class="form-error">
              请输入有效的URL地址
            </span>
            <span class="form-hint">
              随机图片接口地址，每次请求返回不同的图片
            </span>
          </div>

          <!-- Output Directory -->
          <div class="form-group">
            <label class="form-label">保存目录</label>
            <div class="input-group">
              <input
                v-model="config.output_dir"
                type="text"
                readonly
                class="form-input"
              >
              <button type="button" class="btn btn-secondary" @click="selectDirectory">
                选择
              </button>
            </div>
          </div>

          <!-- Options -->
          <div class="form-row">
            <div>
              <label class="form-label">并发数</label>
              <input v-model.number="config.concurrency" type="number" min="1" max="20" class="form-input">
              <span class="form-hint">同时下载的任务数，建议10-20</span>
            </div>

            <div>
              <label class="form-label">重复阈值</label>
              <input v-model.number="config.max_duplicate" type="number" min="0" class="form-input">
              <span class="form-hint">重复次数超过此值停止，0=无限制</span>
            </div>

            <div>
              <label class="form-label">最大重试次数</label>
              <input v-model.number="config.max_retries" type="number" min="0" max="10" class="form-input">
              <span class="form-hint">下载失败后的重试次数</span>
            </div>

            <div>
              <label class="form-label">重试延迟(秒)</label>
              <input v-model.number="config.retry_delay" type="number" min="1" max="60" class="form-input">
              <span class="form-hint">重试前等待的秒数</span>
            </div>
          </div>

          <!-- Referer URL -->
          <div class="form-group">
            <label class="form-label">来源地址 (可选)</label>
            <input v-model="config.referer_url" type="url" class="form-input" placeholder="https://example.com">
          </div>

          <!-- Only Record -->
          <div class="checkbox-group">
            <input id="only-record" v-model="config.only_record" type="checkbox">
            <label for="only-record">仅记录链接，不下载文件</label>
          </div>

          <!-- Actions -->
          <div class="form-actions">
            <div class="btn-group">
              <button type="submit" class="btn" :class="status.is_running ? 'btn-danger' : 'btn-primary'">
                {{ status.is_running ? '停止' : '开始' }}
              </button>

              <button v-if="items.length > 0" type="button" :disabled="status.is_running" class="btn btn-success" @click="exportResults">
                导出CSV
              </button>

              <button v-if="items.length > 0" type="button" :disabled="status.is_running" class="btn btn-secondary" @click="saveHistory">
                保存历史
              </button>

              <button v-if="items.length > 0" type="button" :disabled="status.is_running" class="btn btn-secondary" @click="clearList">
                清空列表
              </button>
            </div>

            <!-- Quick Stats -->
            <div class="quick-stats">
              <div class="quick-stat">
                <span class="quick-stat-label">总数:</span>
                <span class="quick-stat-value">{{ status.total_found }}</span>
              </div>
              <div class="quick-stat">
                <span class="quick-stat-label">完成:</span>
                <span class="quick-stat-value success">{{ status.total_completed }}</span>
              </div>
              <div class="quick-stat">
                <span class="quick-stat-label">下载中:</span>
                <span class="quick-stat-value info">{{ status.total_downloading }}</span>
              </div>
              <div v-if="status.total_failed > 0" class="quick-stat">
                <span class="quick-stat-label">失败:</span>
                <span class="quick-stat-value danger">{{ status.total_failed }}</span>
              </div>
            </div>
          </div>
        </form>
      </div>

      <!-- Stats Card -->
      <stats-panel v-if="status.is_running || items.length > 0" :status="status" class="card" />

      <!-- Search and Filter -->
      <search-filter v-if="items.length > 0" @filter="handleFilter" />

      <!-- Batch Actions -->
      <batch-actions
        v-if="items.length > 0"
        :items="items"
        :selected-ids="selectedIds"
        @update-selection="selectedIds = $event"
        @batch-retry="handleBatchRetry"
        @batch-delete="handleBatchDelete"
        @batch-export="handleBatchExport"
      />

      <!-- Results Table -->
      <div v-if="items.length > 0" class="card">
        <div class="table-header">
          <h3 class="table-title">
            下载列表 <span class="table-subtitle">(共 {{ filteredItems.length }} 项{{ filteredItems.length > 100 ? '，显示前100项' : '' }})</span>
          </h3>
        </div>

        <div class="table-wrapper">
          <table>
            <thead>
              <tr>
                <th>
                  <input type="checkbox" :checked="selectedIds.length === filteredItems.length && filteredItems.length > 0" @change="toggleSelectAll">
                </th>
                <th>文件名</th>
                <th class="center">
                  大小
                </th>
                <th class="center">
                  状态
                </th>
                <th class="center">
                  重复
                </th>
                <th class="progress-col">
                  进度
                </th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="item in filteredItems.slice(0, 100)"
                :key="item.id"
                :class="{ selected: selectedIds.includes(item.id) }"
                @contextmenu="showContextMenu($event, item)"
              >
                <td>
                  <input type="checkbox" :checked="selectedIds.includes(item.id)" @change="toggleItemSelection(item.id)">
                </td>
                <td>
                  <a href="#" class="file-link" @click.prevent="openFile(item)">{{ item.filename }}</a>
                </td>
                <td class="center">
                  {{ formatBytes(item.size) }}
                </td>
                <td class="center">
                  <span class="status-badge" :class="item.status.toLowerCase()">{{ getStatusText(item.status) }}</span>
                </td>
                <td class="center">
                  {{ item.duplicate_count }}
                </td>
                <td class="progress-col">
                  <div class="progress-container">
                    <div class="progress-bar">
                      <div class="progress-fill" :class="item.status.toLowerCase()" :style="{ width: `${item.progress}%` }" />
                    </div>
                    <div class="progress-info">
                      <span class="progress-percent">{{ item.progress }}%</span>
                      <span v-if="item.status === DownloadStatus.Downloading" class="progress-text">
                        <template v-if="item.speed">
                          {{ formatSpeed(item.speed) }} | {{ formatTimeRemaining(item) }}
                        </template>
                        <template v-else>
                          下载中...
                        </template>
                      </span>
                    </div>
                  </div>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </main>

    <!-- Toast Notifications -->
    <toast ref="toastRef" />

    <!-- Context Menu -->
    <context-menu ref="contextMenuRef" @action="handleContextAction" />
  </div>
</template>

<script setup lang="ts">
import type { UnlistenFn } from '@tauri-apps/api/event';
import { getVersion } from '@tauri-apps/api/app';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { desktopDir, join } from '@tauri-apps/api/path';
import { open, confirm as tauriConfirm } from '@tauri-apps/plugin-dialog';
import { onMounted, onUnmounted, ref, useTemplateRef } from 'vue';
import BatchActions from './components/BatchActions.vue';
import ContextMenu from './components/ContextMenu.vue';
import SearchFilter from './components/SearchFilter.vue';
import StatsPanel from './components/StatsPanel.vue';
import Toast from './components/Toast.vue';
import { useStorage } from './composables/useStorage';
import { DownloadStatus, DownloadStatusUtils } from './utils/downloadStatus';

interface DownloadConfig {
  api_url: string;
  output_dir: string;
  referer_url: string | null;
  concurrency: number;
  max_duplicate: number;
  only_record: boolean;
  max_retries: number;
  retry_delay: number;
}

interface DownloadItem {
  id: string;
  url: string;
  filename: string;
  size: number;
  progress: number;
  status: DownloadStatus;
  duplicate_count: number;
  start_time: string;
  hash: string | null;
  downloaded?: number;
  speed?: number;
}

interface BatchStatus {
  is_running: boolean;
  total_found: number;
  total_completed: number;
  total_duplicates: number;
  total_failed: number;
  total_downloading: number;
  total_size: number;
  downloaded_size: number;
  avg_speed: number;
  elapsed_time: number;
}

const config = useStorage<DownloadConfig>('images-thief-config', {
  api_url: '',
  output_dir: '',
  referer_url: null,
  concurrency: 10,
  max_duplicate: 10,
  only_record: false,
  max_retries: 3,
  retry_delay: 3
});

const status = ref<BatchStatus>({
  is_running: false,
  total_found: 0,
  total_completed: 0,
  total_duplicates: 0,
  total_failed: 0,
  total_downloading: 0,
  total_size: 0,
  downloaded_size: 0,
  avg_speed: 0,
  elapsed_time: 0
});

const items = ref<DownloadItem[]>([]);
const filteredItems = ref<DownloadItem[]>([]);
const selectedIds = ref<string[]>([]);
const toastRef = useTemplateRef('toastRef');
const contextMenuRef = useTemplateRef('contextMenuRef');

const filters = ref({
  search: '',
  status: '',
  sortBy: 'start_time',
  sortOrder: 'desc' as 'asc' | 'desc'
});

const appVersion = ref('...');

let statusInterval: number | null = null;
let unlistenProgress: UnlistenFn | null = null;
let unlistenItemAdded: UnlistenFn | null = null;

onMounted(async () => {
  // Get app version
  appVersion.value = await getVersion();

  // Set default output directory
  if (!config.value.output_dir) {
    const desktopPath = await desktopDir();
    // Use Tauri's cross-platform path join function
    config.value.output_dir = await join(desktopPath, 'images-thief-downloads');
  }

  // Load history
  try {
    const history = await invoke('load_history') as DownloadItem[];
    if (history.length > 0) {
      items.value = history;
      applyFilters();
    }
  } catch (error) {
    console.error('Failed to load history:', error);
  }

  // Listen to events
  unlistenProgress = await listen('download_progress', (event) => {
    const item = event.payload as DownloadItem;
    const index = items.value.findIndex((i) => i.id === item.id);
    if (index !== -1) {
      items.value[index] = item;
      // Only update filtered list if item is in it
      const filteredIndex = filteredItems.value.findIndex((i) => i.id === item.id);
      if (filteredIndex !== -1) {
        filteredItems.value[filteredIndex] = item;
      }
    }
  });

  unlistenItemAdded = await listen('download_item_added', (event) => {
    const item = event.payload as DownloadItem;
    items.value.unshift(item);
    applyFilters();
  });

  // Listen to batch start event to clear list
  await listen('batch_started', () => {
    items.value = [];
    filteredItems.value = [];
    selectedIds.value = [];
  });

  // Start status polling
  statusInterval = setInterval(updateStatus, 1000);
});

onUnmounted(async () => {
  if (statusInterval) clearInterval(statusInterval);
  if (unlistenProgress) unlistenProgress();
  if (unlistenItemAdded) unlistenItemAdded();

  // Save history on exit
  try {
    await invoke('save_history');
  } catch (error) {
    console.error('Failed to save history:', error);
  }
});

async function updateStatus() {
  try {
    const newStatus: BatchStatus = await invoke('get_batch_status');
    status.value = newStatus;
  } catch (error) {
    console.error('Failed to get status:', error);
  }
}

async function handleStart() {
  if (status.value.is_running) {
    try {
      await invoke('stop_batch_download');
      await invoke('save_history');
      toastRef.value?.addToast('info', '已停止下载');
    } catch (error) {
      toastRef.value?.addToast('error', `停止失败: ${error}`);
    }
  } else {
    if (!config.value.api_url) {
      toastRef.value?.addToast('warning', '请输入API地址');
      return;
    }
    try {
      await invoke('start_batch_download', { config: config.value });
      toastRef.value?.addToast('success', '开始下载任务');
    } catch (error) {
      toastRef.value?.addToast('error', `启动失败: ${error}`);
    }
  }
}

async function selectDirectory() {
  const selected = await open({
    directory: true,
    defaultPath: config.value.output_dir
  });

  if (selected && typeof selected === 'string') {
    config.value.output_dir = selected;
  }
}

async function checkForUpdate() {
  try {
    toastRef.value?.addToast('info', '正在检查更新...');
    const updateResult = await invoke('check_update') as any;

    if (updateResult) {
      if (updateResult.available) {
        const shouldUpdate = await tauriConfirm(
          `发现新版本 ${updateResult.version}，是否立即更新？\n\n更新内容：\n${updateResult.body || '无'}`,
          {
            title: '发现新版本',
            okLabel: '立即更新',
            cancelLabel: '稍后再说'
          }
        );

        if (shouldUpdate) {
          // 显示更新进度
          toastRef.value?.addToast('info', `正在下载新版本 ${updateResult.version}...`);

          // 安装更新
          await invoke('install_update');

          toastRef.value?.addToast('success', '更新下载完成，将在下次启动时生效');
        }
      } else {
        toastRef.value?.addToast('success', '当前已是最新版本');
      }
    } else {
      toastRef.value?.addToast('info', '无法检查更新');
    }
  } catch (error) {
    console.error('检查更新失败:', error);
    toastRef.value?.addToast('error', `检查更新失败: ${error}`);
  }
}

async function exportResults() {
  try {
    const csvContent = `URL,Filename,Size,Status,Duplicates\n${
      items.value.map((item) => `${item.url},${item.filename},${item.size},${item.status},${item.duplicate_count}`).join('\n')}`;

    const filename = `images-thief-${Date.now()}.csv`;
    const filePath = await invoke('save_csv', { content: csvContent, filename }) as string;
    toastRef.value?.addToast('success', `CSV已保存到: ${filePath}`);
  } catch (error) {
    toastRef.value?.addToast('error', `导出失败: ${error}`);
  }
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / k ** i).toFixed(2))} ${sizes[i]}`;
}

function getStatusText(status: DownloadStatus): string {
  return DownloadStatusUtils.getText(status);
}

function showContextMenu(event: MouseEvent, item: DownloadItem) {
  const menuItems = [
    { label: '打开文件', icon: '📄', action: 'open-file', disabled: item.status !== DownloadStatus.Completed },
    { label: '打开文件夹', icon: '📁', action: 'open-folder' },
    {
      label: '重新下载',
      icon: '🔄',
      action: 'retry',
      disabled: item.status === DownloadStatus.Completed
    },
    { label: '复制链接', icon: '🔗', action: 'copy-url' },
  ];
  contextMenuRef.value?.show(event, item, menuItems);
}

async function handleContextAction(action: string, item: DownloadItem) {
  switch (action) {
    case 'open-file':
      await openFile(item);
      break;
    case 'open-folder':
      await openFolder();
      break;
    case 'retry':
      await retryDownload(item);
      break;
    case 'copy-url':
      await copyToClipboard(item.url);
      break;
  }
}

async function openFile(item: DownloadItem) {
  if (item.status !== 'Completed') {
    toastRef.value?.addToast('warning', '文件尚未下载完成');
    return;
  }
  try {
    await invoke('open_file', {
      outputDir: config.value.output_dir,
      filename: item.filename
    });
  } catch (error) {
    toastRef.value?.addToast('error', `打开文件失败: ${error}`);
  }
}

async function openFolder() {
  try {
    await invoke('open_folder', { outputDir: config.value.output_dir });
  } catch (error) {
    toastRef.value?.addToast('error', `打开文件夹失败: ${error}`);
  }
}

async function retryDownload(item: DownloadItem) {
  try {
    await invoke('retry_download', {
      url: item.url,
      outputDir: config.value.output_dir,
      filename: item.filename,
      refererUrl: config.value.referer_url
    });
    item.status = DownloadStatus.Downloading;
    item.progress = 0;
    toastRef.value?.addToast('success', `开始重新下载: ${item.filename}`);
  } catch (error) {
    toastRef.value?.addToast('error', `重新下载失败: ${error}`);
  }
}

async function copyToClipboard(text: string) {
  try {
    await navigator.clipboard.writeText(text);
    toastRef.value?.addToast('success', '链接已复制到剪贴板');
  } catch (error) {
    toastRef.value?.addToast('error', `复制失败: ${error}`);
  }
}

function handleFilter(newFilters: typeof filters.value) {
  filters.value = newFilters;
  applyFilters();
}

function applyFilters() {
  let result = [...items.value];

  // 搜索过滤
  if (filters.value.search) {
    result = result.filter((item) =>
      item.filename.toLowerCase().includes(filters.value.search.toLowerCase())
    );
  }

  // 状态过滤
  if (filters.value.status) {
    result = result.filter((item) => item.status === filters.value.status);
  }

  // 排序
  result.sort((a, b) => {
    const aVal = a[filters.value.sortBy as keyof DownloadItem] ?? '';
    const bVal = b[filters.value.sortBy as keyof DownloadItem] ?? '';
    const order = filters.value.sortOrder === 'asc' ? 1 : -1;

    if (aVal < bVal) return -order;
    if (aVal > bVal) return order;
    return 0;
  });

  filteredItems.value = result;
}

function toggleItemSelection(id: string) {
  const index = selectedIds.value.indexOf(id);
  if (index > -1) {
    selectedIds.value.splice(index, 1);
  } else {
    selectedIds.value.push(id);
  }
}

function toggleSelectAll() {
  if (selectedIds.value.length === filteredItems.value.length) {
    selectedIds.value = [];
  } else {
    selectedIds.value = filteredItems.value.map((item) => item.id);
  }
}

async function handleBatchRetry() {
  const selectedItems = items.value.filter((item) => selectedIds.value.includes(item.id));
  let successCount = 0;

  for (const item of selectedItems) {
    try {
      await invoke('retry_download', {
        url: item.url,
        outputDir: config.value.output_dir,
        filename: item.filename,
        refererUrl: config.value.referer_url
      });
      item.status = DownloadStatus.Downloading;
      item.progress = 0;
      successCount++;
    } catch (error) {
      console.error(`Failed to retry ${item.filename}:`, error);
    }
  }

  toastRef.value?.addToast('success', `已重新下载 ${successCount} 个文件`);
  selectedIds.value = [];
}

function handleBatchDelete() {
  items.value = items.value.filter((item) => !selectedIds.value.includes(item.id));
  selectedIds.value = [];
  applyFilters();
  toastRef.value?.addToast('success', '已删除选中记录');
}

async function handleBatchExport() {
  try {
    const selectedItems = items.value.filter((item) => selectedIds.value.includes(item.id));
    const csvContent = `URL,Filename,Size,Status\n${
      selectedItems.map((item) => `${item.url},${item.filename},${item.size},${item.status}`).join('\n')}`;

    const filename = `selected-images-${Date.now()}.csv`;
    const filePath = await invoke('save_csv', { content: csvContent, filename }) as string;
    toastRef.value?.addToast('success', `已导出选中项目到: ${filePath}`);
  } catch (error) {
    toastRef.value?.addToast('error', `导出失败: ${error}`);
  }
}

function isValidUrl(url: string): boolean {
  try {
    // eslint-disable-next-line no-new
    new URL(url);
    return true;
  } catch {
    return false;
  }
}

function formatSpeed(bytesPerSecond: number): string {
  if (bytesPerSecond < 1024) return `${bytesPerSecond.toFixed(0)} B/s`;
  if (bytesPerSecond < 1024 * 1024) return `${(bytesPerSecond / 1024).toFixed(1)} KB/s`;
  return `${(bytesPerSecond / (1024 * 1024)).toFixed(1)} MB/s`;
}

function formatTimeRemaining(item: DownloadItem): string {
  if (!item.speed || item.speed === 0 || item.progress >= 100) return '';

  const remaining = item.size * (1 - item.progress / 100);
  const seconds = Math.ceil(remaining / item.speed);

  if (seconds < 60) return `${seconds}s`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${seconds % 60}s`;
  return `${Math.floor(seconds / 3600)}h ${Math.floor((seconds % 3600) / 60)}m`;
}

async function saveHistory() {
  try {
    await invoke('save_history');
    toastRef.value?.addToast('success', '历史记录已保存');
  } catch (error) {
    toastRef.value?.addToast('error', `保存失败: ${error}`);
  }
}

async function clearList() {
  const confirmed = await tauriConfirm('确定要清空所有下载记录吗？', { title: '确认操作', kind: 'warning' });
  if (confirmed) {
    items.value = [];
    filteredItems.value = [];
    selectedIds.value = [];
    toastRef.value?.addToast('success', '列表已清空');
  }
}
</script>
