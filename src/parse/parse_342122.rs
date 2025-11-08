use serde_json::json;
use std::fs::File;
use std::io::Write;

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "higashihiroshima-fd-119.jp";
const GET_SOURCE: &str = "http://higashihiroshima-fd-119.jp/";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    let config = HttpRequestConfig::new(HOST, GET_SOURCE).with_shift_jis(true);
    get_source_with_config(&config)
}

pub fn return_342122() -> Result<(), Box<dyn std::error::Error>> {
    println!("342122, 東広島市（東広島市消防局）");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);

    // 災害情報テーブルを探す
    let table_selector = scraper::Selector::parse("table[border=\"1\"] tr").unwrap();
    let mut disaster_data = vec![];

    // テーブルの行を処理（ヘッダー行をスキップ）
    let mut is_header = true;
    for row in document.select(&table_selector) {
        if is_header {
            is_header = false;
            continue;
        }

        let td_selector = scraper::Selector::parse("td").unwrap();
        let cells: Vec<String> = row
            .select(&td_selector)
            .map(|cell| cell.text().collect::<String>().trim().to_string())
            .collect();

        // セルが5つある場合のみ処理（日時、災害種別、災害区分、状況、場所）
        if cells.len() >= 5 {
            let date_time = &cells[0];
            let disaster_type = &cells[1];
            let disaster_category = &cells[2];
            let location = &cells[4];

            // 東広島市の災害のみをフィルタリング
            if location.contains("東広島市") {
                // 時刻を抽出（"2023/11/28 17:04" -> "17:04"）
                let time = date_time
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or("")
                    .to_string();

                // 災害種別と災害区分を結合
                let full_disaster_type = if disaster_category.is_empty() {
                    disaster_type.to_string()
                } else {
                    format!("{}-{}", disaster_type, disaster_category)
                };

                // 場所に「広島県」が含まれていない場合は追加
                let full_address = if location.contains("広島県") {
                    location.to_string()
                } else {
                    format!("広島県{}", location)
                };

                disaster_data.push(json!({
                    "type": full_disaster_type,
                    "address": full_address,
                    "time": time
                }));
            }
        }
    }

    let output = json!({
        "jisx0402": "342122",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "東広島市消防局"
            }
        ],
        "disasters": disaster_data
    });

    // JSONファイルに書き出し
    let mut file = File::create("dist/342122.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 342122.json （東広島市・東広島市消防局）");
    Ok(())
}
