mod download_manager;
mod storage;
mod types;
mod util;

use download_manager::DownloadManager;
use std::sync::Arc;
use tauri::{AppHandle, State};
use tauri_plugin_updater::UpdaterExt;
use tokio::sync::Mutex;
use types::*;

type ManagerState = Arc<Mutex<Option<DownloadManager>>>;

// Define a serializable struct for update information
#[derive(serde::Serialize)]
struct UpdateInfo {
    version: String,
    date: Option<String>,
    body: Option<String>,
}

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
async fn open_file(output_dir: String, filename: String) -> Result<(), String> {
    let file_path = std::path::Path::new(&output_dir).join(&filename);

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &file_path.to_string_lossy()])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&file_path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&file_path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
async fn open_folder(output_dir: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&output_dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&output_dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&output_dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
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

#[tauri::command]
async fn check_update(app: tauri::AppHandle) -> Result<Option<UpdateInfo>, String> {
    let updater = app.updater().map_err(|e| e.to_string())?;
    match updater.check().await.map_err(|e| e.to_string()) {
        Ok(Some(update)) => {
            // Convert Update to UpdateInfo for serialization
            let update_info = UpdateInfo {
                version: update.version.clone(),
                date: update.date.map(|d| d.to_string()),
                body: update.body.clone(),
            };
            Ok(Some(update_info))
        }
        Ok(None) => Ok(None),
        Err(e) => Err(e),
    }
}

#[tauri::command]
async fn install_update(app: tauri::AppHandle) -> Result<(), String> {
    app.restart();
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
            open_file,
            open_folder,
            save_csv,
            retry_download,
            save_history,
            load_history,
            check_update,
            install_update
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
