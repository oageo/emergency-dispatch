use serde_json::json;
use std::fs::File;
use std::io::Write;
use scraper::{Html, Selector};
use crate::to_half_width;

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "www.m-kouiki119.jp";
const GET_SOURCE: &str = "https://www.m-kouiki119.jp/wp-content/themes/mks/modules/jian.html";

pub fn return_202207() -> Result<(), Box<dyn std::error::Error>> {
    println!("202207, 安曇野市（松本広域消防局）");

    let config = HttpRequestConfig::new(HOST, GET_SOURCE);
    let body = get_source_with_config(&config)?;
    let document = Html::parse_document(&body);

    let mut disaster_data = vec![];

    // 現在発生している災害セクション（<div class="nowjian">）のみを処理
    let nowjian_selector = Selector::parse("div.nowjian").unwrap();

    if let Some(nowjian_element) = document.select(&nowjian_selector).next() {
        let text = nowjian_element.text().collect::<String>();

        // 全角数字を半角数字に変換
        let text = to_half_width(&text);

        // 半角スペース・全角スペースを削除
        let text = text.replace(" ", "").replace("　", "");

        // スキップ条件：災害がない場合
        if text.contains("現在、火災等の災害は発生していません")
            || text.contains("災害は発生していません") {
            // 災害なし - 空の配列を返す
        } else {
            // li要素を探す（災害がある場合）
            let li_selector = Selector::parse("li").unwrap();

            for li_element in nowjian_element.select(&li_selector) {
                let li_text = li_element.text().collect::<String>();

                // 全角数字を半角数字に変換
                let li_text = to_half_width(&li_text);

                // 半角スペース・全角スペースを削除
                let li_text = li_text.replace(" ", "").replace("　", "");

                // 安曇野市の災害のみをフィルタリング
                if li_text.contains("安曇野市") {
                    if let Some((date_time_part, rest)) = li_text.split_once("頃、") {
                        if let Some(time_str) = date_time_part.split("日").nth(1) {
                            let time = time_str
                                .replace("時", ":")
                                .replace("分", "");

                            if let Some((address_part, type_part)) = rest.split_once("付近で") {
                                let address = format!("長野県{}", address_part.trim());

                                let disaster_type = if let Some((disaster, _)) = type_part.split_once("が発生") {
                                    disaster.trim().to_string()
                                } else {
                                    type_part.trim().to_string()
                                };

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
            }
        }
    }

    let output = json!({
        "jisx0402": "202207",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "松本広域消防局"
            }
        ],
        "disasters": disaster_data
    });

    let mut file = File::create("dist/202207.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 202207.json （安曇野市）");
    Ok(())
}
