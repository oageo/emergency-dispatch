use reqwest::blocking::Client;
use reqwest::header::{HeaderMap};
use serde_json::json;
use std::fs::File;
use std::io::Write;
use encoding_rs::SHIFT_JIS; // Shift_JISエンコーディング用

// `ACCESS_UA`をlib.rsから参照
use super::super::ACCESS_UA;
use crate::to_half_width; // `lib.rs`の関数をインポート

const HOST: &str = "mama.city.ichikawa.chiba.jp";
const ACCEPT: &str = "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8";
const ACCEPT_LANGUAGE: &str = "ja,en-US;q=0.7,en;q=0.3";
const CONNECTION: &str = "keep-alive";
const CONTENT_TYPE: &str = "application/x-www-form-urlencoded";
const GET_SOURCE: &str = "http://mama.city.ichikawa.chiba.jp/saigai/index.html";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::HOST, HOST.parse().unwrap());
    headers.insert(reqwest::header::ACCEPT, ACCEPT.parse().unwrap());
    headers.insert(reqwest::header::ACCEPT_LANGUAGE, ACCEPT_LANGUAGE.parse().unwrap());
    headers.insert(reqwest::header::CONNECTION, CONNECTION.parse().unwrap());
    headers.insert(reqwest::header::CONTENT_TYPE, CONTENT_TYPE.parse().unwrap());
    headers.insert(reqwest::header::USER_AGENT, ACCESS_UA.parse()?);

    let client = Client::builder()
        .default_headers(headers.clone())
        .build()?;

    let res = client.get(GET_SOURCE)
        .headers(headers)
        .send()?;
    let body_bytes = res.bytes()?; // バイト列として取得
    let (body, _, _) = SHIFT_JIS.decode(&body_bytes); // Shift_JISからUTF-8に変換
    Ok(body.into_owned())
}

pub fn return_122033() -> Result<(), Box<dyn std::error::Error>> {
    println!("122033, 市川市消防局");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);
    let selector = scraper::Selector::parse("html body table tbody tr td div table tbody tr td div font.fs-18-bk-b").unwrap();
    let mut disaster_data = vec![];

    // 各<font>要素を解析
    for element in document.select(&selector) {
        let text = to_half_width(&element.text().collect::<String>().trim().to_string()); // `to_half_width`を使用

        // 「終了」が含まれている場合はスキップ
        if text.contains("終了") {
            continue;
        }

        // データを解析
        if let Some((date_time, rest)) = text.split_once("頃市川市") {
            if let Some((address, disaster_type)) = rest.split_once("付近の") {
                let time = if date_time.contains("午後") {
                    date_time
                        .replace("午後", "")
                        .split_whitespace()
                        .nth(1)
                        .unwrap_or("")
                        .replace("時", ":")
                        .replace("分", "")
                        .split(':')
                        .enumerate()
                        .map(|(i, part)| {
                            if i == 0 {
                                // 時を24時間表記に変換
                                let hour: u32 = part.parse().unwrap_or(0);
                                (hour + 12) % 24 // 午後の場合は12を加算
                            } else {
                                part.parse().unwrap_or(0)
                            }
                        })
                        .collect::<Vec<u32>>()
                        .iter()
                        .map(|n| format!("{:02}", n))
                        .collect::<Vec<String>>()
                        .join(":")
                } else {
                    date_time
                        .replace("午前", "")
                        .split_whitespace()
                        .nth(1)
                        .unwrap_or("")
                        .replace("時", ":")
                        .replace("分", "")
                };

                let disaster_type = disaster_type.trim().replace("活動", "");
                let address = format!("千葉県市川市{}", address.trim());

                disaster_data.push(json!({
                    "type": disaster_type,
                    "address": address,
                    "time": time
                }));
            }
        }
    }

    let output = json!({
        "jisx0402": "122033",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "市川市消防局"
            }
        ],
        "disasters": disaster_data
    });

    // JSONファイルに書き出し
    let mut file = File::create("dist/122033.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 122033.json （市川市消防局）");
    Ok(())
}