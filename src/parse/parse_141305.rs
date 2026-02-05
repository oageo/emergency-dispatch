use serde_json::json;
use std::fs::File;
use std::io::Write;
use crate::to_half_width;

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "sc.city.kawasaki.jp";
const GET_SOURCE: &str = "https://sc.city.kawasaki.jp/saigai/index.htm";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    let config = HttpRequestConfig::new(HOST, GET_SOURCE).with_shift_jis(true);
    get_source_with_config(&config)
}

pub fn return_141305() -> Result<(), Box<dyn std::error::Error>> {
    println!("141305, 川崎市消防局");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);

    // テーブル内の災害情報を取得
    let selector = scraper::Selector::parse("table tbody tr td font").unwrap();
    let mut disaster_data = vec![];

    for element in document.select(&selector) {
        let text = element.text().collect::<String>();

        // 全角数字を半角数字に変換
        let text = to_half_width(&text);

        // スキップ条件
        if text.contains("市内に災害は発生しておりません")
            || text.contains("完了しました")
            || text.contains("横浜市")
            || text.trim().is_empty() {
            continue;
        }

        // 「頃」で分割して時刻部分を取得
        if let Some((time_part, rest)) = text.split_once("頃") {
            // 時刻を抽出（「M月D日　HH時MM分」→「HH:MM」）
            let time = if let Some(time_pos) = time_part.rfind("時") {
                let after_hour = &time_part[time_pos + "時".len()..];
                let hour_part = time_part[..time_pos]
                    .split(|c: char| !c.is_ascii_digit())
                    .filter(|s| !s.is_empty())
                    .last()
                    .unwrap_or("");

                let minute_part = after_hour
                    .split("分")
                    .next()
                    .unwrap_or("")
                    .trim();

                format!("{}:{}", hour_part, minute_part)
            } else {
                "".to_string()
            };

            // 場所と災害種別を抽出
            let rest = rest.trim();

            // 「付近より」または「付近で発生した」で分割
            let (location, disaster_type) = if let Some((loc, dtype)) = rest.split_once("付近より") {
                let dtype = dtype
                    .replace("の通報があり、消防車が出場しています", "")
                    .replace("火災", "")
                    .trim()
                    .to_string();
                let dtype = if dtype.is_empty() { "火災".to_string() } else { dtype };
                (loc.trim(), dtype)
            } else if let Some((loc, dtype)) = rest.split_once("付近で発生した") {
                let dtype = dtype
                    .split("は処理が完了しました")
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                (loc.trim(), dtype)
            } else {
                continue;
            };

            // 住所からスペースを削除
            let location = location
                .replace(" ", "")
                .replace("　", "");

            // 大田区・世田谷区の場合は東京都、それ以外は神奈川県川崎市
            let address = if location.contains("大田区") || location.contains("世田谷区") {
                format!("東京都{}", location)
            } else {
                format!("神奈川県川崎市{}", location)
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

    let output = json!({
        "jisx0402": "141305",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "川崎市消防局"
            }
        ],
        "disasters": disaster_data
    });

    let mut file = File::create("dist/141305.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 141305.json （川崎市消防局）");
    Ok(())
}
