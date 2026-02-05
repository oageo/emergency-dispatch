use serde_json::json;
use std::fs::File;
use std::io::Write;
use crate::to_half_width;

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "www.city.higashiosaka.lg.jp";
const GET_SOURCE: &str = "https://www.city.higashiosaka.lg.jp/saigai/saigai.html";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    let config = HttpRequestConfig::new(HOST, GET_SOURCE).with_shift_jis(true);
    get_source_with_config(&config)
}

pub fn return_272272() -> Result<(), Box<dyn std::error::Error>> {
    println!("272272, 東大阪市消防局");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);

    // 災害速報を取得
    let selector = scraper::Selector::parse("p.ema_new font#edit06").unwrap();
    let mut disaster_data = vec![];

    if let Some(element) = document.select(&selector).next() {
        let text = element.text().collect::<String>();

        // 全角数字を半角数字に変換
        let text = to_half_width(&text);

        // 「現在災害は発生しておりません」が含まれている場合はスキップ
        if text.contains("現在災害は発生しておりません") {
            // 災害なし
        } else {
            // 「HH:MM:SSごろ、東大阪市[住所]付近で[災害種別]が発生し、現在、消防車が出動しています。」
            // の形式でパースする

            // 「ごろ、」で分割
            if let Some((time_part, rest)) = text.split_once("ごろ、") {
                // 時刻を「HH:MM:SS」→「HH:MM」に変換
                let time = if let Some(colon_pos) = time_part.rfind(':') {
                    // 最後のコロンより前を取得（秒を除外）
                    let without_seconds = &time_part[..colon_pos];
                    without_seconds.trim().to_string()
                } else {
                    time_part.trim().to_string()
                };

                // 「で」で分割して住所と災害種別を取得
                if let Some((address_part, type_part)) = rest.split_once("で") {
                    let address_raw = address_part.trim();

                    // 住所からスペースを削除
                    let address = address_raw
                        .replace(" ", "")
                        .replace("　", "");

                    // 災害種別を抽出（「が発生し」以降を削除）
                    let disaster_type = if let Some((disaster, _)) = type_part.split_once("が発生し") {
                        disaster.trim().to_string()
                    } else {
                        type_part.trim().to_string()
                    };

                    // 必要な情報がすべて揃っている場合のみ追加
                    if !time.is_empty() && !address.is_empty() && !disaster_type.is_empty() {
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
        "jisx0402": "272272",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "東大阪市消防局"
            }
        ],
        "disasters": disaster_data
    });

    let mut file = File::create("dist/272272.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 272272.json （東大阪市消防局）");
    Ok(())
}
