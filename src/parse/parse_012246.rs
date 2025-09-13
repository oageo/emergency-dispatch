use serde_json::json;
use std::fs::File;
use std::io::Write;

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "www.119.city.sapporo.jp";
const GET_SOURCE: &str = "https://www.119.city.sapporo.jp/saigai/03/index.html";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    let config = HttpRequestConfig::new(HOST, GET_SOURCE).with_shift_jis(true);
    get_source_with_config(&config)
}

pub fn return_012246() -> Result<(), Box<dyn std::error::Error>> {
    println!("012246, 千歳市消防本部");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);
    let selector = scraper::Selector::parse("html body").unwrap();
    let mut disaster_data = vec![];
    if let Some(element) = document.select(&selector).next() {
        let text = element.text().collect::<Vec<_>>().join("\n");
        
        // 千歳市の出動情報部分のみを抽出
        if let Some(_) = text.find("◆現在の出動") {
            let after_start = text.split("◆現在の出動").nth(1).unwrap_or("");
            // 終了点を見つける（救急出動情報または「出動中の災害は以上です」で終了）
            let dispatch_text = if let Some(split_text) = after_start.split("◆救急出動情報").next() {
                if let Some(final_text) = split_text.split("出動中の災害は以上です").next() {
                    final_text
                } else {
                    split_text
                }
            } else {
                after_start
            };
            
            // 災害がない場合のチェック（「出動中の災害は以上です」は区切りなので除外しない）
            if !dispatch_text.contains("現在出動中の災害はありません") {
                // 出動種別（●で始まる行）を処理
                let mut current_disaster_type = String::new();
                for line in dispatch_text.lines() {
                    let line = line.trim();
                    if line.starts_with("●") {
                        current_disaster_type = line.trim_start_matches('●').replace("出動", "").trim().to_string();
                    } else if line.starts_with("・") && !current_disaster_type.is_empty() {
                        // 出動場所の処理
                        let location_time = line.trim_start_matches('・').trim();
                        if let Some((location, time_part)) = location_time.rsplit_once('（') {
                            let time = time_part.trim_end_matches('）').replace("時", ":").replace("分", "");
                            let address = format!("北海道{}", location.trim());
                            disaster_data.push(json!({
                                "type": current_disaster_type.clone(),
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
        "jisx0402": "012246",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "千歳市消防本部"
            }
        ],
        "disasters": disaster_data
    });

    // JSONファイルに書き出し
    let mut file = File::create("dist/012246.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 012246.json （千歳市消防本部）");
    Ok(())
}