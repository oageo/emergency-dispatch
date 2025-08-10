use serde_json::json;
use std::fs::File;
use std::io::Write;
use crate::to_half_width; 

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "www.nagaoka-fd.com";
const GET_SOURCE: &str = "http://www.nagaoka-fd.com/fire/saigai/saigaipc.html";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    let config = HttpRequestConfig::new(HOST, GET_SOURCE).with_shift_jis(true);
    get_source_with_config(&config)
}

pub fn return_152021() -> Result<(), Box<dyn std::error::Error>> {
    println!("152021, 長岡市消防本部");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);
    let selector = scraper::Selector::parse("html body center table tbody tr td ul").unwrap();
    let li_selector = scraper::Selector::parse("li span").unwrap();
    let mut disaster_data = vec![];

    // 最初の<ul>要素のみを取得
    if let Some(ul_element) = document.select(&selector).next() {
        // 各<li>要素を解析
        for element in ul_element.select(&li_selector) {
            let text = to_half_width(&element.text().collect::<String>().trim().to_string()); // 全角数字を半角数字に変換

            // 「現在、災害は発生しておりません」が含まれている場合はスキップ
            if text.contains("現在、災害は発生しておりません") {
                break;
            }

            // データを解析
            if let Some((date_time, rest)) = text.split_once("　長岡市") {
                if let Some((address, disaster_type)) = rest.split_once("に") {
                    let time = date_time
                        .split_whitespace()
                        .nth(1)
                        .unwrap_or("")
                        .replace("時", ":")
                        .replace("分", "");
                    let disaster_type = disaster_type
                        .trim()
                        .split("のため") // 「のため」以降を削除
                        .next()
                        .unwrap_or("")
                        .replace("活動", ""); // 「活動」を削除
                    let address = format!("新潟県長岡市{}", address.trim().replace(' ', "")); // スペースを詰める

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
        "jisx0402": "152021",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "長岡市消防本部"
            }
        ],
        "disasters": disaster_data
    });

    // JSONファイルに書き出し
    let mut file = File::create("dist/152021.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 152021.json （長岡市消防本部）");
    Ok(())
}