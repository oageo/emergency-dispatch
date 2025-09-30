use serde_json::json;
use std::fs::File;
use std::io::Write;

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "www.syobo.city.kasugai.aichi.jp";
const GET_SOURCE: &str = "http://www.syobo.city.kasugai.aichi.jp/syobo/real/kasai.html";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    let config = HttpRequestConfig::new(HOST, GET_SOURCE).with_shift_jis(true);
    get_source_with_config(&config)
}

pub fn return_232068() -> Result<(), Box<dyn std::error::Error>> {
    println!("232068, 春日井市消防本部");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);
    let selector = scraper::Selector::parse("ul").unwrap();
    let mut disaster_data = vec![];

    // 最初の<ul>要素を取得（現在発生している災害の部分）
    if let Some(ul_element) = document.select(&selector).next() {
        let li_selector = scraper::Selector::parse("li").unwrap();

        // 各<li>要素を解析
        for element in ul_element.select(&li_selector) {
            let text = element.text().collect::<String>().trim().to_string();

            // 現在災害が発生していない場合は処理しない
            if text.contains("現在、火災等の災害は発生していません") {
                break;
            }

            // 「〇月〇日 〇時〇分頃　春日井市〇〇付近で、〇〇が発生中です。」形式を解析
            if text.contains("が発生中です") && text.contains("春日井市") {
                // 「頃　春日井市」で分割
                if let Some((date_time_part, rest)) = text.split_once("頃　春日井市") {
                    if let Some((address, disaster_part)) = rest.split_once("付近で、") {
                        // 時間部分を抽出（例：「09月10日 17時31分」→「17:31」）
                        let time = date_time_part.split_whitespace()
                            .nth(1)
                            .unwrap_or("")
                            .replace("時", ":")
                            .replace("分", "");

                        // 災害種別を抽出（「高所事故救助が発生中です。」→「高所事故救助」）
                        let disaster_type = disaster_part
                            .replace("が発生中です。", "")
                            .trim()
                            .to_string();

                        let address = format!("愛知県春日井市{}", address.trim());

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

    let output = json!({
        "jisx0402": "232068",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "春日井市消防本部"
            }
        ],
        "disasters": disaster_data
    });

    // JSONファイルに書き出し
    let mut file = File::create("dist/232068.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 232068.json （春日井市消防本部）");
    Ok(())
}