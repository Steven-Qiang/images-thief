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
