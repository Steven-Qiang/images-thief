<template>
  <a href="#" class="update-link" @click.prevent="checkForUpdateOnLinkClick">检查更新</a>
  <div v-if="showModal" class="modal-overlay" @click="closeModal">
    <div class="modal-content" @click.stop>
      <div class="modal-header">
        <h3 v-if="!checking && !downloading && !installing">
          {{ updateInfo ? `发现新版本 v${updateInfo.version}` : '检查更新' }}
        </h3>
        <h3 v-else-if="checking">
          正在检查更新...
        </h3>
        <h3 v-else-if="downloading">
          正在下载更新 v{{ updateInfo?.version }}
        </h3>
        <h3 v-else-if="installing">
          正在安装更新
        </h3>
        <button class="close-btn" @click="closeModal">
          ×
        </button>
      </div>

      <div class="modal-body">
        <div v-if="!checking && !downloading && !installing">
          <div v-if="updateInfo">
            <div class="update-details">
              <p><strong>版本:</strong> v{{ updateInfo.version }}</p>
              <p v-if="updateInfo.date">
                <strong>发布日期:</strong> {{ formatDate(updateInfo.date) }}
              </p>
              <p>
                <a href="#" class="release-link" @click.prevent="openReleases">
                  查看发布页面
                </a>
              </p>
            </div>

            <div v-if="updateInfo.body" class="update-notes">
              <h4>更新内容:</h4>
              <pre class="notes-content">{{ updateInfo.body }}</pre>
            </div>

            <div class="modal-actions">
              <button class="btn btn-primary" @click="startDownload">
                立即更新
              </button>
              <button class="btn btn-secondary" @click="closeModal">
                稍后再说
              </button>
            </div>
          </div>
          <div v-else-if="!checking">
            <p>当前已是最新版本</p>
            <div class="modal-actions">
              <button class="btn btn-secondary" @click="closeModal">
                关闭
              </button>
            </div>
          </div>
        </div>

        <div v-else-if="checking">
          <p>正在检查是否有新版本...</p>
          <div class="checking-animation">
            <div class="spinner" />
          </div>
        </div>

        <div v-else-if="downloading">
          <div class="download-section">
            <div class="progress-container">
              <div class="progress-bar">
                <div class="progress-fill" :style="{ width: `${downloadProgress}%` }" />
              </div>
              <div class="progress-text">
                <span>{{ Math.round(downloadProgress) }}%</span>
                <span v-if="downloadSpeed"> • {{ formatSpeed(downloadSpeed) }}</span>
                <span v-if="eta"> • 预计剩余: {{ eta }}</span>
              </div>
            </div>

            <div class="download-stats">
              <p>已下载: {{ downloadedSize }} / {{ totalSize }}</p>
              <p v-if="downloadSpeed">
                下载速度: {{ formatSpeed(downloadSpeed) }}
              </p>
            </div>
          </div>
        </div>

        <div v-else-if="installing">
          <p>正在安装更新...</p>
          <div class="installing-animation">
            <div class="spinner" />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { openUrl } from '@tauri-apps/plugin-opener';
import { relaunch } from '@tauri-apps/plugin-process';
import { check } from '@tauri-apps/plugin-updater';
import { nextTick, onMounted, ref } from 'vue';

interface UpdateInfo {
  version: string;
  date?: string;
  body?: string;
}

const showModal = ref(false);
const updateInfo = ref<UpdateInfo | null>(null);
const checking = ref(false);
const downloading = ref(false);
const installing = ref(false);
const downloadProgress = ref(0);
const downloadSpeed = ref(0); // bytes per second
const eta = ref(''); // estimated time of arrival
const downloadedSize = ref('0 B');
const totalSize = ref('0 B');

let totalContentLength = 0;
let totalDownloaded = 0;
let lastTime = 0;
let speedSamples: number[] = [];

function closeModal() {
  // 重置状态
  updateInfo.value = null;
  checking.value = false;
  downloading.value = false;
  installing.value = false;
  downloadProgress.value = 0;
  downloadSpeed.value = 0;
  eta.value = '';
  downloadedSize.value = '0 B';
  totalSize.value = '0 B';
  totalContentLength = 0;
  totalDownloaded = 0;
  speedSamples = [];
  showModal.value = false;
}

async function startDownload() {
  if (!updateInfo.value) return;

  try {
    downloading.value = true;
    downloadProgress.value = 0;

    const update = await check();
    if (update) {
      await update.downloadAndInstall(handleDownloadProgress);

      installing.value = true;
      // 在安装完成后重启应用
      setTimeout(async () => {
        await relaunch();
      }, 1000);
    }
  } catch (error) {
    console.error('更新下载失败:', error);
    alert(`更新下载失败: ${(error as Error).message}`);
    downloading.value = false;
  }
}

function handleDownloadProgress(event: any) {
  if (event.event === 'Started') {
    const contentLength = event.data?.contentLength || event.contentLength || 0;
    totalContentLength = contentLength;
    totalSize.value = formatBytes(contentLength);
    totalDownloaded = 0;
    lastTime = Date.now();
    speedSamples = [];
    downloadProgress.value = 0;
    downloadSpeed.value = 0;
    eta.value = '';
    downloadedSize.value = '0 B';
  } else if (event.event === 'Progress') {
    const chunkLength = event.data?.chunkLength || event.chunkLength || 0;
    totalDownloaded += chunkLength;

    // 使用 Started 事件中保存的 totalContentLength
    if (totalContentLength > 0) {
      downloadProgress.value = Math.min((totalDownloaded / totalContentLength) * 100, 100);
    }

    // 计算速度
    const now = Date.now();
    const deltaTime = now - lastTime;
    if (deltaTime > 0) {
      const currentSpeed = (chunkLength / deltaTime) * 1000;
      speedSamples.push(currentSpeed);
      if (speedSamples.length > 5) {
        speedSamples.shift();
      }
      downloadSpeed.value = speedSamples.reduce((a, b) => a + b, 0) / speedSamples.length;
      lastTime = now;
    }

    // 计算剩余时间
    if (downloadSpeed.value > 0 && totalContentLength > 0) {
      const remainingBytes = totalContentLength - totalDownloaded;
      const remainingSeconds = remainingBytes / downloadSpeed.value;
      if (remainingSeconds > 0 && isFinite(remainingSeconds)) {
        const minutes = Math.floor(remainingSeconds / 60);
        const seconds = Math.floor(remainingSeconds % 60);
        eta.value = `${minutes}m ${seconds}s`;
      }
    }

    downloadedSize.value = formatBytes(totalDownloaded);
  } else if (event.event === 'Finished') {
    downloadProgress.value = 100;
    downloading.value = false;
    installing.value = true;
  }
}

function formatDate(dateString?: string) {
  if (!dateString) return 'N/A';
  const date = new Date(dateString);
  return date.toLocaleDateString();
}

function formatBytes(bytes: number) {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / k ** i).toFixed(2))} ${sizes[i]}`;
}

function formatSpeed(bytesPerSecond: number) {
  if (bytesPerSecond < 1024) return `${bytesPerSecond.toFixed(0)} B/s`;
  if (bytesPerSecond < 1024 * 1024) return `${(bytesPerSecond / 1024).toFixed(1)} KB/s`;
  return `${(bytesPerSecond / (1024 * 1024)).toFixed(1)} MB/s`;
}

// 静默检查更新
async function silentCheckForUpdate() {
  try {
    const update = await check();
    if (update) {
      updateInfo.value = {
        version: update.version,
        date: update.date?.toString(),
        body: update.body
      };
      showModal.value = true; // 有更新时自动弹出
    }
  } catch (error) {
    console.error('静默检查更新失败:', error);
  }
}

// 检查更新
async function checkForUpdate() {
  try {
    checking.value = true;
    const update = await check();
    if (update) {
      updateInfo.value = {
        version: update.version,
        date: update.date?.toString(),
        body: update.body
      };
    } else {
      updateInfo.value = null; // 表示没有更新
    }
  } catch (error) {
    console.error('检查更新失败:', error);
    alert(`检查更新失败: ${(error as Error).message}`);
    updateInfo.value = null;
  } finally {
    checking.value = false;
  }
}

async function checkForUpdateOnLinkClick() {
  showModal.value = true;
  await nextTick();
  await checkForUpdate();
}

async function openReleases() {
  try {
    await openUrl('https://github.com/Steven-Qiang/images-thief/releases');
  } catch (error) {
    console.error('打开发布页面失败:', error);
  }
}

// 页面加载时自动检查更新
onMounted(() => {
  silentCheckForUpdate();
});
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
}

.modal-content {
  background: white;
  border-radius: 8px;
  width: 500px;
  max-width: 90vw;
  max-height: 80vh;
  overflow: hidden;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid #eee;
}

.modal-header h3 {
  margin: 0;
  font-size: 16px;
  color: #333;
}

.close-btn {
  background: none;
  border: none;
  font-size: 20px;
  cursor: pointer;
  color: #999;
  padding: 0;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.close-btn:hover {
  color: #333;
}

.modal-body {
  padding: 20px;
}

.update-details {
  margin-bottom: 16px;
}

.update-details p {
  margin: 6px 0;
  font-size: 14px;
}

.update-notes {
  margin-bottom: 20px;
  padding: 12px;
  background-color: #f8f9fa;
  border-radius: 4px;
  max-height: 200px;
  overflow-y: auto;
}

.update-notes h4 {
  margin-top: 0;
  margin-bottom: 8px;
  font-size: 14px;
  color: #333;
}

.notes-content {
  white-space: pre-wrap;
  font-size: 13px;
  line-height: 1.4;
  color: #666;
  margin: 0;
}

.modal-actions {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
  margin-top: 20px;
}

.download-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.progress-container {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.progress-bar {
  width: 100%;
  height: 12px;
  background-color: #f0f0f0;
  border-radius: 6px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #4caf50, #45a049);
  transition: width 0.3s ease;
  border-radius: 6px;
}

.progress-text {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 13px;
  color: #666;
  margin-top: 4px;
}

.download-stats {
  font-size: 13px;
  color: #666;
}

.download-stats p {
  margin: 4px 0;
}

.checking-animation,
.installing-animation {
  display: flex;
  justify-content: center;
  margin: 20px 0;
}

.spinner {
  width: 24px;
  height: 24px;
  border: 3px solid #f3f3f3;
  border-top: 3px solid #4caf50;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% {
    transform: rotate(0deg);
  }
  100% {
    transform: rotate(360deg);
  }
}

.btn {
  padding: 8px 16px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  transition: background-color 0.2s;
}

.btn-primary {
  background-color: #4caf50;
  color: white;
}

.btn-primary:hover {
  background-color: #45a049;
}

.btn-secondary {
  background-color: #f8f9fa;
  color: #333;
  border: 1px solid #ddd;
}

.btn-secondary:hover {
  background-color: #e9ecef;
}

.update-link {
  color: #4caf50;
  text-decoration: none;
  font-size: 13px;
}

.update-link:hover {
  text-decoration: underline;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #4caf50, #2196f3);
  transition: width 0.3s ease;
  border-radius: 6px;
  min-width: 0;
}

.progress-text {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
  color: #666;
  margin-top: 4px;
}

.download-stats {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 13px;
  color: #666;
}

.checking-animation,
.installing-animation {
  display: flex;
  justify-content: center;
  margin: 20px 0;
}

.spinner {
  width: 30px;
  height: 30px;
  border: 3px solid #f3f3f3;
  border-top: 3px solid #2196f3;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% {
    transform: rotate(0deg);
  }
  100% {
    transform: rotate(360deg);
  }
}

.btn {
  padding: 8px 16px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  border: 1px solid transparent;
  transition: all 0.2s;
}

.btn:hover {
  opacity: 0.9;
}

.btn-primary {
  background: #2196f3;
  color: white;
}

.btn-secondary {
  background: #f1f1f1;
  color: #333;
}

.release-link {
  color: #4caf50;
  text-decoration: none;
  font-size: 13px;
}

.release-link:hover {
  text-decoration: underline;
}

.update-link {
  font-size: 0.8em;
  color: #007bff;
  text-decoration: underline;
  margin-left: 8px;
}

.update-link:hover {
  color: #0056b3;
  text-decoration: underline;
}
</style>
