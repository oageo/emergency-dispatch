use serde_json::json;
use std::fs::File;
use std::io::Write;
use crate::to_half_width; 

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "www.m.shirei-saigai-jyouhou.sd.web-sanin.jp";
const GET_SOURCE: &str = "https://www.m.shirei-saigai-jyouhou.sd.web-sanin.jp/index.html";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    let config = HttpRequestConfig::new(HOST, GET_SOURCE).with_shift_jis(true);
    get_source_with_config(&config)
}

pub fn return_322016() -> Result<(), Box<dyn std::error::Error>> {
    println!("322016, 松江市消防本部");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);

    // 時刻を取得
    let time_selector = scraper::Selector::parse("html body table tbody tr td b font.TIME").unwrap();
    let time = document
        .select(&time_selector)
        .next()
        .map(|element| {
            to_half_width(&element.text().collect::<String>().trim().to_string())
                .replace("現在","")
                .split_whitespace()
                .nth(1)
                .unwrap_or("")
                .replace("時", ":")
                .replace("分", "") // 時刻を整形
        })
        .unwrap_or_else(|| "不明".to_string());

    // 災害情報を取得
    let selector = scraper::Selector::parse("html body div table.SGINFO tbody tr td").unwrap();
    let mut disaster_data = vec![];

    for element in document.select(&selector) {
        let text = to_half_width(&element.text().collect::<String>().trim().to_string()); // 全角数字を半角数字に変換

        // 「ただいま管内で災害は発生しておりません」または「ないことを確認し終了」が含まれている場合はスキップ
        if text.contains("ただいま管内で災害は発生しておりません") || text.contains("ないことを確認し終了") {
            continue;
        }

        // データを解析
        if let Some((location, reason)) = text.split_once("で") {
            let address = format!(
                "島根県松江市{}",
                location
                    .trim()
                    .replace('　', "") // 全角スペースを削除
                    .replace("ただいま", "") // 「ただいま」を削除
            );
            let reason = reason.trim();
            let disaster_type = if let Some((ty, _)) = reason.split_once("要請") {
                ty
            } else if let Some((ty, _)) = reason.split_once("が発生") {
                ty
            } else if let Some((ty, _)) = reason.split_once("のため") {
                ty
            } else {
                reason
            };
            let disaster_type = disaster_type.trim_end_matches("活動");

            disaster_data.push(json!({
                "type": disaster_type,
                "address": address,
                "time": time
            }));
        }
    }

    let output = json!({
        "jisx0402": "322016",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "松江市消防本部"
            }
        ],
        "disasters": disaster_data
    });

    // JSONファイルに書き出し
    let mut file = File::create("dist/322016.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 322016.json （松江市消防本部）");
    Ok(())
}