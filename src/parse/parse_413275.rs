use serde_json::json;
use std::fs::File;
use std::io::Write;

use super::super::{get_source_with_config, to_half_width, HttpRequestConfig};

const HOST: &str = "www.chubu.saga.saga.jp";
const GET_SOURCE: &str = "https://www.chubu.saga.saga.jp/disaster/index.html";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    let config = HttpRequestConfig::new(HOST, GET_SOURCE).with_shift_jis(true);
    get_source_with_config(&config)
}

pub fn return_413275() -> Result<(), Box<dyn std::error::Error>> {
    println!("413275, 吉野ヶ里町(佐賀広域消防局)");
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
            // 最初のセルに「●」マーカーがあるかチェック
            if cells.len() < 1 || !cells[0].contains("●") {
                continue;
            }

            let text = cells[1].clone();

            // 吉野ヶ里町の災害のみをフィルタリング
            if !text.contains("吉野ヶ里町") {
                continue;
            }

            // フォーマット: "１０：０３ごろ吉野ヶ里町田手でPA連携が発生しています。"
            if let Some((time_part, rest)) = text.split_once("ごろ吉野ヶ里町") {
                if let Some((address_part, disaster_info)) = rest.split_once("で") {
                    // 時刻を抽出（全角数字と全角コロンを半角に変換）
                    let time = to_half_width(time_part.trim())
                        .replace("：", ":")  // 全角コロン(U+FF1A)を半角コロン(U+003A)に変換
                        .trim()
                        .to_string();

                    // 住所を抽出（吉野ヶ里町名を追加、神埼郡を含める）
                    let address = address_part.trim();
                    let full_address = format!("佐賀県神埼郡吉野ヶ里町{}", address);

                    // 災害種別を抽出
                    let disaster_type = disaster_info
                        .split_once("が発生")
                        .map(|(t, _)| t.trim().to_string())
                        .unwrap_or_else(|| {
                            disaster_info
                                .replace("しています。", "")
                                .replace("しました。", "")
                                .trim()
                                .to_string()
                        });

                    disaster_data.push(json!({
                        "type": disaster_type,
                        "address": full_address,
                        "time": time
                    }));
                }
            }
        }
    }

    let output = json!({
        "jisx0402": "413275",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "佐賀広域消防局"
            }
        ],
        "disasters": disaster_data
    });

    // JSONファイルに書き出し
    let mut file = File::create("dist/413275.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 413275.json （吉野ヶ里町・佐賀広域消防局）");
    Ok(())
}
