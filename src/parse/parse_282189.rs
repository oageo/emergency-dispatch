use reqwest::blocking::Client;
use reqwest::header::{HeaderMap};
use serde_json::json;
use std::fs::File;
use std::io::Write;
use encoding_rs::SHIFT_JIS; // Shift_JISエンコーディング用

// `ACCESS_UA`をlib.rsから参照
use super::super::ACCESS_UA;

const HOST: &str = "www.city.ono.hyogo.jp";
const ACCEPT: &str = "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8";
const ACCEPT_LANGUAGE: &str = "ja,en-US;q=0.7,en;q=0.3";
const CONNECTION: &str = "keep-alive";
const CONTENT_TYPE: &str = "application/x-www-form-urlencoded";
const GET_SOURCE: &str = "https://www.city.ono.hyogo.jp/section/Jian.html";

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

pub fn return_282189() -> Result<(), Box<dyn std::error::Error>> {
    println!("282189, 小野市消防本部");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);
    let selector = scraper::Selector::parse("html body center table tbody tr td table tbody tr td table tbody tr").unwrap();
    let mut disaster_data = vec![];

    let mut rows = document.select(&selector);

    rows.next();
    // 2行目以降を解析
    for row in rows {
        let cells: Vec<String> = row
            .select(&scraper::Selector::parse("td").unwrap())
            .map(|cell| cell.text().collect::<String>().trim().to_string())
            .collect();

        // 空行や「現在発生中の事案はありません」などはスキップ
        if cells.iter().any(|cell| cell.contains("現在発生中の事案はありません")) || cells.len() < 5 {
            continue;
        }

        // 日時列から時刻部分のみ抽出
        let time = cells[0]
            .split_whitespace()
            .nth(1)
            .unwrap_or("")
            .to_string();

        let disaster_type = cells[2].clone();
        let address = format!("兵庫県小野市{}", &cells[4].replace('　', "").trim()[3..]);

        disaster_data.push(json!({
            "type": disaster_type,
            "address": address,
            "time": time
        }));
    }

    let output = json!({
        "jisx0402": "282189",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "小野市消防本部"
            }
        ],
        "disasters": disaster_data
    });

    // JSONファイルに書き出し
    let mut file = File::create("dist/282189.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 282189.json （小野市消防本部）");
    Ok(())
}