use serde_json::json;
use std::fs::File;
use std::io::Write;
use scraper::{Html, Selector};
use crate::to_half_width;

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "www.otsu119.jp";
const GET_SOURCE: &str = "http://www.otsu119.jp/fire/saigai/saigaiPc.html";

pub fn return_252018() -> Result<(), Box<dyn std::error::Error>> {
    println!("252018, 大津市消防局");

    let config = HttpRequestConfig::new(HOST, GET_SOURCE);
    let body = get_source_with_config(&config)?;
    let document = Html::parse_document(&body);

    let mut disaster_data = vec![];

    // div#col_main 内の最初の ul のみを処理（現在発生している災害）
    let ul_selector = Selector::parse("div#col_main ul").unwrap();
    let li_selector = Selector::parse("li").unwrap();

    if let Some(first_ul) = document.select(&ul_selector).next() {
        for li_element in first_ul.select(&li_selector) {
            let text = li_element.text().collect::<String>();

            // 全角数字を半角数字に変換
            let text = to_half_width(&text);

            // 半角スペース・全角スペースを削除
            let text = text.replace(" ", "").replace("　", "");

            // スキップ条件
            if text.contains("現在、火事などの災害は発生していません")
                || text.contains("災害は発生していません")
                || text.trim().is_empty() {
                continue;
            }

            // 「HH時MM分頃、大津市[住所]で[災害種別]が発生し、消防車が出動しています。」の形式でパース
            if let Some((time_part, rest)) = text.split_once("頃、") {
                // 時刻を抽出（「HH時MM分」→「HH:MM」）
                let time = time_part
                    .trim()
                    .replace("時", ":")
                    .replace("分", "");

                // 住所と災害種別を抽出
                if let Some((address_part, type_part)) = rest.split_once("で") {
                    let address = address_part.trim();

                    // 大津市の住所に滋賀県を追加
                    let address = if address.starts_with("大津市") {
                        format!("滋賀県{}", address)
                    } else {
                        address.to_string()
                    };

                    // 災害種別を抽出（「が発生し」以降を削除）
                    let disaster_type = if let Some((disaster, _)) = type_part.split_once("が発生し") {
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

    let output = json!({
        "jisx0402": "252018",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "大津市消防局"
            }
        ],
        "disasters": disaster_data
    });

    let mut file = File::create("dist/252018.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 252018.json （大津市消防局）");
    Ok(())
}
