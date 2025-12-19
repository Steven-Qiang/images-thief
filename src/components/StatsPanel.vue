<template>
  <div class="stats-panel">
    <div>
      <div class="stat-label">
        发现
      </div>
      <div class="stat-value primary">
        {{ status.total_found }}
      </div>
    </div>
    <div>
      <div class="stat-label">
        完成
      </div>
      <div class="stat-value success">
        {{ status.total_completed }}
      </div>
    </div>
    <div>
      <div class="stat-label">
        下载中
      </div>
      <div class="stat-value info">
        {{ status.total_downloading }}
      </div>
    </div>
    <div>
      <div class="stat-label">
        失败
      </div>
      <div class="stat-value danger">
        {{ status.total_failed }}
      </div>
    </div>
    <div>
      <div class="stat-label">
        重复
      </div>
      <div class="stat-value warning">
        {{ status.total_duplicates }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
interface BatchStatus {
  is_running: boolean;
  total_found: number;
  total_completed: number;
  total_duplicates: number;
  total_failed: number;
  total_downloading: number;
}

defineProps<{
  status: BatchStatus;
}>();
</script>

<style scoped>
.stats-panel {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 20px;
  text-align: center;
  padding: 20px;
  background: #fff;
  border: 1px solid #e0e0e0;
  border-radius: 6px;
  margin-bottom: 24px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08);
}

.stat-label {
  font-size: 12px;
  color: #999;
  margin-bottom: 8px;
}

.stat-value {
  font-size: 28px;
  font-weight: 600;
}

.stat-value.primary {
  color: #333;
}
.stat-value.success {
  color: #388e3c;
}
.stat-value.info {
  color: #1976d2;
}
.stat-value.danger {
  color: #d32f2f;
}
.stat-value.warning {
  color: #f57c00;
}
</style>
