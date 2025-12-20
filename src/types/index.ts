import type { DownloadStatus } from '../utils/downloadStatus';

export interface Row {
  uniqueId: string;
  url: string;
  size: number;
  duplicate: number;
  filename: string;
  progress: number;
  startTime: string;
}

export interface FormData {
  apiUrl: string;
  outputDir: string;
  options: string[];
  rawOptions: {
    refererUrl: string;
    maxDuplicate: number;
    onlyRecord: boolean;
    concurrency: number;
  };
}

export interface RedirectInfo {
  url: string;
  size: number;
  filename: string;
}

export interface DownloadConfig {
  api_url: string;
  output_dir: string;
  referer_url: string | null;
  concurrency: number;
  max_duplicate: number;
  only_record: boolean;
  max_retries: number;
  retry_delay: number;
}
export interface DownloadItem {
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
export interface BatchStatus {
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
