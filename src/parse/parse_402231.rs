use serde_json::json;
use std::fs::File;
use std::io::Write;

use super::super::{get_source_with_config, to_half_width, HttpRequestConfig};

const HOST: &str = "m119.city.fukuoka.lg.jp";
const GET_SOURCE: &str = "https://m119.city.fukuoka.lg.jp/kasuhoku/hpinfo.html";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    let config = HttpRequestConfig::new(HOST, GET_SOURCE);
    get_source_with_config(&config)
}

pub fn return_402231() -> Result<(), Box<dyn std::error::Error>> {
    println!("402231, 古賀市（粕屋北部消防本部）");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);

    let selector = scraper::Selector::parse("dl.emergencyinfo dd").unwrap();
    let mut disaster_data = vec![];

    for element in document.select(&selector) {
        let text = element.text().collect::<String>().trim().to_string();

        // 「災害が発生しておりません」という文字列があった場合は終了
        if text.contains("災害が発生しておりません") {
            break;
        }

        // 古賀市の災害情報のみを処理
        if text.contains("古賀市") {
            // 例: １６：１７　古賀市　小竹６１９番付近に警戒のため、消防隊が出動しています。

            // 時刻を抽出（全角数字を半角に変換）
            let time = to_half_width(
                text
                    .split('　')
                    .next()
                    .unwrap_or("")
                    .trim()
            ).replace('：', ":");

            // 住所部分を抽出（「古賀市」から「に」または「で」まで）
            let address_part = if let Some(city_start) = text.find("古賀市") {
                let after_city = &text[city_start..];
                if let Some(end_pos) = after_city.find('に').or_else(|| after_city.find('で')) {
                    format!("福岡県{}", &after_city[..end_pos])
                } else {
                    "福岡県古賀市".to_string()
                }
            } else {
                "福岡県古賀市".to_string()
            };

            // 災害種別を抽出（「に」または「で」の後、「のため」まで）
            let disaster_type = if let Some(ni_pos) = text.find('に') {
                text[ni_pos + 'に'.len_utf8()..]
                    .split("のため")
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string()
            } else if let Some(de_pos) = text.find('で') {
                text[de_pos + 'で'.len_utf8()..]
                    .split("のため")
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string()
            } else {
                "不明".to_string()
            };

            disaster_data.push(json!({
                "type": disaster_type,
                "address": address_part,
                "time": time
            }));
        }
    }

    let output = json!({
        "jisx0402": "402231",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "粕屋北部消防本部"
            }
        ],
        "disasters": disaster_data
    });

    let mut file = File::create("dist/402231.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 402231.json （古賀市・粕屋北部消防本部）");
    Ok(())
}
