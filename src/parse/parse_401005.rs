use serde_json::json;
use std::fs::File;
use std::io::Write;
use crate::to_half_width; 

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "kitakyushu.xpressmail.jp";
const GET_SOURCE: &str = "http://kitakyushu.xpressmail.jp/saigai/navi/denbun.html";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    let config = HttpRequestConfig::new(HOST, GET_SOURCE);
    get_source_with_config(&config)
}

pub fn return_401005() -> Result<(), Box<dyn std::error::Error>> {
    println!("401005, 北九州市消防局");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);
    let selector = scraper::Selector::parse("html body").unwrap();
    let mut disaster_data = vec![];

    // 各要素を解析
    for element in document.select(&selector) {
        let text = to_half_width(&element.text().collect::<String>().trim().to_string()); // 全角数字を半角数字に変換
        // 「災害は、発生しておりません。」が含まれている場合はスキップ
        if text.contains("災害は、発生しておりません。") {
            continue;
        }

        // データを解析
        else if let Some((date_time, rest)) = text.split_once("頃") {
            if let Some((location, reason)) = rest.split_once("で") {
                // 時刻を抽出
                let time = date_time
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or("")
                    .replace("時", ":")
                    .replace("分", ""); // 時刻を整形

                // 住所を整形
                let address = format!(
                    "福岡県北九州市{}",
                    location.trim().replace('　', "").replace("付近", "") // 空白と「付近」を削除
                );

                // 災害種別を抽出
                let disaster_type = reason
                    .trim()
                    .split("のため")
                    .next()
                    .unwrap_or("") // 「のため」以降を削除
                    .trim_end_matches("活動"); // 「活動」を削除

                disaster_data.push(json!({
                    "type": disaster_type,
                    "address": address,
                    "time": time
                }));
            }
        }
    }

    let output = json!({
        "jisx0402": "401005",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "北九州市消防局"
            }
        ],
        "disasters": disaster_data
    });

    // JSONファイルに書き出し
    let mut file = File::create("dist/401005.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 401005.json （北九州市消防局）");
    Ok(())
}