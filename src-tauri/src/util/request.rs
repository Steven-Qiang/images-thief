/*
 * @file: request.rs
 * @description: request.rs
 * @package: images-thief
 * @create: 2022-12-12 12:06:17
 * @author: qiangmouren (2962051004@qq.com)
 * -----
 * @last-modified: 2022-12-12 12:07:29
 * -----
 */

extern crate url;
use crate::util::util::create_client;
use url::Url;

pub async fn get_file_size(url: String) -> String {
    let client = create_client();
    let res = client.head(&url).send().await.unwrap();
    println!("StatusCode: {:?}", res.status());
    if res.status() != reqwest::StatusCode::OK {
        return "".to_string();
    }
    let size = res
        .headers()
        .get(reqwest::header::CONTENT_LENGTH)
        .unwrap()
        .to_str()
        .unwrap();
    size.to_string()
}

pub fn get_file_name(url: String) -> String {
    let url = Url::parse(&url).unwrap();
    let path = url.path();
    let file_name = path.split("/").last().unwrap();
    file_name.to_string()
}
