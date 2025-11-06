use serde_json::json;
use std::fs::File;
use std::io::Write;

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "www.om119.jp";
const GET_SOURCE: &str = "https://www.om119.jp/section/saigaiPc.html";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    let config = HttpRequestConfig::new(HOST, GET_SOURCE).with_shift_jis(true);
    get_source_with_config(&config)
}

pub fn return_272221() -> Result<(), Box<dyn std::error::Error>> {
    println!("272221, 羽曳野市（大阪南消防組合）");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);
    let selector = scraper::Selector::parse("ul").unwrap();
    let mut disaster_data = vec![];

    // 最初の<ul>要素のみを取得（現在発生している災害）
    if let Some(ul_element) = document.select(&selector).next() {
        let li_selector = scraper::Selector::parse("li span.item").unwrap();

        // 各<li>要素を解析
        for element in ul_element.select(&li_selector) {
            let text = element.text().collect::<String>().trim().to_string();

            // 羽曳野市の災害のみをフィルタリング
            if text.contains("羽曳野市") {
                // テキストを解析: "11月6日14時49分ごろ、羽曳野市高鷲４丁目付近において、救急車の応援の通報により出動中です。"
                let parts: Vec<&str> = text.split("、").collect();
                if parts.len() >= 3 {
                    // 時刻部分を抽出
                    let time_part = parts[0];
                    let time = time_part
                        .split("日")
                        .nth(1)
                        .unwrap_or("")
                        .replace("時", ":")
                        .replace("分ごろ", "");

                    // 住所部分を抽出
                    let address_part = parts[1];
                    let address = address_part
                        .replace("付近において", "")
                        .trim()
                        .to_string();
                    let full_address = format!("大阪府{}", address);

                    // 種別部分を抽出
                    let type_part = parts[2];
                    let disaster_type = type_part
                        .replace("の通報により出動中です。", "")
                        .replace("の通報により出動しました。", "")
                        .trim()
                        .to_string();

                    disaster_data.push(json!({
                        "type": disaster_type,
                        "address": full_address,
                        "time": time
                    }));
                }
            }
        }
    }

    let output = json!({
        "jisx0402": "272221",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "大阪南消防組合"
            }
        ],
        "disasters": disaster_data
    });

    // JSONファイルに書き出し
    let mut file = File::create("dist/272221.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 272221.json （羽曳野市）");
    Ok(())
}
