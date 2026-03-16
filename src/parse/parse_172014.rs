use serde_json::json;
use std::fs::File;
use std::io::Write;

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "fire.city.kanazawa.ishikawa.jp";
const GET_SOURCE: &str = "https://fire.city.kanazawa.ishikawa.jp/contents.html";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    let config = HttpRequestConfig::new(HOST, GET_SOURCE).with_euc_jp(true);
    get_source_with_config(&config)
}

pub fn return_172014() -> Result<(), Box<dyn std::error::Error>> {
    println!("172014, 金沢市消防局");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);
    let b_selector = scraper::Selector::parse("b").unwrap();
    let mut disaster_data = vec![];

    // ページ構造:
    // === YYYY年MM月DD日 HH時MM分 更新 ===<br>
    // <b>N. YYYY/MM/DD （曜） HH:MM<br></b>
    // &nbsp;&nbsp;&nbsp;金沢市[住所]地内で[種別]が発生しています。<br>
    for b_elem in document.select(&b_selector) {
        let b_text = b_elem.text().collect::<String>();
        let b_text = b_text.trim();

        // "N. YYYY/MM/DD （曜） HH:MM" の形式かチェック（"/"と":"を含む）
        if !b_text.contains('/') || !b_text.contains(':') {
            continue;
        }

        // 時刻を抽出（末尾のトークン "HH:MM"）
        let time = b_text.split_whitespace().last().unwrap_or("").to_string();
        if time.is_empty() {
            continue;
        }

        // 次の兄弟ノード（テキストノード）から住所・種別を取得
        if let Some(sibling) = b_elem.next_sibling() {
            if let scraper::node::Node::Text(text_node) = sibling.value() {
                // &nbsp; (\u{a0}) を除去
                let content = text_node.trim().replace('\u{a0}', "").replace('　', "");

                // "金沢市[住所]地内で[種別]が発生しています。"
                if let Some((location_part, rest)) = content.split_once("地内で") {
                    let address = format!("石川県{}", location_part.trim());
                    let disaster_type = rest
                        .split("が発生しています")
                        .next()
                        .unwrap_or("")
                        .trim()
                        .to_string();

                    if !disaster_type.is_empty() && !time.is_empty() {
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
        "jisx0402": "172014",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "金沢市消防局"
            }
        ],
        "disasters": disaster_data
    });

    let mut file = File::create("dist/172014.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 172014.json （金沢市消防局）");
    Ok(())
}
