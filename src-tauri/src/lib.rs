mod download_manager;
mod storage;
mod types;
mod util;

use download_manager::DownloadManager;
use std::sync::Arc;
use tauri::{AppHandle, State};
use tokio::sync::Mutex;
use types::*;

type ManagerState = Arc<Mutex<Option<DownloadManager>>>;

#[tauri::command]
async fn start_batch_download(
    config: DownloadConfig,
    manager: State<'_, ManagerState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let mut manager_guard = manager.lock().await;

    // Check if already running
    if let Some(existing) = manager_guard.as_ref() {
        if existing.get_status().await.is_running {
            return Err("Download already in progress".to_string());
        }
    }

    // Clear old manager and create new one
    let mut download_manager = DownloadManager::new(config);
    download_manager.start(app_handle).await?;
    *manager_guard = Some(download_manager);

    Ok(())
}

#[tauri::command]
async fn stop_batch_download(manager: State<'_, ManagerState>) -> Result<(), String> {
    let manager_guard = manager.lock().await;

    if let Some(download_manager) = manager_guard.as_ref() {
        download_manager.stop().await;
        Ok(())
    } else {
        Err("No download in progress".to_string())
    }
}

#[tauri::command]
async fn get_batch_status(manager: State<'_, ManagerState>) -> Result<BatchStatus, String> {
    let manager_guard = manager.lock().await;

    if let Some(download_manager) = manager_guard.as_ref() {
        Ok(download_manager.get_status().await)
    } else {
        Ok(BatchStatus {
            is_running: false,
            total_found: 0,
            total_completed: 0,
            total_duplicates: 0,
            total_failed: 0,
            total_downloading: 0,
            total_size: 0,
            downloaded_size: 0,
            avg_speed: 0.0,
            elapsed_time: 0,
        })
    }
}

#[tauri::command]
async fn get_download_items(manager: State<'_, ManagerState>) -> Result<Vec<DownloadItem>, String> {
    let manager_guard = manager.lock().await;

    if let Some(download_manager) = manager_guard.as_ref() {
        Ok(download_manager.get_items().await)
    } else {
        Ok(vec![])
    }
}

#[tauri::command]
async fn save_csv(content: String, filename: String) -> Result<String, String> {
    let desktop_dir = dirs::desktop_dir().ok_or("Cannot find desktop directory")?;
    let file_path = desktop_dir.join(filename);

    std::fs::write(&file_path, content).map_err(|e| e.to_string())?;

    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
async fn retry_download(
    url: String,
    output_dir: String,
    filename: String,
    referer_url: Option<String>,
) -> Result<(), String> {
    tokio::spawn(async move {
        match crate::util::request::download_image(
            &url,
            &output_dir,
            &filename,
            referer_url.as_deref(),
        )
        .await
        {
            Ok(_) => println!("[DEBUG] Retry download completed: {}", filename),
            Err(e) => println!("[ERROR] Retry download failed: {}", e),
        }
    });
    Ok(())
}

#[tauri::command]
async fn save_history(manager: State<'_, ManagerState>) -> Result<(), String> {
    let manager_guard = manager.lock().await;
    if let Some(download_manager) = manager_guard.as_ref() {
        let items = download_manager.get_items().await;
        storage::save_download_history(&items)?;
    }
    Ok(())
}

#[tauri::command]
async fn load_history() -> Result<Vec<DownloadItem>, String> {
    storage::load_download_history()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(ManagerState::new(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![
            start_batch_download,
            stop_batch_download,
            get_batch_status,
            get_download_items,
            save_csv,
            retry_download,
            save_history,
            load_history
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
