use serde_json::json;
use std::fs::File;
use std::io::Write;
use scraper::{Html, Selector};

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "www.kochi119.com";
const GET_SOURCE: &str = "http://www.kochi119.com/tosa/p/";

pub fn return_392057() -> Result<(), Box<dyn std::error::Error>> {
    println!("392057, 土佐市消防本部");

    let config = HttpRequestConfig::new(HOST, GET_SOURCE).with_shift_jis(true);
    let body = get_source_with_config(&config)?;
    let document = Html::parse_document(&body);

    // 「現在発生している災害」パネルの表のみを対象とする（最初に出現する表）
    // 2番目以降の表は「過去の災害経過情報」（既に終了・鎮火した過去情報）なので取得しない
    let table_selector = Selector::parse("div.panel-body table").unwrap();
    let td_selector = Selector::parse("tr td").unwrap();
    let mut disaster_data = vec![];

    if let Some(table_element) = document.select(&table_selector).next() {
        for td_element in table_element.select(&td_selector) {
            let text = td_element.text().collect::<String>();
            let lines = text
                .split('\n')
                .map(|line| line.trim())
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>();

            // 「現在、火事などの災害は発生していません。」の場合はスキップ
            if text.contains("現在、火事などの災害は発生していません") {
                continue;
            }

            let Some(address_index) = lines.iter().position(|line| line.contains("に出動中です")) else {
                continue;
            };
            if address_index == 0 {
                continue;
            }

            let address = lines[address_index]
                .split("付近に出動中です")
                .next()
                .unwrap_or("")
                .trim();
            let address = if address.starts_with("土佐市") {
                format!("高知県{}", address)
            } else {
                address.to_string()
            };

            let disaster_type = lines[address_index - 1].trim().to_string();

            let time = lines
                .iter()
                .find(|line| line.contains("時") && line.contains("分") && !line.contains("月"))
                .map(|line| line.replace("時", ":").replace("分", "").trim().to_string())
                .unwrap_or_default();

            if !time.is_empty() && !address.is_empty() && !disaster_type.is_empty() {
                disaster_data.push(json!({
                    "type": disaster_type,
                    "address": address,
                    "time": time
                }));
            }
        }
    }

    let output = json!({
        "jisx0402": "392057",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "土佐市消防本部"
            }
        ],
        "disasters": disaster_data
    });

    let mut file = File::create("dist/392057.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 392057.json （土佐市消防本部）");
    Ok(())
}
