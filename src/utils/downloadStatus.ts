// 定义下载状态枚举
export enum DownloadStatus {
  Pending = 'Pending',
  Downloading = 'Downloading',
  Completed = 'Completed',
  Failed = 'Failed',
  Duplicate = 'Duplicate',
}

// 定义状态配置接口
interface StatusConfig {
  label: string;
  color: string;
  icon: string;
}

// 状态配置映射
const STATUS_CONFIG: Record<DownloadStatus, StatusConfig> = {
  [DownloadStatus.Pending]: { label: '等待', color: '#757575', icon: '⏳' },
  [DownloadStatus.Downloading]: { label: '下载中', color: '#1976d2', icon: '⬇️' },
  [DownloadStatus.Completed]: { label: '完成', color: '#388e3c', icon: '✅' },
  [DownloadStatus.Failed]: { label: '失败', color: '#d32f2f', icon: '❌' },
  [DownloadStatus.Duplicate]: { label: '重复', color: '#e65100', icon: '🔄' },
};

// 下载状态工具类
export const DownloadStatusUtils = {
  // 获取状态文本
  getText(status: DownloadStatus): string {
    return STATUS_CONFIG[status]?.label || status;
  },

  // 获取状态颜色
  getColor(status: DownloadStatus): string {
    return STATUS_CONFIG[status]?.color || '#000000';
  },

  // 获取状态图标
  getIcon(status: DownloadStatus): string {
    return STATUS_CONFIG[status]?.icon || '❓';
  },

  // 获取所有状态选项 (用于选择器等)
  getOptions(): Array<{ value: DownloadStatus; label: string }> {
    return Object.entries(STATUS_CONFIG).map(([value, config]) => ({
      value: value as DownloadStatus,
      label: config.label,
    }));
  },

  // 根据状态获取类型信息
  getType(status: DownloadStatus): StatusConfig {
    return STATUS_CONFIG[status] || { label: status, color: '#000000', icon: '❓' };
  },

  // 检查是否为活动状态（下载中）
  isActive(status: DownloadStatus): boolean {
    return status === DownloadStatus.Downloading;
  },

  // 检查是否为最终状态（完成/失败/重复）
  isFinal(status: DownloadStatus): boolean {
    return [DownloadStatus.Completed, DownloadStatus.Failed, DownloadStatus.Duplicate].includes(status);
  },

  // 检查是否可以重试
  canRetry(status: DownloadStatus): boolean {
    return status === DownloadStatus.Failed || status === DownloadStatus.Duplicate;
  },

  // 获取状态的CSS类名
  getCssClass(status: DownloadStatus): string {
    return status.toLowerCase();
  },

  // 从字符串获取状态枚举
  fromString(statusStr: string): DownloadStatus | null {
    const entries = Object.entries(DownloadStatus);
    for (const [, enumValue] of entries) {
      if (enumValue === statusStr) {
        return enumValue as DownloadStatus;
      }
    }
    return null;
  },
};
