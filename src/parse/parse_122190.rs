use serde_json::json;
use std::fs::File;
use std::io::Write;
use crate::to_half_width; 

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "chb1018.hs.plala.or.jp";
const GET_SOURCE: &str = "http://chb1018.hs.plala.or.jp/chiba119/Web/ichihara/annai_list.html";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    let config = HttpRequestConfig::new(HOST, GET_SOURCE).with_shift_jis(true);
    get_source_with_config(&config)
}

pub fn return_122190() -> Result<(), Box<dyn std::error::Error>> {
    println!("122190, 市原市消防局");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);
    let selector = scraper::Selector::parse("html body div strong").unwrap();
    let mut disaster_data = vec![];

    for element in document.select(&selector) {
        let text = to_half_width(&element.text().collect::<String>())
            .replace('　', "")
            .replace(' ', "")
            .trim()
            .to_string();
        // 「必ず火の元の点検をしましょう」が含まれている場合はスキップ
        if text.contains("必ず火の元の点検をしましょう") {
            continue;
        }
        else if let Some((before, after)) = text.split_once("頃、市原市") {
            // 時刻
            let time = before.chars().rev().take(6).collect::<String>().chars().rev().collect::<String>()
                .replace("時", ":").replace("分", "");

            // 住所（「番」まで）
            let location = if let Some(addr_end) = after.find("番") {
                format!("市原市{}番", &after[..addr_end])
            } else {
                format!("市原市{}", after)
            };

            // 災害種別（「消防隊が」以降、「活動」まで）
            let disaster_type = if let Some(rest) = after.split("消防隊が").nth(1) {
                rest.split("活動").next().unwrap_or("").to_string()
            } else {
                "".to_string()
            };

            let disaster_type = disaster_type.trim();
            let address = format!("千葉県{}", location);

            if !disaster_type.is_empty() && !address.is_empty() && !time.is_empty() {
                disaster_data.push(json!({
                    "type": disaster_type,
                    "address": address,
                    "time": time
                }));
            }
        }
    }

    let output = json!({
        "jisx0402": "122190",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "市原市消防局"
            }
        ],
        "disasters": disaster_data
    });

    // JSONファイルに書き出し
    let mut file = File::create("dist/122190.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 122190.json （市原市消防局）");
    Ok(())
}