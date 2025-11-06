use serde_json::json;
use std::fs::File;
use std::io::Write;

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "www.sakatashobo.jp";
const GET_SOURCE: &str = "http://www.sakatashobo.jp/";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    let config = HttpRequestConfig::new(HOST, GET_SOURCE).with_shift_jis(true);
    get_source_with_config(&config)
}

pub fn return_063649() -> Result<(), Box<dyn std::error::Error>> {
    println!("063649, 庄内町消防本部（酒田地区広域行政組合消防本部）");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);

    // 災害情報テーブルを探す
    let table_selector = scraper::Selector::parse("table[border=\"1\"] tr").unwrap();
    let mut disaster_data = vec![];

    let mut is_disaster_table = false;

    for row in document.select(&table_selector) {
        let cells: Vec<String> = row
            .select(&scraper::Selector::parse("td").unwrap())
            .map(|cell| cell.text().collect::<String>().trim().to_string())
            .collect();

        // ヘッダー行を確認（覚知時刻、災害種別、災害区分、状況、住所）
        let headers: Vec<String> = row
            .select(&scraper::Selector::parse("th").unwrap())
            .map(|cell| cell.text().collect::<String>().trim().to_string())
            .collect();

        if headers.iter().any(|h| h.contains("覚知時刻")) {
            is_disaster_table = true;
            continue;
        }

        // 災害テーブルのデータ行を解析
        if is_disaster_table && cells.len() >= 5 {
            let time_str = &cells[0]; // "2025/07/20 08:15"
            let disaster_type = &cells[1]; // "その他"
            let disaster_category = &cells[2]; // "防災ヘリ支援"
            let address = &cells[4]; // "庄内町立谷沢字玉川"

            // 庄内町の災害のみをフィルタリング
            if address.starts_with("庄内町") {
                // 時刻を HH:MM 形式に変換
                let time = time_str
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or("")
                    .to_string();

                // 災害種別と災害区分を組み合わせ
                let full_type = if disaster_category.is_empty() {
                    disaster_type.clone()
                } else {
                    format!("{}（{}）", disaster_type, disaster_category)
                };

                let full_address = format!("山形県{}", address);

                disaster_data.push(json!({
                    "type": full_type,
                    "address": full_address,
                    "time": time
                }));
            }
        }
    }

    let output = json!({
        "jisx0402": "063649",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "酒田地区広域行政組合消防本部"
            }
        ],
        "disasters": disaster_data
    });

    // JSONファイルに書き出し
    let mut file = File::create("dist/063649.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 063649.json （庄内町消防本部（酒田地区広域行政組合消防本部））");
    Ok(())
}
