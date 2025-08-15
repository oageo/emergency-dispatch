use serde_json::json;
use std::fs::File;
use std::io::Write;
use crate::to_half_width; 

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "fc23371220232011.web4.blks.jp";
const GET_SOURCE: &str = "http://fc23371220232011.web4.blks.jp/html/index.html";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    let config = HttpRequestConfig::new(HOST, GET_SOURCE).with_shift_jis(true);
    get_source_with_config(&config)
}

pub fn return_012025() -> Result<(), Box<dyn std::error::Error>> {
    println!("012025, 函館市消防本部");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);

    // 災害情報を取得
    let selector = scraper::Selector::parse("html body div table.SGINFO tbody tr td").unwrap();
    let mut disaster_data = vec![];

    for element in document.select(&selector) {
        let text = to_half_width(&element.text().collect::<String>().trim().to_string());
        // 「災害は発生しておりません」等が含まれている場合はスキップ
        if text.contains("災害は発生しておりません") || text.contains("ないことを確認し終了") {
            continue;
        }
        else if let Some((before, after)) = text.split_once("函館市") {
            // 時刻を抽出
            let time = before.chars().rev().take(6).collect::<String>().chars().rev().collect::<String>()
                .replace("時", ":").replace("分", "");

            // 住所と災害種別を抽出
            if let Some((location, reason)) = after.split_once("付近で") {
                let address = format!("北海道函館市{}", location.trim());
                
                // 災害種別を抽出（「のため」まで）
                let disaster_type = if let Some((ty, _)) = reason.split_once("のため") {
                    ty.trim()
                } else {
                    reason.split("、").next().unwrap_or(reason).trim()
                };

                if !disaster_type.is_empty() && !address.is_empty() && !time.is_empty() {
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
        "jisx0402": "012025",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "函館市消防本部"
            }
        ],
        "disasters": disaster_data
    });

    // JSONファイルに書き出し
    let mut file = File::create("dist/012025.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 012025.json （函館市消防本部）");
    Ok(())
}