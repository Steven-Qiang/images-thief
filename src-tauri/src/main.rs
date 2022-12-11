#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::io::Write;
use std::path::Path;

use reqwest;
mod util;

use futures_util::StreamExt;
use tauri::Runtime;
use util::request::{get_file_name, get_file_size};
use util::util::{create_client, url_join};

#[tauri::command(async)]
async fn get<R: Runtime>(
    _app: tauri::AppHandle<R>,
    _window: tauri::Window<R>,
    url: String,
    referer: String,
) -> String {
    println!("url: {}, referer: {}", url, referer);
    let client = create_client();
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::REFERER,
        reqwest::header::HeaderValue::from_str(&referer).unwrap(),
    );
    let res = client.get(&url).headers(headers).send().await.unwrap();
    println!("StatusCode: {:?}", res.status());

    if res.status() != reqwest::StatusCode::MOVED_PERMANENTLY
        && res.status() != reqwest::StatusCode::FOUND
    {
        return "".to_string();
    }

    let redirect_url = res
        .headers()
        .get(reqwest::header::LOCATION)
        .unwrap()
        .to_str()
        .unwrap();
    println!("redirect_url: {}", redirect_url);
    if redirect_url == "" {
        return "".to_string();
    }
    let real_url: String;
    if !redirect_url.starts_with("http") {
        real_url = url_join(&url.as_str(), &redirect_url);
    } else {
        real_url = redirect_url.to_string();
    }
    println!("real_url: {}", real_url);
    let size = get_file_size((*real_url).to_string()).await;
    println!("size: {}", size);
    let file_name = get_file_name((*real_url).to_string());
    let json = format!(
        r#"{{"url":"{}","size":"{}","filename":"{}"}}"#,
        real_url, size, file_name
    );
    json.as_str().to_string()
}

#[tauri::command(async)]
async fn download<R: Runtime>(
    _app: tauri::AppHandle<R>,
    window: tauri::Window<R>,
    url: String,
    referer: String,
    filename: String,
    size: String,
    path: String,
    unique_id: String,
) -> String {
    println!("url: {}, referer: {}", url, referer);
    let client = create_client();
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::REFERER,
        reqwest::header::HeaderValue::from_str(&referer).unwrap(),
    );
    let res = client.get(&url).headers(headers).send().await.unwrap();
    println!("StatusCode: {:?}", res.status());

    if res.status() != reqwest::StatusCode::OK {
        return "".to_string();
    }

    let file_path = Path::new(&path).join(filename);
    println!("file_path: {}", file_path.to_str().unwrap());
    let mut file = std::fs::File::create(file_path).unwrap();
    let mut stream = res.bytes_stream();
    let mut total = 0;
    while let Some(item) = stream.next().await {
        let bytes = item.unwrap();
        total += bytes.len();
        file.write_all(&bytes).unwrap();
        let progress = (total as f64 / size.parse::<f64>().unwrap()) * 100.0;
        // println!("progress: {}", progress);
        let json = format!(
            r#"{{"progress":"{}","unique_id":"{}"}}"#,
            progress, unique_id
        );
        window.emit("progress", json).unwrap();
    }
    "success".to_string()
}

fn main() {
    tauri::Builder::default()
        .setup(|_app| Ok(()))
        .invoke_handler(tauri::generate_handler![get, download])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
