use crate::types::*;
use crate::util::request::{download_image_with_progress, fetch_image_info};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::{Mutex, RwLock};
use tokio::time::{Duration, Instant};
use uuid::Uuid;

pub struct DownloadManager {
    config: DownloadConfig,
    items: Arc<RwLock<HashMap<String, DownloadItem>>>,
    dedup_cache: Arc<Mutex<DeduplicationCache>>,
    is_running: Arc<RwLock<bool>>,
    start_time: Option<Instant>,
}

impl DownloadManager {
    pub fn new(config: DownloadConfig) -> Self {
        Self {
            config,
            items: Arc::new(RwLock::new(HashMap::new())),
            dedup_cache: Arc::new(Mutex::new(DeduplicationCache {
                hashes: std::collections::HashSet::new(),
                url_counts: HashMap::new(),
            })),
            is_running: Arc::new(RwLock::new(false)),
            start_time: None,
        }
    }

    pub async fn start(&mut self, app_handle: AppHandle) -> Result<(), String> {
        let mut running = self.is_running.write().await;
        if *running {
            return Err("Already running".to_string());
        }
        *running = true;
        drop(running);

        // Reset state
        self.start_time = Some(Instant::now());
        self.items.write().await.clear();
        self.dedup_cache.lock().await.hashes.clear();
        self.dedup_cache.lock().await.url_counts.clear();
        let _ = app_handle.emit("batch_started", ());
        println!("[DEBUG] Download manager started");

        let config = self.config.clone();
        let items = self.items.clone();
        let dedup_cache = self.dedup_cache.clone();
        let is_running = self.is_running.clone();

        tokio::spawn(async move {
            let semaphore = Arc::new(tokio::sync::Semaphore::new(config.concurrency as usize));

            loop {
                if !*is_running.read().await {
                    println!("[DEBUG] Breaking main loop - stopped");
                    break;
                }

                println!("[DEBUG] Fetching image info from: {}", config.api_url);
                match fetch_image_info(&config.api_url, config.referer_url.as_deref()).await {
                    Ok(Some(info)) => {
                        println!("[DEBUG] Found image: {} ({})", info.filename, info.url);
                        let mut cache = dedup_cache.lock().await;
                        let count = cache.url_counts.entry(info.url.clone()).or_insert(0);
                        *count += 1;

                        if config.max_duplicate > 0 && *count > config.max_duplicate {
                            let _ = app_handle.emit("batch_stopped", "Max duplicates reached");
                            break;
                        }

                        // Check if stopped before processing
                        if !*is_running.read().await {
                            break;
                        }

                        let item = DownloadItem {
                            id: Uuid::new_v4().to_string(),
                            url: info.url.clone(),
                            filename: info.filename,
                            size: info.size,
                            progress: 0,
                            status: DownloadStatus::Pending,
                            duplicate_count: *count,
                            start_time: chrono::Utc::now().to_rfc3339(),
                            hash: None,
                            downloaded: 0,
                            speed: 0.0,
                        };

                        items.write().await.insert(item.id.clone(), item.clone());
                        let _ = app_handle.emit("download_item_added", &item);

                        if !config.only_record {
                            let permit = semaphore.clone().acquire_owned().await.unwrap();
                            let item_clone = item.clone();
                            let items_clone = items.clone();
                            let dedup_cache_clone = dedup_cache.clone();
                            let app_handle_clone = app_handle.clone();
                            let config_clone = config.clone();
                            let is_running_clone = is_running.clone();

                            tokio::spawn(async move {
                                let _permit = permit;
                                // Check if still running before download
                                if *is_running_clone.read().await {
                                    Self::download_single_item(
                                        item_clone,
                                        items_clone,
                                        dedup_cache_clone,
                                        app_handle_clone,
                                        config_clone,
                                    )
                                    .await;
                                }
                            });
                        }
                    }
                    Ok(None) => {
                        println!("[DEBUG] No image found, retrying...");
                        // Short sleep with frequent stop checks
                        for _ in 0..10 {
                            tokio::time::sleep(Duration::from_millis(10)).await;
                            if !*is_running.read().await {
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        println!("[ERROR] Failed to fetch image info: {}", e);
                        // Short sleep with frequent stop checks
                        for _ in 0..100 {
                            tokio::time::sleep(Duration::from_millis(10)).await;
                            if !*is_running.read().await {
                                break;
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    async fn download_single_item(
        mut item: DownloadItem,
        items: Arc<RwLock<HashMap<String, DownloadItem>>>,
        dedup_cache: Arc<Mutex<DeduplicationCache>>,
        app_handle: AppHandle,
        config: DownloadConfig,
    ) {
        item.status = DownloadStatus::Downloading;
        items.write().await.insert(item.id.clone(), item.clone());
        let _ = app_handle.emit("download_progress", &item);

        println!("[DEBUG] Downloading: {} to {}", item.url, config.output_dir);

        // 重试逻辑
        let mut retries = 0;
        let result = loop {
            match download_image_with_progress(
                &item.url,
                &config.output_dir,
                &item.filename,
                config.referer_url.as_deref(),
                Some(app_handle.clone()),
                Some(item.id.clone()),
            )
            .await
            {
                Ok(result) => break Ok(result),
                Err(e) if retries < config.max_retries => {
                    retries += 1;
                    println!(
                        "[WARN] Download failed (attempt {}/{}): {}",
                        retries, config.max_retries, e
                    );
                    tokio::time::sleep(Duration::from_secs(config.retry_delay)).await;
                    continue;
                }
                Err(e) => break Err(e),
            }
        };

        match result {
            Ok((hash, downloaded, speed)) => {
                println!(
                    "[DEBUG] Download completed: {} (hash: {}, speed: {:.2} KB/s)",
                    item.filename,
                    hash,
                    speed / 1024.0
                );
                let mut cache = dedup_cache.lock().await;
                if cache.hashes.contains(&hash) {
                    item.status = DownloadStatus::Duplicate;
                } else {
                    cache.hashes.insert(hash.clone());
                    item.status = DownloadStatus::Completed;
                    item.progress = 100;
                }
                item.hash = Some(hash);
                item.downloaded = downloaded;
                item.speed = speed;
            }
            Err(e) => {
                println!("[ERROR] Download failed for {}: {}", item.filename, e);
                item.status = DownloadStatus::Failed;
            }
        }

        items.write().await.insert(item.id.clone(), item.clone());
        let _ = app_handle.emit("download_progress", &item);
    }

    pub async fn stop(&self) {
        println!("[DEBUG] Stopping download manager...");
        let mut running = self.is_running.write().await;
        *running = false;
        println!("[DEBUG] Download manager stopped");
    }

    pub async fn get_status(&self) -> BatchStatus {
        let items = self.items.read().await;
        let is_running = *self.is_running.read().await;

        let total_found = items.len() as u32;
        let total_completed = items
            .values()
            .filter(|item| matches!(item.status, DownloadStatus::Completed))
            .count() as u32;
        let total_duplicates = items
            .values()
            .filter(|item| matches!(item.status, DownloadStatus::Duplicate))
            .count() as u32;
        let total_failed = items
            .values()
            .filter(|item| matches!(item.status, DownloadStatus::Failed))
            .count() as u32;
        let total_downloading = items
            .values()
            .filter(|item| matches!(item.status, DownloadStatus::Downloading))
            .count() as u32;

        let total_size: u64 = items.values().map(|item| item.size).sum();
        let downloaded_size: u64 = items
            .values()
            .filter(|item| matches!(item.status, DownloadStatus::Completed))
            .map(|item| item.size)
            .sum();

        // Only calculate speed when running
        let avg_speed: f64 = if is_running {
            items
                .values()
                .filter(|item| {
                    matches!(item.status, DownloadStatus::Downloading) && item.speed > 0.0
                })
                .map(|item| item.speed)
                .sum()
        } else {
            0.0
        };

        let elapsed_time = self
            .start_time
            .map(|start| start.elapsed().as_secs())
            .unwrap_or(0);

        BatchStatus {
            is_running,
            total_found,
            total_completed,
            total_duplicates,
            total_failed,
            total_downloading,
            total_size,
            downloaded_size,
            avg_speed,
            elapsed_time,
        }
    }

    pub async fn get_items(&self) -> Vec<DownloadItem> {
        self.items.read().await.values().cloned().collect()
    }
}
