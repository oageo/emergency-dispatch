use serde_json::json;
use std::fs::File;
use std::io::Write;
use crate::to_half_width; 

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "www.city.kashiwa.lg.jp";
const GET_SOURCE: &str = "https://www.city.kashiwa.lg.jp/fdk/disaster/index.html";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    let config = HttpRequestConfig::new(HOST, GET_SOURCE).with_shift_jis(true);
    get_source_with_config(&config)
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

        // 「現在、管内で災害は発生しておりません」または「終了」、「鎮火」が含まれている場合はスキップ
        if text.contains("現在、管内で災害は発生しておりません") || text.contains("終了") || text.contains("鎮火") {
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

            // 全角括弧を除去する安全な方法
            if addr.contains('（') && addr.contains('）') {
                let parts: Vec<&str> = addr.split('（').collect();
                if parts.len() >= 2 {
                    let before_bracket = parts[0];
                    let after_parts: Vec<&str> = parts[1].split('）').collect();
                    if after_parts.len() >= 2 {
                        addr = format!("{}{}", before_bracket, after_parts[1]);
                    } else {
                        addr = before_bracket.to_string();
                    }
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