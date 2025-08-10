use serde_json::json;
use std::fs::File;
use std::io::Write;
use crate::to_half_width; // 全角数字を半角数字に変換する関数

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "disif.city.kyoto.lg.jp";
const GET_SOURCE: &str = "https://disif.city.kyoto.lg.jp/annai/main/";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    let config = HttpRequestConfig::new(HOST, GET_SOURCE).with_shift_jis(true);
    get_source_with_config(&config)
}

pub fn return_261009() -> Result<(), Box<dyn std::error::Error>> {
    println!("261009, 京都市消防局");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);
    let selector = scraper::Selector::parse("td.td_list").unwrap();
    let mut disaster_data = vec![];

    // 各<td>要素を解析
    for element in document.select(&selector) {
        let text = to_half_width(&element.text().collect::<String>().trim().to_string()); // 全角数字を半角数字に変換

        // 「消防車等が出動している災害はありません」が含まれている場合はスキップ
        if text.contains("消防車等が出動している災害はありません") {
            continue;
        }

        else {
            for line in text.split("<hr>") {
                // 複数の災害情報が1つのテキスト内に含まれる場合に対応
                for sub_line in line.split("【").filter(|s| !s.is_empty()) {
                    if let Some((disaster_type, details)) = sub_line.split_once("】") {
                        if let Some((date_time, location_details)) = details.split_once("頃、") {
                            // 高速道路の場合と通常の住所の場合を分岐
                            let (address, disaster_type) = if location_details.contains("高速道路") {
                                // 高速道路の場合
                                let highway_address = location_details
                                    .split("の災害に") // 「の災害に」以降を削除
                                    .next()
                                    .unwrap_or(location_details)
                                    .trim();
                                (highway_address.to_string(), disaster_type.trim().to_string())
                            } else if let Some((address, _)) = location_details.split_once("付近") {
                                // 通常の住所の場合
                                (
                                    format!("京都府京都市{}", address.trim()),
                                    disaster_type.trim().to_string(),
                                )
                            } else {
                                continue; // 解析できない場合はスキップ
                            };

                            // 時間を「日」の後から抽出
                            let time = date_time
                                .split('日') // 「日」で分割
                                .nth(1) // 「日」の後の部分を取得
                                .unwrap_or("")
                                .trim() // 前後の空白を削除
                                .replace("時", ":")
                                .replace("分", "");

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
        "jisx0402": "261009",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "京都市消防局"
            }
        ],
        "disasters": disaster_data
    });

    // JSONファイルに書き出し
    let mut file = File::create("dist/261009.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 261009.json （京都市消防局）");
    Ok(())
}