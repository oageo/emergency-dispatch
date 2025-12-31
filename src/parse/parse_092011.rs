use serde_json::json;
use std::fs::File;
use std::io::Write;
use scraper::{Html, Selector};
use crate::to_half_width;

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "utsunomiya.mwjp.jp";
const LIST_URL: &str = "https://utsunomiya.mwjp.jp/mobile/index.cgi?page=119";

pub fn return_092011() -> Result<(), Box<dyn std::error::Error>> {
    println!("092011, 宇都宮市消防局");

    // ステップ1: 一覧ページを取得
    let config = HttpRequestConfig::new(HOST, LIST_URL).with_shift_jis(true);
    let list_body = get_source_with_config(&config)?;
    let list_document = Html::parse_document(&list_body);

    // ステップ2: 一覧ページからリンクを抽出（「終了」を含まないもののみ）
    let link_selector = Selector::parse("a").unwrap();
    let mut disaster_data = vec![];

    for link_element in list_document.select(&link_selector) {
        let link_text = link_element.text().collect::<String>();

        // 「終了」が含まれているリンクはスキップ
        if link_text.contains("終了") {
            continue;
        }

        // 「発生」が含まれているリンクのみ処理
        if !link_text.contains("発生") {
            continue;
        }

        if let Some(href) = link_element.value().attr("href") {
            // 相対URLを絶対URLに変換
            let detail_url = if href.starts_with("http") {
                href.to_string()
            } else {
                format!("https://{}/{}", HOST, href)
            };

            // ステップ3: 詳細ページを取得
            let detail_config = HttpRequestConfig::new(HOST, &detail_url).with_shift_jis(true);
            match get_source_with_config(&detail_config) {
                Ok(detail_body) => {
                    let detail_document = Html::parse_document(&detail_body);

                    // ステップ4: 詳細ページから情報を抽出
                    // まず全角数字を半角に変換
                    let body_text = to_half_width(&detail_document.root_element().text().collect::<String>());

                    // 指令時刻を抽出: "指令時刻：31日22時15分" → "22:15"
                    let time = if let Some(time_part) = body_text.split("指令時刻：").nth(1) {
                        let time_str = time_part.split("災害住所：").next().unwrap_or("");
                        // "31日22時15分" から時刻部分のみ抽出
                        let time_only = time_str
                            .split("日")
                            .nth(1)
                            .unwrap_or("")
                            .trim();
                        time_only
                            .replace("時", ":")
                            .replace("分", "")
                            .trim()
                            .to_string()
                    } else {
                        "不明".to_string()
                    };

                    // 災害住所を抽出: "災害住所：宇都宮市上籠谷町地内"
                    let address = if let Some(addr_part) = body_text.split("災害住所：").nth(1) {
                        let addr_str = addr_part
                            .split("指令目標：")
                            .next()
                            .unwrap_or("")
                            .trim()
                            .replace("地内", ""); // 「地内」を除去
                        // 既に「宇都宮市」が含まれているので、栃木県を追加
                        if addr_str.starts_with("宇都宮市") {
                            format!("栃木県{}", addr_str)
                        } else {
                            addr_str.to_string()
                        }
                    } else {
                        continue; // 住所が取得できない場合はスキップ
                    };

                    // 災害区分を抽出: "災害区分：交通油漏れ"
                    let disaster_type = if let Some(type_part) = body_text.split("災害区分：").nth(1) {
                        // "交通油漏れ消防車が出動しました。" → "交通油漏れ"
                        type_part
                            .split("消防車が出動しました")
                            .next()
                            .unwrap_or("")
                            .trim()
                            .to_string()
                    } else {
                        "不明".to_string()
                    };

                    disaster_data.push(json!({
                        "type": disaster_type,
                        "address": address,
                        "time": time
                    }));
                },
                Err(e) => {
                    eprintln!("  [詳細ページ取得失敗] {}: {}", detail_url, e);
                    continue;
                }
            }
        }
    }

    let output = json!({
        "jisx0402": "092011",
        "source": [
            {
                "url": LIST_URL,
                "name": "宇都宮市消防局"
            }
        ],
        "disasters": disaster_data
    });

    let mut file = File::create("dist/092011.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 092011.json （宇都宮市消防局）");
    Ok(())
}
