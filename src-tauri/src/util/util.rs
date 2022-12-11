/*
 * @file: util.rs
 * @description: util.rs
 * @package: images-thief
 * @create: 2022-12-12 12:05:02
 * @author: qiangmouren (2962051004@qq.com)
 * -----
 * @last-modified: 2022-12-12 12:05:28
 * -----
 */

use reqwest;
use url::Url;

pub fn url_join(base: &str, path: &str) -> String {
    Url::parse(base).unwrap().join(path).unwrap().to_string()
}

pub fn create_client() -> reqwest::Client {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_str("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36").unwrap(),
    );
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .default_headers(headers)
        .build();
    client.unwrap()
}
