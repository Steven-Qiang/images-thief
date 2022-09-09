#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use reqwest::header::HeaderMap;
use std::io::Write;

const USER_AGENT:&str="Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/80.0.3987.132 Safari/537.36";

#[tauri::command(async)]
async fn fetch(api_url: &str, ref_url: &str) -> Result<String, String> {
    let mut headers = HeaderMap::new();
    headers.insert("Referer", ref_url.parse().unwrap());
    headers.insert("User-Agent", USER_AGENT.parse().unwrap());
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let response = client.get(api_url).send().await;
    match response {
        Ok(res) => {
            let location = res.headers().get("location");
            match location {
                Some(loc) => {
                    let loc_str = loc.to_str().unwrap();
                    Ok(loc_str.to_string())
                }
                None => Err("No location header".to_string()),
            }
        }
        Err(err) => {
            println!("Error: {}", err);
            Err(err.to_string())
        }
    }
}

#[tauri::command(async)]
async fn download(
    img_url: &str,
    ref_url: &str,
    filename: &str,
    output: &str,
) -> Result<String, String> {
    let mut headers = HeaderMap::new();
    headers.insert("Referer", ref_url.parse().unwrap());
    headers.insert("User-Agent", USER_AGENT.parse().unwrap());
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();
    let response = client.get(img_url).send().await;
    match response {
        Ok(res) => {
            let img = res.bytes().await.unwrap();
            let mut file = std::fs::File::create(format!("{}/{}", output, filename)).unwrap();
            file.write_all(&img).unwrap();
            Ok(format!(
                "{{\"ok\":true,\"size\":{},\"filename\":\"{}\"}}",
                img.len(),
                filename
            ))
        }
        Err(e) => {
            println!("Error: {}", e);
            Ok(format!("{{\"ok\":false,\"error\":\"{}\"}}", e))
        }
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![fetch, download])
        .run(tauri::generate_context!())
        .expect("failed to run app");
}
