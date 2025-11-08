use serde_json::json;
use std::fs::File;
use std::io::Write;

use super::super::{get_source_with_config, to_half_width, HttpRequestConfig};

const HOST: &str = "yokosuka.fire.yokosuka.kanagawa.jp";
const GET_SOURCE: &str = "https://yokosuka.fire.yokosuka.kanagawa.jp/saigai/index.html";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    let config = HttpRequestConfig::new(HOST, GET_SOURCE).with_shift_jis(true);
    get_source_with_config(&config)
}

pub fn return_142018() -> Result<(), Box<dyn std::error::Error>> {
    println!("142018, 横須賀市消防局");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);
    let selector = scraper::Selector::parse("table.SGINFO tr").unwrap();
    let mut disaster_data = vec![];

    for row in document.select(&selector) {
        let td_selector = scraper::Selector::parse("td").unwrap();
        let cells: Vec<String> = row
            .select(&td_selector)
            .map(|cell| cell.text().collect::<String>().trim().to_string())
            .collect();

        // 2つ目のセルにテキストがある場合のみ処理
        if cells.len() >= 2 && !cells[1].is_empty() {
            // 全角数字を半角数字に変換
            let text = to_half_width(&cells[1]);

            // 「横須賀市・三浦市内に災害は発生しておりません。」のメッセージはスキップ
            if text.contains("災害は発生しておりません") {
                continue;
            }

            // 横須賀市の災害のみをフィルタリング
            if text.contains("横須賀市") {
                // フォーマット: "08月09日21時29分頃、横須賀市公郷町2丁目11番付近で　救急活動　が発生し、消防隊が出動しています。"（全角数字は既に半角に変換済み）
                if let Some((date_time, rest)) = text.split_once("頃、横須賀市") {
                    if let Some((address, disaster_info)) = rest.split_once("付近で") {
                        // 日時から時刻を抽出
                        let time_parts: Vec<&str> = date_time.split("日").collect();
                        if time_parts.len() >= 2 {
                            let time = time_parts[1]
                                .replace("時", ":")
                                .replace("分", "")
                                .trim()
                                .to_string();

                            // 災害種別を抽出（「が発生し、消防隊が出動しています。」を除去）
                            let disaster_type = disaster_info
                                .split_once("が発生")
                                .map(|(t, _)| t.trim().to_string())
                                .unwrap_or_default();

                            let full_address = format!("神奈川県横須賀市{}", address.trim());

                            disaster_data.push(json!({
                                "type": disaster_type,
                                "address": full_address,
                                "time": time
                            }));
                        }
                    }
                }
            }
        }
    }

    let output = json!({
        "jisx0402": "142018",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "横須賀市消防局"
            }
        ],
        "disasters": disaster_data
    });

    // JSONファイルに書き出し
    let mut file = File::create("dist/142018.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 142018.json （横須賀市消防局）");
    Ok(())
}
