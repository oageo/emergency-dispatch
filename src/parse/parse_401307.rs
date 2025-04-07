use reqwest::blocking::Client;
use reqwest::header::{HeaderMap};
use serde_json::json;
use std::fs::File;
use std::io::Write;
use crate::to_half_width; 

// `ACCESS_UA`をlib.rsから参照
use super::super::ACCESS_UA;

const HOST: &str = "m119.city.fukuoka.lg.jp";
const ACCEPT: &str = "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8";
const ACCEPT_LANGUAGE: &str = "ja,en-US;q=0.7,en;q=0.3";
const CONNECTION: &str = "keep-alive";
const CONTENT_TYPE: &str = "application/x-www-form-urlencoded";
const GET_SOURCE: &str = "https://m119.city.fukuoka.lg.jp/fukuoka/hpinfo.html";

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

pub fn return_401307() -> Result<(), Box<dyn std::error::Error>> {
    println!("401307, 福岡市消防局");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);
    let selector = scraper::Selector::parse("html body div.emergencywrapper dl.emergencyinfo dd").unwrap();
    let mut disaster_data = vec![];

    // 各<dd>要素を解析
    for element in document.select(&selector) {
        let text = to_half_width(&element.text().collect::<String>().trim().to_string()); // 全角数字を半角数字に変換

        if let Some((time_and_location, reason)) = text.split_once("近に") {
            if let Some((time, location)) = time_and_location.split_once('　') {
                let time = time.trim().replace("：", ":"); // 時刻を整形

                // 住所と災害種別を分離
                if let Some((address, _)) = location.split_once("付") {
                    let address = format!("福岡県福岡市{}", address.trim().replace('　', "")); // 空白を詰める
                    let disaster_type = reason.trim().replace("のため、消防隊が出動しています。", ""); // 「消防隊が出動しています」を削除

                    disaster_data.push(json!({
                        "type": disaster_type,
                        "address": address,
                        "time": time
                    }));
                }
            }
        }
    }

    let output = json!({
        "jisx0402": "401307",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "福岡市消防局"
            }
        ],
        "disasters": disaster_data
    });

    // JSONファイルに書き出し
    let mut file = File::create("dist/401307.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 401307.json （福岡市消防局）");
    Ok(())
}