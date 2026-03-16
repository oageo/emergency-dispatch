use serde_json::json;
use std::fs::File;
use std::io::Write;
use crate::to_half_width;

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "www.city.komatsu.lg.jp";
const GET_SOURCE: &str = "https://www.city.komatsu.lg.jp/section/syoubou/index.html";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    let config = HttpRequestConfig::new(HOST, GET_SOURCE).with_shift_jis(true);
    get_source_with_config(&config)
}

pub fn return_172031() -> Result<(), Box<dyn std::error::Error>> {
    println!("172031, 小松市消防本部");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);
    let li_selector = scraper::Selector::parse("ul#FDstatus li").unwrap();
    let mut disaster_data = vec![];

    for li_element in document.select(&li_selector) {
        let text = to_half_width(&li_element.text().collect::<String>().trim().to_string());

        // 終了済みの出動はスキップ
        if text.contains("終了") || text.contains("鎮火") || text.contains("解除") {
            continue;
        }

        // 形式: "MM月DD日 HH時MM分頃に住所 地内で活動内容のため消防車が出動しました。"
        if let Some((date_time, rest)) = text.split_once("頃に") {
            // 時刻を抽出（date_time部分の最後のトークン "HH時MM分"）
            let time = date_time
                .split_whitespace()
                .last()
                .unwrap_or("")
                .replace("時", ":")
                .replace("分", "");

            // 住所と災害種別を抽出（"地内で"で分割）
            if let Some((location, disaster_info)) = rest.split_once("地内で") {
                let address = format!("石川県小松市{}", location.trim());

                // 災害種別を抽出
                let disaster_type = if disaster_info.contains("が発生し") {
                    disaster_info.split("が発生し").next().unwrap_or("").trim()
                } else {
                    disaster_info.split("のため消防車が出動").next().unwrap_or("").trim()
                };

                if !disaster_type.is_empty() && !time.is_empty() {
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
        "jisx0402": "172031",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "小松市消防本部"
            }
        ],
        "disasters": disaster_data
    });

    // JSONファイルに書き出し
    let mut file = File::create("dist/172031.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 172031.json （小松市消防本部）");
    Ok(())
}