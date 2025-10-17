use serde_json::json;
use std::fs::File;
use std::io::Write;

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "tomakomai119.ec-net.jp";
const GET_SOURCE: &str = "http://tomakomai119.ec-net.jp/csv/fireguidance1_0.html";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    // Shift_JISエンコーディングでHTMLを取得
    let config = HttpRequestConfig::new(HOST, GET_SOURCE).with_shift_jis(true);
    get_source_with_config(&config)
}

pub fn return_012131() -> Result<(), Box<dyn std::error::Error>> {
    println!("012131, 苫小牧市消防本部");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);

    // spanタグ内の出動情報を取得
    let selector = scraper::Selector::parse("span").unwrap();
    let mut disaster_data = vec![];

    if let Some(span_element) = document.select(&selector).next() {
        let span_text = span_element.text().collect::<String>();

        // 各行をループ処理（複数の出動情報に対応）
        for line in span_text.lines() {
            let line = line.trim();

            // "・"で始まる行を出動情報として処理
            if line.starts_with("・") {
                let content = line.trim_start_matches("・");

                // "頃、"と"で"の両方が含まれている行のみ処理
                if content.contains("頃、") && content.contains("で") {
                    // "頃、"で分割して時刻部分と残りを取得
                    let time_part = content.split("頃、").next().unwrap_or("");
                    let rest = content.split("頃、").nth(1).unwrap_or("");

                    // 時刻をHH:MM形式に変換（例: "2022年09月25日07:24" -> "07:24"）
                    let time = if let Some(_) = time_part.rfind("日") {
                        let after_day = time_part.split("日").nth(1).unwrap_or("");
                        let time_str: String = after_day
                            .chars()
                            .filter(|c| c.is_ascii_digit() || *c == ':')
                            .collect();

                        if !time_str.contains(':') && time_str.len() >= 3 {
                            let hour = &time_str[..time_str.len() - 2];
                            let minute = &time_str[time_str.len() - 2..];
                            format!("{}:{}", hour, minute)
                        } else {
                            time_str
                        }
                    } else {
                        String::new()
                    };

                    // "で"で分割して住所部分と種別部分を取得
                    let address_part = rest.split("で").next().unwrap_or("");
                    let type_part = rest.split("で").nth(1).unwrap_or("");

                    // 住所を整形（例: "植苗付近" -> "北海道苫小牧市植苗付近"）
                    let address_trimmed = address_part.trim();
                    let address = if address_trimmed.starts_with("北海道") {
                        address_trimmed.to_string()
                    } else if address_trimmed.starts_with("苫小牧市") {
                        format!("北海道{}", address_trimmed)
                    } else {
                        format!("北海道苫小牧市{}", address_trimmed)
                    };

                    // 災害種別を抽出（末尾の定型文を削除）
                    let disaster_type = type_part
                        .replace("が発生しております。", "")
                        .replace("が発生しました。", "")
                        .trim()
                        .to_string();

                    // 必要な情報がすべて揃っている場合のみ追加
                    if !time.is_empty() && !address.is_empty() && !disaster_type.is_empty() {
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
        "jisx0402": "012131",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "苫小牧市消防本部"
            }
        ],
        "disasters": disaster_data
    });

    // JSONファイルに書き出し
    let mut file = File::create("dist/012131.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 012131.json （苫小牧市消防本部）");
    Ok(())
}