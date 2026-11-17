use serde_json::json;
use std::fs::File;
use std::io::Write;
use scraper::{Html, Selector};
use crate::to_half_width;

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "www.chigasaki-samukawa119.info";
const GET_SOURCE: &str = "http://www.chigasaki-samukawa119.info/chigasaki-saigai/index.html";

pub fn return_143219() -> Result<(), Box<dyn std::error::Error>> {
    println!("143219, 寒川町（茅ヶ崎市消防本部）");

    let config = HttpRequestConfig::new(HOST, GET_SOURCE);
    let body = get_source_with_config(&config)?;
    let document = Html::parse_document(&body);

    // 茅ヶ崎市・寒川町の災害情報が1つの表に並ぶ。同じ表がPC表示とスマートフォン表示の
    // 2か所にあるため、最初の表（PC表示）のみを読む
    let table_selector = Selector::parse("table.SGINFO").unwrap();
    let td_selector = Selector::parse("td").unwrap();
    let Some(table_element) = document.select(&table_selector).next() else {
        return Err(format!("災害情報の表が見つかりません: {}", GET_SOURCE).into());
    };
    let mut disaster_data = vec![];

    for element in table_element.select(&td_selector) {
        // 全角数字を半角数字に変換し、住所などに含まれる空白を詰める
        let text: String = to_half_width(&element.text().collect::<String>())
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();

        // 災害がない場合
        if text.contains("災害は発生しておりません") {
            continue;
        }

        // 終了・鎮火した事案はスキップ（現在進行中の災害ではないため）
        if text.contains("終了") || text.contains("鎮火") {
            continue;
        }

        // 例: "11月07日12時57分頃、茅ヶ崎市甘沼付近で救急事案が発生し、消防車が出動しております。"
        let Some((date_time_part, rest)) = text.split_once("頃、") else {
            continue;
        };
        let Some((_, time_str)) = date_time_part.split_once('日') else {
            continue;
        };
        let time = time_str.replace("時", ":").replace("分", "");

        let Some((location, type_part)) = rest.split_once("付近で") else {
            continue;
        };
        let Some((disaster_type, _)) = type_part.split_once("が発生") else {
            continue;
        };

        // 寒川町の事案のみ
        if !location.starts_with("寒川町") {
            continue;
        }
        let address = format!("神奈川県{}", location);

        if !time.is_empty() && !disaster_type.is_empty() {
            disaster_data.push(json!({
                "type": disaster_type,
                "address": address,
                "time": time
            }));
        }
    }

    let output = json!({
        "jisx0402": "143219",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "茅ヶ崎市消防本部"
            }
        ],
        "disasters": disaster_data
    });

    let mut file = File::create("dist/143219.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 143219.json （寒川町・茅ヶ崎市消防本部）");
    Ok(())
}
