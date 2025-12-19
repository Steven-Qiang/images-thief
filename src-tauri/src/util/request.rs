use crate::types::*;
use crate::util::util::create_client;
use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::Path;
use url::Url;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone)]
pub struct ImageInfo {
    pub url: String,
    pub filename: String,
    pub size: u64,
}

pub async fn detect_api_type(
    url: &str,
) -> Result<ApiType, Box<dyn std::error::Error + Send + Sync>> {
    let client = create_client();
    let response = client.head(url).send().await?;

    if response.status().is_redirection() {
        Ok(ApiType::Redirect)
    } else if response
        .headers()
        .get("content-type")
        .and_then(|ct| ct.to_str().ok())
        .map(|ct| ct.starts_with("image/"))
        .unwrap_or(false)
    {
        Ok(ApiType::Direct)
    } else {
        Ok(ApiType::Redirect)
    }
}

pub async fn fetch_image_info(
    api_url: &str,
    referer: Option<&str>,
) -> Result<Option<ImageInfo>, Box<dyn std::error::Error + Send + Sync>> {
    let api_type = detect_api_type(api_url).await?;

    match api_type {
        ApiType::Redirect => fetch_redirect_info(api_url, referer).await,
        ApiType::Direct => fetch_direct_info(api_url, referer).await,
    }
}

async fn fetch_redirect_info(
    api_url: &str,
    referer: Option<&str>,
) -> Result<Option<ImageInfo>, Box<dyn std::error::Error + Send + Sync>> {
    let client = create_client();
    let mut request = client.get(api_url);

    if let Some(ref_url) = referer {
        request = request.header("Referer", ref_url);
    }

    let response = request.send().await?;

    if !response.status().is_redirection() {
        return Ok(None);
    }

    let redirect_url = response
        .headers()
        .get("location")
        .and_then(|loc| loc.to_str().ok())
        .ok_or("No redirect location")?;

    let final_url = if redirect_url.starts_with("http") {
        redirect_url.to_string()
    } else {
        Url::parse(api_url)?.join(redirect_url)?.to_string()
    };

    let size = get_content_length(&final_url).await.unwrap_or(0);
    let filename = extract_filename(&final_url);

    Ok(Some(ImageInfo {
        url: final_url,
        filename,
        size,
    }))
}

async fn fetch_direct_info(
    api_url: &str,
    referer: Option<&str>,
) -> Result<Option<ImageInfo>, Box<dyn std::error::Error + Send + Sync>> {
    let client = create_client();
    let mut request = client.head(api_url);

    if let Some(ref_url) = referer {
        request = request.header("Referer", ref_url);
    }

    let response = request.send().await?;

    if !response.status().is_success() {
        return Ok(None);
    }

    let size = response
        .headers()
        .get("content-length")
        .and_then(|len| len.to_str().ok())
        .and_then(|len| len.parse().ok())
        .unwrap_or(0);

    let filename = format!("{}.jpg", uuid::Uuid::new_v4());

    Ok(Some(ImageInfo {
        url: api_url.to_string(),
        filename,
        size,
    }))
}

pub async fn download_image(
    url: &str,
    output_dir: &str,
    filename: &str,
    referer: Option<&str>,
) -> Result<(String, u64, f64), Box<dyn std::error::Error + Send + Sync>> {
    download_image_with_progress(url, output_dir, filename, referer, None, None).await
}

pub async fn download_image_with_progress(
    url: &str,
    output_dir: &str,
    filename: &str,
    referer: Option<&str>,
    app_handle: Option<AppHandle>,
    item_id: Option<String>,
) -> Result<(String, u64, f64), Box<dyn std::error::Error + Send + Sync>> {
    std::fs::create_dir_all(output_dir)?;
    
    let client = create_client();
    let mut request = client.get(url);

    if let Some(ref_url) = referer {
        request = request.header("Referer", ref_url);
    }

    let response = request.send().await?;

    if !response.status().is_success() {
        return Err("Download failed".into());
    }

    let total_size = response.content_length().unwrap_or(0);
    let file_path = Path::new(output_dir).join(filename);
    let mut file = std::fs::File::create(&file_path)?;
    let mut hasher = Sha256::new();
    let mut stream = response.bytes_stream();
    
    let start_time = std::time::Instant::now();
    let mut total_downloaded = 0u64;
    let mut last_update = std::time::Instant::now();

    while let Some(chunk) = stream.next().await {
        let bytes = chunk?;
        total_downloaded += bytes.len() as u64;
        hasher.update(&bytes);
        file.write_all(&bytes)?;

        // 每100ms更新一次进度
        if last_update.elapsed().as_millis() > 100 {
            if let (Some(handle), Some(id)) = (&app_handle, &item_id) {
                let elapsed = start_time.elapsed().as_secs_f64();
                let speed = if elapsed > 0.0 { total_downloaded as f64 / elapsed } else { 0.0 };
                let progress = if total_size > 0 {
                    ((total_downloaded as f64 / total_size as f64) * 100.0) as u8
                } else {
                    0
                };

                let update = DownloadItem {
                    id: id.clone(),
                    url: url.to_string(),
                    filename: filename.to_string(),
                    size: total_size,
                    progress,
                    status: DownloadStatus::Downloading,
                    duplicate_count: 0,
                    start_time: String::new(),
                    hash: None,
                    downloaded: total_downloaded,
                    speed,
                };
                let _ = handle.emit("download_progress", &update);
            }
            last_update = std::time::Instant::now();
        }
    }
    
    let elapsed = start_time.elapsed().as_secs_f64();
    let speed = if elapsed > 0.0 { total_downloaded as f64 / elapsed } else { 0.0 };

    let hash = format!("{:x}", hasher.finalize());
    Ok((hash, total_downloaded, speed))
}

async fn get_content_length(url: &str) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    let client = create_client();
    let response = client.head(url).send().await?;

    let size = response
        .headers()
        .get("content-length")
        .and_then(|len| len.to_str().ok())
        .and_then(|len| len.parse().ok())
        .unwrap_or(0);

    Ok(size)
}

fn extract_filename(url: &str) -> String {
    Url::parse(url)
        .ok()
        .and_then(|u| {
            u.path_segments()
                .and_then(|segments| segments.last().map(|s| s.to_string()))
        })
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| format!("{}.jpg", uuid::Uuid::new_v4()))
}
