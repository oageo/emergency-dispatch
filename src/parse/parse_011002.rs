use serde_json::json;
use std::fs::File;
use std::io::Write;

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "www.119.city.sapporo.jp";
const GET_SOURCE: &str = "http://www.119.city.sapporo.jp/saigai/sghp.html";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    let config = HttpRequestConfig::new(HOST, GET_SOURCE);
    get_source_with_config(&config)
}

pub fn return_011002() -> Result<(), Box<dyn std::error::Error>> {
    println!("011002, 札幌市消防局");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);
    let selector = scraper::Selector::parse("html body.format_free div#tmp_wrapper div#tmp_wrapper2 div#tmp_wrapper3 div#tmp_wrap_main.column_lnavi div#tmp_main div.wrap_col_main div.col_main div#tmp_contents").unwrap();
    let mut disaster_data = vec![];
    if let Some(element) = document.select(&selector).next() {
        let text = element.text().collect::<Vec<_>>().join("\n");
        
        // 札幌市の部分のみを抽出
        if let Some(sapporo_start) = text.find("〇札幌市") {
            let after_sapporo = &text[sapporo_start..];
            // 札幌市の部分の終了点を見つける（江別市で終了）
            let sapporo_end = after_sapporo.find("〇江別市").unwrap_or(after_sapporo.len());
            let sapporo_text = &after_sapporo[..sapporo_end];
            
            // 災害がない場合のチェック
            if !sapporo_text.contains("現在出動中の災害はありません") {
                // 出動種別（●で始まる行）を処理
                let mut current_disaster_type = String::new();
                for line in sapporo_text.lines() {
                    let line = line.trim();
                    if line.starts_with("●") {
                        current_disaster_type = line.trim_start_matches('●').replace("出動", "").trim().to_string();
                    } else if line.starts_with("・") && !current_disaster_type.is_empty() {
                        // 出動場所の処理
                        let location_time = line.trim_start_matches('・').trim();
                        if let Some((location, time_part)) = location_time.rsplit_once('（') {
                            let time = time_part.trim_end_matches('）').replace("時", ":").replace("分", "");
                            let address = format!("北海道札幌市{}", location.trim());
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
        "jisx0402": "011002",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "札幌市消防局"
            }
        ],
        "disasters": disaster_data
    });

    // JSONファイルに書き出し
    let mut file = File::create("dist/011002.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 011002.json （札幌市消防局）");
    Ok(())
}