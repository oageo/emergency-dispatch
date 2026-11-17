use serde_json::json;
use std::fs::File;
use std::io::Write;
use scraper::{Html, Selector};
use crate::to_half_width;
use chrono::Local;

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "www.city.hagi.lg.jp";
const LIST_URL: &str = "https://www.city.hagi.lg.jp/soshiki/list8-1.html";

pub fn return_352047() -> Result<(), Box<dyn std::error::Error>> {
    println!("352047, 萩市（萩市消防本部）");

    // 現在の日付を取得（前日の計算用）
    let now = Local::now();
    let yesterday = now.date_naive().pred_opt().unwrap();

    // ステップ1: 一覧ページを取得
    let config = HttpRequestConfig::new(HOST, LIST_URL);
    let list_body = get_source_with_config(&config)?;
    let list_document = Html::parse_document(&list_body);

    // ステップ2: リンクと更新日を抽出
    let item_selector = Selector::parse("div.list_ccc li").unwrap();
    let link_selector = Selector::parse("a").unwrap();
    let date_selector = Selector::parse("span.span_b.article_date").unwrap();

    let mut disaster_data = vec![];

    for item_element in list_document.select(&item_selector) {
        let item_text = item_element.text().collect::<String>();

        // 「発生」が含まれているもののみ処理（「鎮火」は除外）
        if !item_text.contains("発生") || item_text.contains("鎮火") {
            continue;
        }

        // 更新日を抽出: "(2026年1月13日更新)"
        if let Some(date_element) = item_element.select(&date_selector).next() {
            let date_text = to_half_width(&date_element.text().collect::<String>());

            // "2026年1月13日" の形式をパース
            if let Some(date_str) = date_text.strip_prefix("(").and_then(|s| s.strip_suffix("更新)")) {
                if let Some((year_str, rest)) = date_str.split_once("年") {
                    if let Some((month_str, day_str)) = rest.split_once("月") {
                        if let (Ok(year), Ok(month), Ok(day)) = (
                            year_str.trim().parse::<i32>(),
                            month_str.trim().parse::<u32>(),
                            day_str.trim().trim_end_matches("日").parse::<u32>()
                        ) {
                            // 日付をチェック
                            if let Some(update_date) = chrono::NaiveDate::from_ymd_opt(year, month, day) {
                                // 前日より前の場合はスキップ
                                if update_date < yesterday {
                                    continue;
                                }
                            }
                        }
                    }
                }
            }
        }

        // リンクを抽出
        if let Some(link_element) = item_element.select(&link_selector).next() {
            if let Some(href) = link_element.value().attr("href") {
                // 相対URLを絶対URLに変換
                let detail_url = if href.starts_with("http") {
                    href.to_string()
                } else {
                    format!("https://{}{}", HOST, href)
                };

                // ステップ3: 詳細ページを取得
                let detail_config = HttpRequestConfig::new(HOST, &detail_url);
                match get_source_with_config(&detail_config) {
                    Ok(detail_body) => {
                        let detail_document = Html::parse_document(&detail_body);

                        // 詳細情報を抽出
                        let writing_selector = Selector::parse("div.detail_writing").unwrap();
                        if let Some(writing_element) = detail_document.select(&writing_selector).next() {
                            let text = writing_element.text().collect::<String>();
                            let lines: Vec<&str> = text.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();

                            if lines.len() >= 2 {
                                // 1行目: 住所（例: "萩市大字御許町"）
                                let address_raw = lines[0];

                                // 萩市の情報のみ処理
                                if !address_raw.contains("萩市") {
                                    continue;
                                }

                                let address = if address_raw.starts_with("萩市") {
                                    format!("山口県{}", address_raw)
                                } else {
                                    address_raw.to_string()
                                };

                                // 3行目: 災害種別（例: "建物火災が発生しました。"）
                                let disaster_type = if lines.len() >= 3 {
                                    lines[2]
                                        .replace("が発生しました。", "")
                                        .replace("が発生しました", "")
                                        .trim()
                                        .to_string()
                                } else {
                                    "不明".to_string()
                                };

                                // 現在時刻を取得
                                let time = now.format("%H:%M").to_string();

                                disaster_data.push(json!({
                                    "type": disaster_type,
                                    "address": address,
                                    "time": time
                                }));
                            }
                        }
                    },
                    Err(e) => {
                        eprintln!("  [詳細ページ取得失敗] {}: {}", detail_url, e);
                        continue;
                    }
                }
            }
        }
    }

    let output = json!({
        "jisx0402": "352047",
        "source": [
            {
                "url": LIST_URL,
                "name": "萩市消防本部"
            }
        ],
        "disasters": disaster_data
    });

    let mut file = File::create("dist/352047.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 352047.json （萩市・萩市消防本部）");
    Ok(())
}
