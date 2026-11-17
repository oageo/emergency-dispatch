use serde_json::json;
use std::fs::File;
use std::io::Write;
use crate::to_half_width;

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "www.shobo.koriyama.fukushima.jp";
const GET_SOURCE: &str = "https://www.shobo.koriyama.fukushima.jp/saigai/index.html";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    let config = HttpRequestConfig::new(HOST, GET_SOURCE);
    get_source_with_config(&config)
}

pub fn return_075221() -> Result<(), Box<dyn std::error::Error>> {
    println!("075221, 小野町（郡山地方広域消防組合）");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);

    // 災害情報を取得
    let selector = scraper::Selector::parse("div.def_box1").unwrap();
    let mut disaster_data = vec![];

    for element in document.select(&selector) {
        let text = element.inner_html();

        // 全角数字を半角数字に変換
        let text = to_half_width(&text);

        // 鎮火情報が含まれている場合はスキップ
        if text.contains("鎮火") {
            continue;
        }

        // <hr>で分割（前半: 時刻、後半: 災害種別と住所）
        if let Some((time_part, content_part)) = text.split_once("<hr>") {
            // 時刻を抽出（例: "2026年01月14日 12時02分頃" -> "12:02"）
            let time = if let Some(time_str) = time_part.split("頃").next() {
                // 最後の部分（「HH時MM分」）を取得
                let time_str = time_str.trim();
                if let Some(day_pos) = time_str.rfind("日") {
                    let after_day = &time_str[day_pos + "日".len()..];
                    after_day
                        .trim()
                        .replace("時", ":")
                        .replace("分", "")
                } else {
                    "".to_string()
                }
            } else {
                "".to_string()
            };

            // 災害種別と住所を「‐」で分割
            if let Some((disaster_type, address_raw)) = content_part.split_once("‐") {
                let disaster_type = disaster_type.trim().to_string();
                let address_raw = address_raw
                    .split("<br")
                    .next()
                    .unwrap_or("")
                    .trim();

                // 小野町の情報のみ処理
                if address_raw.contains("小野町") {
                    // 住所から「田村郡」とスペースを削除
                    let address_cleaned = address_raw
                        .replace("田村郡", "")
                        .replace(" ", "")
                        .replace("　", "");

                    // 福島県を追加
                    let address = if address_cleaned.starts_with("小野町") {
                        format!("福島県{}", address_cleaned)
                    } else {
                        address_cleaned.to_string()
                    };

                    if !time.is_empty() && !disaster_type.is_empty() && !address.is_empty() {
                        disaster_data.push(json!({
                            "type": disaster_type,
                            "address": address,
                            "time": time
                        }));
                    }
                }
            }
        }
    }

    let output = json!({
        "jisx0402": "075221",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "郡山地方広域消防組合消防本部"
            }
        ],
        "disasters": disaster_data
    });

    let mut file = File::create("dist/075221.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 075221.json （小野町・郡山地方広域消防組合）");
    Ok(())
}
