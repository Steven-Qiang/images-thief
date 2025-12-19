use crate::types::DownloadItem;
use serde_json;
use std::fs;
use std::path::PathBuf;

fn get_history_file() -> Result<PathBuf, String> {
    let app_dir = dirs::data_local_dir()
        .ok_or("Cannot find data directory")?
        .join("images-thief");
    
    fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
    Ok(app_dir.join("history.json"))
}

pub fn save_download_history(items: &[DownloadItem]) -> Result<(), String> {
    let history_file = get_history_file()?;
    let json = serde_json::to_string_pretty(items).map_err(|e| e.to_string())?;
    fs::write(history_file, json).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn load_download_history() -> Result<Vec<DownloadItem>, String> {
    let history_file = get_history_file()?;
    
    if !history_file.exists() {
        return Ok(vec![]);
    }
    
    let content = fs::read_to_string(history_file).map_err(|e| e.to_string())?;
    let items: Vec<DownloadItem> = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    Ok(items)
}
