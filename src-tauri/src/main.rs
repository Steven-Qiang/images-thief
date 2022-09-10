#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use reqwest::header::HeaderMap;
use std::{io::Write, path::Path};

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

#[derive(serde::Serialize)]
struct Resp {
    ok: bool,
    size: u64,
    filename: String,
    msg: String,
}
#[derive(serde::Serialize)]
struct ErrResp {
    ok: bool,
    msg: String,
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
    let p = format!("{}/{}", output, filename);
    let path = Path::new(p.as_str());
    if path.exists() {
        let resp_json = serde_json::to_string(&Resp {
            ok: true,
            size: path.metadata().unwrap().len(),
            filename: filename.to_string(),
            msg: "文件已存在".to_string(),
        })
        .unwrap();
        Ok(resp_json)
    } else {
        let response = client.get(img_url).send().await;
        match response {
            Ok(res) => {
                let img = res.bytes().await.unwrap();
                let mut file = std::fs::File::create(path).unwrap();
                file.write_all(&img).unwrap();

                let resp_json = serde_json::to_string(&Resp {
                    ok: true,
                    size: img.len() as u64,
                    filename: filename.to_string(),
                    msg: "下载成功".to_string(),
                })
                .unwrap();
                Ok(resp_json)
            }
            Err(e) => {
                println!("Error: {}", e);
                let resp_json = serde_json::to_string(&ErrResp {
                    ok: false,
                    msg: e.to_string(),
                })
                .unwrap();
                Ok(resp_json)
            }
        }
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![fetch, download])
        .run(tauri::generate_context!())
        .expect("failed to run app");
}
