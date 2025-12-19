use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadConfig {
    pub api_url: String,
    pub output_dir: String,
    pub referer_url: Option<String>,
    pub concurrency: u32,
    pub max_duplicate: u32,
    pub only_record: bool,
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    #[serde(default = "default_retry_delay")]
    pub retry_delay: u64,
}

fn default_max_retries() -> u32 { 3 }
fn default_retry_delay() -> u64 { 2 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadItem {
    pub id: String,
    pub url: String,
    pub filename: String,
    pub size: u64,
    pub progress: u8,
    pub status: DownloadStatus,
    pub duplicate_count: u32,
    pub start_time: String,
    pub hash: Option<String>,
    pub downloaded: u64,
    pub speed: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Completed,
    Failed,
    Duplicate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchStatus {
    pub is_running: bool,
    pub total_found: u32,
    pub total_completed: u32,
    pub total_duplicates: u32,
    pub total_failed: u32,
    pub total_downloading: u32,
    pub total_size: u64,
    pub downloaded_size: u64,
    pub avg_speed: f64,
    pub elapsed_time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiType {
    Redirect,
    Direct,
}

pub struct DeduplicationCache {
    pub hashes: HashSet<String>,
    pub url_counts: HashMap<String, u32>,
}