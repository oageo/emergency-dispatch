use reqwest::blocking::Client;
use reqwest::header::{HeaderMap};
use serde_json::json;
use std::fs::File;
use std::io::Write;
use crate::to_half_width; 

// `ACCESS_UA`をlib.rsから参照
use super::super::ACCESS_UA;

const HOST: &str = "niigata119.city.niigata.lg.jp";
const ACCEPT: &str = "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8";
const ACCEPT_LANGUAGE: &str = "ja,en-US;q=0.7,en;q=0.3";
const CONNECTION: &str = "keep-alive";
const CONTENT_TYPE: &str = "application/x-www-form-urlencoded";
const GET_SOURCE: &str = "https://niigata119.city.niigata.lg.jp/";

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
    let body = res.text()?;
    Ok(body)
}

pub fn return_151009() -> Result<(), Box<dyn std::error::Error>> {
    println!("151009, 新潟市消防局");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);
    let selector = scraper::Selector::parse("html body div#pageWrap.clearfix div#topWrap header div#header div#news p#newInfo").unwrap();
    let mut disaster_data = vec![];

    // 各<p>要素を解析
    for element in document.select(&selector) {
        let text = to_half_width(&element.text().collect::<String>().trim().to_string()); // 全角数字を半角数字に変換

        // 「災害は発生しておりません」が含まれている場合はスキップ
        if text.contains("災害は発生しておりません") {
            continue;
        }

        // データを解析
        else if let Some((date_time, rest)) = text.split_once("頃、") {
            if let Some((location, reason)) = rest.split_once("で") {
                let time = date_time
                    .split('日') // 「日」で分割して日付部分を除去
                    .nth(1) // 「日」の後の部分を取得
                    .unwrap_or("")
                    .trim() // 前後の空白を削除
                    .replace("時", ":")
                    .replace("分", ""); // 時刻を整形
                let address = format!(
                    "新潟県新潟市{}",
                    location.trim().replace('　', "").replace("付近", "") // 空白と「付近」を削除
                );
                let disaster_type = reason
                    .trim()
                    .split("のため")
                    .next()
                    .unwrap_or("") // 「のため」以降を削除
                    .trim_end_matches("活動");

                disaster_data.push(json!({
                    "type": disaster_type,
                    "address": address,
                    "time": time
                }));
            }
        }
    }

    let output = json!({
        "jisx0402": "151009",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "新潟市消防局"
            }
        ],
        "disasters": disaster_data
    });

    // JSONファイルに書き出し
    let mut file = File::create("dist/151009.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 151009.json （新潟市消防局）");
    Ok(())
}