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

pub fn return_272213() -> Result<(), Box<dyn std::error::Error>> {
    println!("272213, 柏原市（大阪南消防組合）");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);
    let selector = scraper::Selector::parse("ul li span.item").unwrap();
    let mut disaster_data = vec![];

    for element in document.select(&selector) {
        let text = element.text().collect::<String>().trim().to_string();

        // 災害なしの判定
        if text.contains("現在、火事などの災害は発生していません") {
            break;
        }

        // "ごろ、" で時刻と残りを分割
        if let Some((date_time, rest)) = text.split_once("ごろ、") {
            // "付近において、" で住所と災害種別を分割
            if let Some((location, disaster_info)) = rest.split_once("付近において、") {
                // 柏原市の災害のみフィルタリング
                if !location.contains("柏原市") {
                    continue;
                }

                // 時刻抽出: "11月4日21時54分" → "21:54"
                let time = if let Some(time_part) = date_time.split("日").nth(1) {
                    time_part.replace("時", ":").replace("分", "")
                } else {
                    "00:00".to_string()
                };

                // 災害種別抽出: "救急車の応援の通報により出動中です。" → "救急車の応援"
                let disaster_type = disaster_info
                    .replace("の通報により出動中です。", "")
                    .trim()
                    .to_string();

                // 住所から市町村名を抽出
                let address = if let Some(city_part) = location.split("市").nth(1) {
                    format!("大阪府柏原市{}", city_part.trim())
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
        "jisx0402": "272213",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "大阪南消防組合"
            }
        ],
        "disasters": disaster_data
    });

    let mut file = File::create("dist/272213.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 272213.json （柏原市、大阪南消防組合）");
    Ok(())
}
