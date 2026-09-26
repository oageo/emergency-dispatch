use serde_json::json;
use std::fs::File;
use std::io::Write;
use scraper::{Html, Selector};
use crate::to_half_width;

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "www.tottori-seibukoiki.jp";
const GET_SOURCE: &str = "https://www.tottori-seibukoiki.jp/syobo/saigai/fr/saigai.html";

pub fn return_312045() -> Result<(), Box<dyn std::error::Error>> {
    println!("312045, 境港市（鳥取県西部広域行政管理組合消防局）");

    let config = HttpRequestConfig::new(HOST, GET_SOURCE).with_shift_jis(true);
    let body = get_source_with_config(&config)?;
    let document = Html::parse_document(&body);

    let mut disaster_data = vec![];

    // 各事案は<Br>（過去には<li>）区切りの1文（1テキストノード）になっている
    let body_selector = Selector::parse("body").unwrap();

    if let Some(body_element) = document.select(&body_selector).next() {
        for text in body_element.text() {
            // 全角数字を半角数字に変換し、空白・改行を除去
            let text: String = to_half_width(text).chars().filter(|c| !c.is_whitespace()).collect();

            // 災害がない場合
            if text.contains("災害は発生しておりません") {
                continue;
            }

            // 終了・鎮火した事案はスキップ（現在進行中の災害ではないため）
            // 例: "…で発生しました警戒事案は、活動を終了しました。"、"…で発生しました火災は、…鎮火しました。"
            if text.contains("終了") || text.contains("鎮火") {
                continue;
            }

            // 例: "・09月11日09時18分頃、西伯郡大山町高田西高田で警戒事案が発生し、消防車が出動しております。"
            let Some((date_time_part, rest)) = text.split_once("頃、") else {
                continue;
            };

            // "・09月11日09時18分" → "09:18"
            let Some((_, time_str)) = date_time_part.split_once('日') else {
                continue;
            };
            let time = time_str.replace("時", ":").replace("分", "");

            // 住所と災害種別を抽出
            let Some((location_and_type, _)) = rest.split_once("が発生し") else {
                continue;
            };
            let Some((location, disaster_type)) = location_and_type.rsplit_once('で') else {
                continue;
            };

            // 境港市の事案のみ（町村は郡名から始まる）
            if !location.starts_with("境港市") {
                continue;
            }
            let address = format!("鳥取県{}", location);

            // "警戒事案" → "警戒"、"火災" → そのまま
            let disaster_type = disaster_type.strip_suffix("事案").unwrap_or(disaster_type);

            if !time.is_empty() && !disaster_type.is_empty() {
                disaster_data.push(json!({
                    "type": disaster_type,
                    "address": address,
                    "time": time
                }));
            }
        }
    }

    let output = json!({
        "jisx0402": "312045",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "鳥取県西部広域行政管理組合消防局"
            }
        ],
        "disasters": disaster_data
    });

    let mut file = File::create("dist/312045.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 312045.json （境港市・鳥取県西部広域行政管理組合消防局）");
    Ok(())
}
