use serde_json::json;
use std::fs::File;
use std::io::Write;
use crate::to_half_width; 

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "mama.city.ichikawa.chiba.jp";
const GET_SOURCE: &str = "http://mama.city.ichikawa.chiba.jp/saigai/index.html";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    let config = HttpRequestConfig::new(HOST, GET_SOURCE).with_shift_jis(true);
    get_source_with_config(&config)
}

pub fn return_122033() -> Result<(), Box<dyn std::error::Error>> {
    println!("122033, 市川市消防局");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);
    let selector = scraper::Selector::parse("font.fs-18-bk-b").unwrap();
    let mut disaster_data = vec![];

    // 各<font>要素を解析
    for element in document.select(&selector) {
        let text = to_half_width(&element.text().collect::<String>().trim().to_string()); // `to_half_width`を使用

        // 「終了」または「只今、災害は発生しておりません。」が含まれている場合はスキップ
        if text.contains("終了") || text.contains("只今、災害は発生しておりません。") {
            continue;
        }

        // データを解析 - 実際のフォーマット: 「９月１４日午後４時１４分頃市川市南行徳２丁目７番付近で救急連携活動のため消防車が出場しています。」
        if let Some(time_part) = text.split("頃市川市").next() {
            if let Some(location_part) = text.split("頃市川市").nth(1) {
                if let Some((address, type_part)) = location_part.split_once("付近で") {
                    // 時間の処理
                    let time = if time_part.contains("午後") {
                        let time_str = time_part
                            .split("午後")
                            .nth(1)
                            .unwrap_or("")
                            .trim();

                        // 「４時１４分」を「16:14」に変換
                        if let Some(hour_str) = time_str.split("時").next() {
                            let hour: u32 = hour_str.parse().unwrap_or(0);
                            let minute_str = time_str
                                .split("時")
                                .nth(1)
                                .unwrap_or("0")
                                .replace("分", "")
                                .trim()
                                .to_string();
                            let minute: u32 = minute_str.parse().unwrap_or(0);
                            let adjusted_hour = if hour == 12 { 12 } else { hour + 12 };
                            format!("{:02}:{:02}", adjusted_hour, minute)
                        } else {
                            "00:00".to_string()
                        }
                    } else if time_part.contains("午前") {
                        let time_str = time_part
                            .split("午前")
                            .nth(1)
                            .unwrap_or("")
                            .trim();

                        // 「９時１４分」を「09:14」に変換
                        if let Some(hour_str) = time_str.split("時").next() {
                            let hour: u32 = hour_str.parse().unwrap_or(0);
                            let minute_str = time_str
                                .split("時")
                                .nth(1)
                                .unwrap_or("0")
                                .replace("分", "")
                                .trim()
                                .to_string();
                            let minute: u32 = minute_str.parse().unwrap_or(0);
                            let adjusted_hour = if hour == 12 { 0 } else { hour };
                            format!("{:02}:{:02}", adjusted_hour, minute)
                        } else {
                            "00:00".to_string()
                        }
                    } else {
                        "00:00".to_string()
                    };

                    // 災害タイプの抽出
                    let disaster_type = type_part
                        .split("のため消防車が出場しています")
                        .next()
                        .unwrap_or(type_part)
                        .trim()
                        .replace("活動", "")
                        .trim()
                        .to_string();

                    let address = format!("千葉県市川市{}", address.trim());

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
        "jisx0402": "122033",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "市川市消防局"
            }
        ],
        "disasters": disaster_data
    });

    // JSONファイルに書き出し
    let mut file = File::create("dist/122033.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 122033.json （市川市消防局）");
    Ok(())
}