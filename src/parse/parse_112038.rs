use serde_json::json;
use std::fs::File;
use std::io::Write;
use scraper::{Html, Selector};
use crate::to_half_width;

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "kawaguchi-city.mailio.jp";
const LIST_URL: &str = "https://kawaguchi-city.mailio.jp/public/backnumber/ade3c8df79a74e889008f2736ff00f94";

pub fn return_112038() -> Result<(), Box<dyn std::error::Error>> {
    println!("112038, 川口市消防局");

    // ステップ1: 一覧ページを取得
    let config = HttpRequestConfig::new(HOST, LIST_URL);
    let list_body = get_source_with_config(&config)?;
    let list_document = Html::parse_document(&list_body);

    // ステップ2: 一覧ページからリンクを抽出（<img>要素が含まれているもののみ）
    let item_selector = Selector::parse("li.list-group-item").unwrap();
    let link_selector = Selector::parse("a").unwrap();
    let img_selector = Selector::parse("img.new").unwrap();
    let mut disaster_data = vec![];

    for item_element in list_document.select(&item_selector) {
        // <img>要素が含まれているかチェック
        if item_element.select(&img_selector).next().is_none() {
            continue;
        }

        // リンクを取得
        if let Some(link_element) = item_element.select(&link_selector).next() {
            if let Some(href) = link_element.value().attr("href") {
                // 詳細ページURLを構築
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

                        // ステップ4: 詳細ページから情報を抽出
                        let message_selector = Selector::parse("div.app-container.page.page-article div.envelope p.message").unwrap();

                        if let Some(message_element) = detail_document.select(&message_selector).next() {
                            let text = message_element.text().collect::<String>();

                            // 全角数字を半角数字に変換
                            let text = to_half_width(&text);

                            // 「誤報」または「鎮火」が含まれている場合はスキップ
                            if text.contains("誤報") || text.contains("鎮火") {
                                continue;
                            }

                            // テキスト全体を改行やスペースを統一してから処理
                            let text = text.replace("\n", " ").replace("\r", " ");
                            let text = text.split_whitespace().collect::<Vec<_>>().join(" ");

                            // 時刻を抽出（「MM月DD日　HH時MM分頃」→「HH:MM」）
                            let time = if let Some(time_part) = text.split("頃").next() {
                                // 最後の「HH時MM分」部分を抽出
                                if let Some(hour_pos) = time_part.rfind("時") {
                                    let after_hour = &time_part[hour_pos + "時".len()..];
                                    let hour_part = time_part[..hour_pos]
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
                                }
                            } else {
                                "".to_string()
                            };

                            // 住所と災害種別を抽出
                            if let Some(rest) = text.split("頃").nth(1) {
                                // 「川口市[住所]付近で[災害種別]が発生しました」
                                if let Some((address_part, type_part)) = rest.split_once("付近で") {
                                    let address = address_part
                                        .trim()
                                        .replace(" ", "")
                                        .replace("　", "");

                                    // 住所に埼玉県を追加
                                    let address = if address.starts_with("川口市") {
                                        format!("埼玉県{}", address)
                                    } else {
                                        address
                                    };

                                    // 災害種別を抽出（「が発生しました」以降を削除）
                                    let disaster_type = if let Some((disaster, _)) = type_part.split_once("が発生し") {
                                        disaster.trim().to_string()
                                    } else {
                                        type_part.trim().to_string()
                                    };

                                    // 必要な情報がすべて揃っている場合のみ追加
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
        "jisx0402": "112038",
        "source": [
            {
                "url": LIST_URL,
                "name": "川口市消防局"
            }
        ],
        "disasters": disaster_data
    });

    let mut file = File::create("dist/112038.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 112038.json （川口市消防局）");
    Ok(())
}
