use serde_json::json;
use std::fs::File;
use std::io::Write;

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "www.city.ono.hyogo.jp";
const GET_SOURCE: &str = "https://www.city.ono.hyogo.jp/section/Jian.html";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    let config = HttpRequestConfig::new(HOST, GET_SOURCE).with_shift_jis(true);
    get_source_with_config(&config)
}

pub fn return_282189() -> Result<(), Box<dyn std::error::Error>> {
    println!("282189, 小野市消防本部");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);
    let selector = scraper::Selector::parse("html body center table tbody tr td table tbody tr td table tbody tr").unwrap();
    let mut disaster_data = vec![];

    let mut rows = document.select(&selector);

    rows.next();
    // 2行目以降を解析
    for row in rows {
        let cells: Vec<String> = row
            .select(&scraper::Selector::parse("td").unwrap())
            .map(|cell| cell.text().collect::<String>().trim().to_string())
            .collect();

        // 空行や「現在発生中の事案はありません」などはスキップ
        if cells.iter().any(|cell| cell.contains("現在発生中の事案はありません")) || cells.len() < 5 {
            continue;
        }

        // 日時列から時刻部分のみ抽出
        let time = cells[0]
            .split_whitespace()
            .nth(1)
            .unwrap_or("")
            .to_string();

        let disaster_type = cells[2].clone();
        let cleaned_address = cells[4].replace('　', "");
        let address_part = cleaned_address.trim();
        let address = format!("兵庫県小野市{}", address_part.strip_prefix("小野市").unwrap_or(address_part));

        disaster_data.push(json!({
            "type": disaster_type,
            "address": address,
            "time": time
        }));
    }

    let output = json!({
        "jisx0402": "282189",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "小野市消防本部"
            }
        ],
        "disasters": disaster_data
    });

    // JSONファイルに書き出し
    let mut file = File::create("dist/282189.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 282189.json （小野市消防本部）");
    Ok(())
}