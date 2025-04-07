use reqwest::blocking::Client;
use reqwest::header::{HeaderMap};
use serde_json::json;
use std::fs::File;
use std::io::Write;
use encoding_rs::SHIFT_JIS; // Shift_JISエンコーディング用
use crate::to_half_width; // 全角数字を半角数字に変換する関数

// `ACCESS_UA`をlib.rsから参照
use super::super::ACCESS_UA;

const HOST: &str = "disif.city.kyoto.lg.jp";
const ACCEPT: &str = "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8";
const ACCEPT_LANGUAGE: &str = "ja,en-US;q=0.7,en;q=0.3";
const CONNECTION: &str = "keep-alive";
const CONTENT_TYPE: &str = "application/x-www-form-urlencoded";
const GET_SOURCE: &str = "https://disif.city.kyoto.lg.jp/annai/main/";

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

pub fn return_261009() -> Result<(), Box<dyn std::error::Error>> {
    println!("261009, 京都市消防局");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);
    let selector = scraper::Selector::parse("td.td_list").unwrap();
    let mut disaster_data = vec![];

    // 各<td>要素を解析
    for element in document.select(&selector) {
        let text = to_half_width(&element.text().collect::<String>().trim().to_string()); // 全角数字を半角数字に変換

        // 「消防車等が出動している災害はありません」が含まれている場合はスキップ
        if text.contains("消防車等が出動している災害はありません") {
            continue;
        }

        else {
            for line in text.split("<hr>") {
                // 複数の災害情報が1つのテキスト内に含まれる場合に対応
                for sub_line in line.split("【").filter(|s| !s.is_empty()) {
                    if let Some((disaster_type, details)) = sub_line.split_once("】") {
                        if let Some((date_time, location_details)) = details.split_once("頃、") {
                            // 高速道路の場合と通常の住所の場合を分岐
                            let (address, disaster_type) = if location_details.contains("高速道路") {
                                // 高速道路の場合
                                let highway_address = location_details
                                    .split("の災害に") // 「の災害に」以降を削除
                                    .next()
                                    .unwrap_or(location_details)
                                    .trim();
                                (highway_address.to_string(), disaster_type.trim().to_string())
                            } else if let Some((address, _)) = location_details.split_once("付近") {
                                // 通常の住所の場合
                                (
                                    format!("京都府京都市{}", address.trim()),
                                    disaster_type.trim().to_string(),
                                )
                            } else {
                                continue; // 解析できない場合はスキップ
                            };

                            // 時間を「日」の後から抽出
                            let time = date_time
                                .split('日') // 「日」で分割
                                .nth(1) // 「日」の後の部分を取得
                                .unwrap_or("")
                                .trim() // 前後の空白を削除
                                .replace("時", ":")
                                .replace("分", "");

                            disaster_data.push(json!({
                                "type": disaster_type,
                                "address": address,
                                "time": time
                            }));
                        }
                    }
                }
            }
        }
    }

    let output = json!({
        "jisx0402": "261009",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "京都市消防局"
            }
        ],
        "disasters": disaster_data
    });

    // JSONファイルに書き出し
    let mut file = File::create("dist/261009.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 261009.json （京都市消防局）");
    Ok(())
}