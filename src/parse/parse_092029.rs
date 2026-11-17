use serde_json::json;
use std::fs::File;
use std::io::Write;
use scraper::{Html, Selector};

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "ashikaga-city.mailio.jp";
const LIST_URL_KAHOKU_NONFIRE: &str =
    "https://ashikaga-city.mailio.jp/public/backnumber/055d18bc0de44d4fb2fd781ef6f9b267";
const LIST_URL_KANAN_NONFIRE: &str =
    "https://ashikaga-city.mailio.jp/public/backnumber/ae233aef378247f8a2e31c0a8da6331c";
const LIST_URL_KAHOKU_FIRE: &str =
    "https://ashikaga-city.mailio.jp/public/backnumber/861906c3b63a4f23ad213ba87c086130";

fn collect_disasters(list_url: &str) -> Vec<serde_json::Value> {
    let mut disaster_data = vec![];

    let config = HttpRequestConfig::new(HOST, list_url);
    let list_body = match get_source_with_config(&config) {
        Ok(body) => body,
        Err(e) => {
            eprintln!("  [一覧ページ取得失敗] {}: {}", list_url, e);
            return disaster_data;
        }
    };
    let list_document = Html::parse_document(&list_body);

    let item_selector = Selector::parse("li.list-group-item").unwrap();
    let link_selector = Selector::parse("a").unwrap();
    let img_selector = Selector::parse("img.new").unwrap();

    for item_element in list_document.select(&item_selector) {
        // <img class="new">要素が含まれているもの（配信から日が浅いもの）のみ処理
        if item_element.select(&img_selector).next().is_none() {
            continue;
        }

        let Some(link_element) = item_element.select(&link_selector).next() else {
            continue;
        };
        let Some(href) = link_element.value().attr("href") else {
            continue;
        };

        let detail_url = if href.starts_with("http") {
            href.to_string()
        } else {
            format!("https://{}{}", HOST, href)
        };

        let detail_config = HttpRequestConfig::new(HOST, &detail_url);
        let detail_body = match get_source_with_config(&detail_config) {
            Ok(body) => body,
            Err(e) => {
                eprintln!("  [詳細ページ取得失敗] {}: {}", detail_url, e);
                continue;
            }
        };
        let detail_document = Html::parse_document(&detail_body);

        let message_selector =
            Selector::parse("div.app-container.page.page-article div.envelope p.message").unwrap();
        let Some(message_element) = detail_document.select(&message_selector).next() else {
            continue;
        };

        let text = message_element.text().collect::<String>();

        // 「誤報」または「鎮火」の連絡はスキップ（現在進行中の災害ではないため）
        if text.contains("誤報") || text.contains("鎮火") {
            continue;
        }

        // 改行・連続空白を単一スペースに統一
        // 例: "足利市消防防災情報 12:51:09 足利市小俣町 の風水害に消防車が出動しました。 ---------------------- 配信元：足利市消防本部 ----------------------"
        let text = text.replace("\r", " ").replace("\n", " ");
        let text = text.split_whitespace().collect::<Vec<_>>().join(" ");

        let Some(after_header) = text.split("足利市消防防災情報").nth(1) else {
            continue;
        };
        let after_header = after_header.trim();

        let mut header_parts = after_header.splitn(2, ' ');
        let Some(time_token) = header_parts.next() else {
            continue;
        };
        let Some(rest) = header_parts.next() else {
            continue;
        };

        // "12:51:09" → "12:51"
        let time = time_token
            .split(':')
            .take(2)
            .collect::<Vec<_>>()
            .join(":");

        // "足利市小俣町 の風水害に消防車が出動しました。..." → 住所と災害種別を抽出
        let Some((address_part, after_no)) = rest.split_once(" の") else {
            continue;
        };
        let address = address_part.trim();
        let address = if address.starts_with("足利市") {
            format!("栃木県{}", address)
        } else {
            address.to_string()
        };

        let disaster_type = after_no
            .split("に消防車が出動しました")
            .next()
            .unwrap_or("")
            .trim()
            .to_string();

        if !time.is_empty() && !address.is_empty() && !disaster_type.is_empty() {
            disaster_data.push(json!({
                "type": disaster_type,
                "address": address,
                "time": time
            }));
        }
    }

    disaster_data
}

pub fn return_092029() -> Result<(), Box<dyn std::error::Error>> {
    println!("092029, 足利市消防本部");

    let mut disaster_data = vec![];
    disaster_data.extend(collect_disasters(LIST_URL_KAHOKU_NONFIRE));
    disaster_data.extend(collect_disasters(LIST_URL_KANAN_NONFIRE));
    disaster_data.extend(collect_disasters(LIST_URL_KAHOKU_FIRE));

    let output = json!({
        "jisx0402": "092029",
        "source": [
            {
                "url": LIST_URL_KAHOKU_NONFIRE,
                "name": "足利市消防本部"
            },
            {
                "url": LIST_URL_KANAN_NONFIRE,
                "name": "足利市消防本部"
            },
            {
                "url": LIST_URL_KAHOKU_FIRE,
                "name": "足利市消防本部"
            }
        ],
        "disasters": disaster_data
    });

    let mut file = File::create("dist/092029.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 092029.json （足利市消防本部）");
    Ok(())
}
