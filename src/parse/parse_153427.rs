use serde_json::json;
use std::fs::File;
use std::io::Write;
use crate::to_half_width;

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "www.tysogo.jp";
const GET_SOURCE: &str = "https://www.tysogo.jp/status.html";

// 弥彦村の大字リスト
const YAHIKO_OAZA: &[&str] = &[
    "麓", "村山", "観音寺", "弥彦", "走出", "上泉", "井田", "山岸",
    "山崎", "中山", "矢作", "平野", "えび穴", "魵穴", "大戸", "美山", "峰見"
];

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    let config = HttpRequestConfig::new(HOST, GET_SOURCE).with_shift_jis(true);
    get_source_with_config(&config)
}

fn is_yahiko_location(location: &str) -> bool {
    YAHIKO_OAZA.iter().any(|&oaza| location.contains(oaza))
}

pub fn return_153427() -> Result<(), Box<dyn std::error::Error>> {
    println!("153427, 弥彦村（燕・弥彦総合事務組合消防本部）");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);

    // 災害情報を取得
    let selector = scraper::Selector::parse("ul#FDstatus li").unwrap();
    let mut disaster_data = vec![];

    for element in document.select(&selector) {
        // 全角数字を半角数字に変換
        let text = to_half_width(&element.text().collect::<String>())
            .replace('　', "")
            .replace(' ', "")
            .trim()
            .to_string();

        // 「（終了）」が含まれている場合は処理を中断
        if text.contains("（終了）") {
            continue;
        }

        // フォーマット: "01月14日12時32分頃に吉田本所地内で救急支援のため消防車が出動しました。"
        if let Some(time_location_part) = text.split("頃に").nth(1) {
            // 時刻を抽出
            if let Some(time_part) = text.split("頃に").next() {
                // 最後の6文字（「12時32分」の形式）を取得
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

                // 場所と災害種別を抽出
                if let Some((location_part, disaster_part)) = time_location_part.split_once("地内で") {
                    let location = location_part.trim();

                    // 弥彦村の大字であるか確認（弥彦村の情報のみ処理）
                    if is_yahiko_location(location) {
                        // 災害種別を抽出（「地内で」以降、「のため消防車が出動しました」まで）
                        let disaster_type = disaster_part
                            .split("のため消防車が出動しました")
                            .next()
                            .unwrap_or("")
                            .replace("。", "")
                            .trim()
                            .to_string();

                        let address = format!("新潟県西蒲原郡弥彦村{}", location);

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
        "jisx0402": "153427",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "燕・弥彦総合事務組合消防本部"
            }
        ],
        "disasters": disaster_data
    });

    let mut file = File::create("dist/153427.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 153427.json （弥彦村・燕・弥彦総合事務組合消防本部）");
    Ok(())
}
