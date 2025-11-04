use serde_json::json;
use std::fs::File;
use std::io::Write;

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "www.om119.jp";
const GET_SOURCE: &str = "https://www.om119.jp/section/saigaiPc.html";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    let config = HttpRequestConfig::new(HOST, GET_SOURCE).with_shift_jis(true);
    get_source_with_config(&config)
}

pub fn return_273643() -> Result<(), Box<dyn std::error::Error>> {
    println!("273643, 河南町（大阪南消防組合）");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);
    let selector = scraper::Selector::parse("ul li span.item").unwrap();
    let mut disaster_data = vec![];

    for element in document.select(&selector) {
        let text = element.text().collect::<String>().trim().to_string();

        if text.contains("現在、火事などの災害は発生していません") {
            break;
        }

        if let Some((date_time, rest)) = text.split_once("ごろ、") {
            if let Some((location, disaster_info)) = rest.split_once("付近において、") {
                // 河南町の災害のみフィルタリング
                if !location.contains("河南町") {
                    continue;
                }

                let time = if let Some(time_part) = date_time.split("日").nth(1) {
                    time_part.replace("時", ":").replace("分", "")
                } else {
                    "00:00".to_string()
                };

                let disaster_type = disaster_info
                    .replace("の通報により出動中です。", "")
                    .trim()
                    .to_string();

                let address = if let Some(town_part) = location.split("町").nth(1) {
                    format!("大阪府河南町{}", town_part.trim())
                } else {
                    format!("大阪府{}", location.trim())
                };

                disaster_data.push(json!({
                    "type": disaster_type,
                    "address": address,
                    "time": time
                }));
            }
        }
    }

    let output = json!({
        "jisx0402": "273643",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "大阪南消防組合"
            }
        ],
        "disasters": disaster_data
    });

    let mut file = File::create("dist/273643.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 273643.json （河南町、大阪南消防組合）");
    Ok(())
}
