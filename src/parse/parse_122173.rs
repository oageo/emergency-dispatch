use reqwest::blocking::Client;
use reqwest::header::{HeaderMap};
use serde_json::json;
use std::fs::File;
use std::io::Write;
use encoding_rs::SHIFT_JIS; // Shift_JISエンコーディング用
use crate::to_half_width; 

// `ACCESS_UA`をlib.rsから参照
use super::super::ACCESS_UA;

const HOST: &str = "www.city.kashiwa.lg.jp";
const ACCEPT: &str = "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8";
const ACCEPT_LANGUAGE: &str = "ja,en-US;q=0.7,en;q=0.3";
const CONNECTION: &str = "keep-alive";
const CONTENT_TYPE: &str = "application/x-www-form-urlencoded";
const GET_SOURCE: &str = "https://www.city.kashiwa.lg.jp/fdk/disaster/index.html";

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

pub fn return_122173() -> Result<(), Box<dyn std::error::Error>> {
    println!("122173, 柏市消防局");
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
    let selector = scraper::Selector::parse("html body div table.SGINFO tbody tr td.MAINTEXT").unwrap();
    let mut disaster_data = vec![];

    for element in document.select(&selector) {
        // まず全角・半角スペースを除去
        let text = to_half_width(&element.text().collect::<String>())
            .replace('　', "")
            .replace(' ', "")
            .trim()
            .to_string();

        // 「現在、管内で災害は発生しておりません」または「終了」が含まれている場合はスキップ
        if text.contains("現在、管内で災害は発生しておりません") || text.contains("終了") {
            continue;
        }
        // データを解析
        else if let Some((location, after_de)) = text.split_once("で") {
            // address: 全角括弧が一組だけの場合に対応
            let mut addr = location
                .split("ごろ柏市")
                .nth(1)
                .unwrap_or("")
                .replace("付近", "")
                .trim()
                .to_string();

            if let (Some(start), Some(end)) = (addr.find('（'), addr.find('）')) {
                if start < end {
                    addr.replace_range(start..=end, "");
                }
            }

            let address = format!("千葉県柏市{}", addr);

            // disaster_type: 「付近で」以降、「が発生」の間
            let disaster_type = if let Some((ty, _)) = after_de.split_once("が発生") {
                ty.trim().to_string()
            } else {
                after_de.trim().to_string()
            };

            disaster_data.push(json!({
                "type": disaster_type,
                "address": address,
                "time": time
            }));
        }
    }

    let output = json!({
        "jisx0402": "122173",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "柏市消防局"
            }
        ],
        "disasters": disaster_data
    });

    let mut file = File::create("dist/122173.json")?;
    // JSONファイルに書き出し
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 122173.json （柏市消防局）");
    Ok(())
}