use serde_json::json;
use std::fs::File;
use std::io::Write;
use crate::to_half_width;

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "www.seibu-saitama119.jp";
const GET_SOURCE: &str = "http://www.seibu-saitama119.jp/disaster/nishiiruma/saigai/pc/";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    let config = HttpRequestConfig::new(HOST, GET_SOURCE).with_shift_jis(true);
    get_source_with_config(&config)
}

pub fn return_113271() -> Result<(), Box<dyn std::error::Error>> {
    println!("113271, 越生町（西入間広域消防組合消防本部）");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);

    // 災害情報を取得
    let selector = scraper::Selector::parse("html body div table.SGINFO tbody tr td").unwrap();
    let mut disaster_data = vec![];

    for element in document.select(&selector) {
        // 全角数字を半角数字に変換
        let text = to_half_width(&element.text().collect::<String>())
            .replace('　', "")
            .replace(' ', "")
            .trim()
            .to_string();

        // 「発生しておりません」、「終了」、または「鎮火しました」が含まれている場合は処理を中断
        if text.contains("発生しておりません") || text.contains("終了") || text.contains("鎮火しました") {
            continue;
        }

        // 越生町の情報のみ処理
        if text.contains("越生町") {
            // 時刻を抽出（「月日時分頃」から「時分」を抽出）
            if let Some(time_part) = text.split("頃、").next() {
                // 最後の6文字（「22時33分」の形式）を取得
                let time = time_part
                    .chars()
                    .rev()
                    .take(6)
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect::<String>()
                    .replace("時", ":")
                    .replace("分", "");

                // 住所を抽出（「越生町」以降、「で」まで）
                if let Some(after_city) = text.split("越生町").nth(1) {
                    if let Some((location_part, disaster_part)) = after_city.split_once("で") {
                        // 住所を整形（「地内」を除去）
                        let address = format!("埼玉県入間郡越生町{}", location_part.replace("地内", "").trim());

                        // 災害種別を抽出（「で」以降、「が発生」まで）
                        let disaster_type = disaster_part
                            .split("が発生")
                            .next()
                            .unwrap_or("")
                            .trim()
                            .to_string();

                        if !disaster_type.is_empty() && !address.is_empty() && !time.is_empty() {
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
    }

    let output = json!({
        "jisx0402": "113271",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "西入間広域消防組合消防本部"
            }
        ],
        "disasters": disaster_data
    });

    let mut file = File::create("dist/113271.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 113271.json （越生町・西入間広域消防組合消防本部）");
    Ok(())
}
