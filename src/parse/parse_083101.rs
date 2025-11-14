use serde_json::json;
use std::fs::File;
use std::io::Write;
use crate::to_half_width;

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "www.ibaraki-sirei.jp";
const GET_SOURCE: &str = "http://www.ibaraki-sirei.jp/saigai/shirosato/annai_list.html";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    let config = HttpRequestConfig::new(HOST, GET_SOURCE).with_shift_jis(true);
    get_source_with_config(&config)
}

pub fn return_083101() -> Result<(), Box<dyn std::error::Error>> {
    println!("083101, 城里町消防本部");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);
    let selector = scraper::Selector::parse("div strong").unwrap();
    let mut disaster_data = vec![];

    for element in document.select(&selector) {
        let text = to_half_width(&element.text().collect::<String>())
            .replace('　', "")
            .replace(' ', "")
            .trim()
            .to_string();
        // 「発生はありません」が含まれている場合はスキップ
        if text.contains("発生はありません") {
            continue;
        }
        else if let Some((before, after)) = text.split_once("ころ城里町") {
            // 時刻
            let time = before.chars().rev().take(6).collect::<String>().chars().rev().collect::<String>()
                .replace("時", ":").replace("分", "");

            // 住所（「付近」まで）
            let location = if after.contains("付近") {
                let parts: Vec<&str> = after.split("付近").collect();
                if !parts.is_empty() {
                    format!("城里町{}付近", parts[0])
                } else {
                    format!("城里町{}", after)
                }
            } else {
                format!("城里町{}", after)
            };

            // 災害種別（「付近で」以降、「が発生し」まで）
            let disaster_type = if let Some(rest) = after.split("付近で").nth(1) {
                rest.split("が発生し").next().unwrap_or("").to_string()
            } else {
                "".to_string()
            };

            let disaster_type = disaster_type.trim();
            let address = format!("茨城県{}", location);

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
        "jisx0402": "083101",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "城里町消防本部"
            }
        ],
        "disasters": disaster_data
    });

    // JSONファイルに書き出し
    let mut file = File::create("dist/083101.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 083101.json （城里町消防本部）");
    Ok(())
}
