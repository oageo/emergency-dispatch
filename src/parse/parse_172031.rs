use serde_json::json;
use std::fs::File;
use std::io::Write;
use crate::to_half_width;

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "www.kfd119.jp";
const GET_SOURCE: &str = "http://www.kfd119.jp/fire/saigai/saigaipc.html";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    let config = HttpRequestConfig::new(HOST, GET_SOURCE).with_shift_jis(true);
    get_source_with_config(&config)
}

pub fn return_172031() -> Result<(), Box<dyn std::error::Error>> {
    println!("172031, 小松市消防本部");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);
    let selector = scraper::Selector::parse("div.panel-body").unwrap();
    let mut disaster_data = vec![];

    // 最初のpanel-body要素のみを取得（現在発生している災害の部分）
    if let Some(panel_element) = document.select(&selector).next() {
        let text = to_half_width(&panel_element.text().collect::<String>().trim().to_string());
        
        if text.contains("現在、火災等の災害は発生していません") {
            // 災害なしの場合は何もしない
        } 
        else if let Some((date_time, rest)) = text.split_once("頃、") {
            if let Some((location, disaster_info)) = rest.split_once("付近で") {
                // 時刻を抽出（date_time部分の最後の時分）
                let time = date_time
                    .split_whitespace()
                    .last()
                    .unwrap_or("")
                    .replace("時", ":")
                    .replace("分", "");
                
                // 災害種別を抽出（「が発生し」まで）
                let disaster_type = if let Some((ty, _)) = disaster_info.split_once("が発生し") {
                    ty.trim()
                } else if let Some((ty, _)) = disaster_info.split_once("事案が発生し") {
                    ty.trim() 
                } else {
                    disaster_info.split("、").next().unwrap_or(disaster_info).trim()
                };

                let address = format!("石川県小松市{}", location.trim());

                if !disaster_type.is_empty() && !address.is_empty() && !time.is_empty() {
                    disaster_data.push(json!({
                        "type": disaster_type,
                        "address": address,
                        "time": time
                    }));
                }
            }
        }
    }

    let output = json!({
        "jisx0402": "172031",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "小松市消防本部"
            }
        ],
        "disasters": disaster_data
    });

    // JSONファイルに書き出し
    let mut file = File::create("dist/172031.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 172031.json （小松市消防本部）");
    Ok(())
}