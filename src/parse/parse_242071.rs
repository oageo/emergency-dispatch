use serde_json::json;
use std::fs::File;
use std::io::Write;
use scraper::{Html, Selector};
use crate::to_half_width;

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "mcfcc119.jp";
const GET_SOURCE: &str = "https://mcfcc119.jp/Disaster-Information.php";

pub fn return_242071() -> Result<(), Box<dyn std::error::Error>> {
    println!("242071, 鈴鹿市消防本部");

    let config = HttpRequestConfig::new(HOST, GET_SOURCE);
    let body = get_source_with_config(&config)?;
    let document = Html::parse_document(&body);

    let mut disaster_data = vec![];

    // 津市・鈴鹿市・亀山市の災害発生状況が同じページ内の別々の<div>（balloon1〜3）にあり、
    // 鈴鹿市は balloon2。各事案は<BR>区切りの1文（1テキストノード）になっている
    let selector = Selector::parse("div#balloon2 div.disasters-region").unwrap();

    if let Some(region_element) = document.select(&selector).next() {
        for text in region_element.text() {
            // 全角数字を半角数字に変換し、半角・全角スペースを削除
            let text = to_half_width(text).replace(" ", "").replace("　", "");

            // 災害がない場合
            if text.contains("災害は発生していません") {
                continue;
            }

            // 終了した事案はスキップ（現在進行中の災害ではないため）
            if text.contains("終了") {
                continue;
            }

            // 例: "09月26日18時07分頃、津市久居烏木町地内で建物火災が発生し、消防隊が出動しています。"
            let Some((date_time_part, rest)) = text.split_once("頃、") else {
                continue;
            };

            // "09月26日18時07分" → "18:07"
            let Some((_, time_str)) = date_time_part.split_once('日') else {
                continue;
            };
            let time = time_str.replace("時", ":").replace("分", "");

            // 住所と災害種別を抽出
            let Some((address_part, type_part)) = rest
                .split_once("地内で")
                .or_else(|| rest.split_once("付近で"))
            else {
                continue;
            };
            let address = format!("三重県{}", address_part);

            let Some((disaster_type, _)) = type_part.split_once("が発生") else {
                continue;
            };

            if !time.is_empty() && !address_part.is_empty() && !disaster_type.is_empty() {
                disaster_data.push(json!({
                    "type": disaster_type,
                    "address": address,
                    "time": time
                }));
            }
        }
    }

    let output = json!({
        "jisx0402": "242071",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "鈴鹿市消防本部"
            }
        ],
        "disasters": disaster_data
    });

    let mut file = File::create("dist/242071.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 242071.json （鈴鹿市消防本部）");
    Ok(())
}
