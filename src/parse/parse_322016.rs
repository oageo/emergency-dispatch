use reqwest::blocking::Client;
use reqwest::header::{HeaderMap};
use serde_json::json;
use std::fs::File;
use std::io::Write;
use encoding_rs::SHIFT_JIS; // Shift_JISエンコーディング用
use crate::to_half_width; 

// `ACCESS_UA`をlib.rsから参照
use super::super::ACCESS_UA;

const HOST: &str = "www.m.shirei-saigai-jyouhou.sd.web-sanin.jp";
const ACCEPT: &str = "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8";
const ACCEPT_LANGUAGE: &str = "ja,en-US;q=0.7,en;q=0.3";
const CONNECTION: &str = "keep-alive";
const CONTENT_TYPE: &str = "application/x-www-form-urlencoded";
const GET_SOURCE: &str = "https://www.m.shirei-saigai-jyouhou.sd.web-sanin.jp/index.html";

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

pub fn return_322016() -> Result<(), Box<dyn std::error::Error>> {
    println!("322016, 松江市消防本部");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);

    // 時刻を取得
    let time_selector = scraper::Selector::parse("html body table tbody tr td b font.TIME").unwrap();
    let time = document
        .select(&time_selector)
        .next()
        .map(|element| {
            to_half_width(&element.text().collect::<String>().trim().to_string())
                .replace("現在","")
                .split_whitespace()
                .nth(1)
                .unwrap_or("")
                .replace("時", ":")
                .replace("分", "") // 時刻を整形
        })
        .unwrap_or_else(|| "不明".to_string());

    // 災害情報を取得
    let selector = scraper::Selector::parse("html body div table.SGINFO tbody tr td").unwrap();
    let mut disaster_data = vec![];

    for element in document.select(&selector) {
        let text = to_half_width(&element.text().collect::<String>().trim().to_string()); // 全角数字を半角数字に変換

        // 「ただいま管内で災害は発生しておりません」または「ないことを確認し終了」が含まれている場合はスキップ
        if text.contains("ただいま管内で災害は発生しておりません") || text.contains("ないことを確認し終了") {
            continue;
        }

        // データを解析
        if let Some((location, reason)) = text.split_once("で") {
            let address = format!(
                "島根県松江市{}",
                location
                    .trim()
                    .replace('　', "") // 全角スペースを削除
                    .replace("ただいま", "") // 「ただいま」を削除
            );
            let disaster_type = reason
                .trim()
                .split("要請")
                .next()
                .unwrap_or("") // 「要請」以降を削除
                .trim_end_matches("活動"); // 「活動」を削除

            disaster_data.push(json!({
                "type": disaster_type,
                "address": address,
                "time": time
            }));
        }
    }

    let output = json!({
        "jisx0402": "322016",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "松江市消防本部"
            }
        ],
        "disasters": disaster_data
    });

    // JSONファイルに書き出し
    let mut file = File::create("dist/322016.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 322016.json （松江市消防本部）");
    Ok(())
}