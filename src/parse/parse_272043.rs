use serde_json::json;
use std::fs::File;
use std::io::Write;
use scraper::{Html, Selector};
use crate::to_half_width;

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "noc119-kikan.mailio.jp";
const LIST_URL: &str = "https://noc119-kikan.mailio.jp/public/backnumber/070af67b6f5147dcb51ac2ec5c065631";

pub fn return_272043() -> Result<(), Box<dyn std::error::Error>> {
    println!("272043, 池田市消防本部");

    // ステップ1: 一覧ページを取得
    let config = HttpRequestConfig::new(HOST, LIST_URL);
    let list_body = get_source_with_config(&config)?;
    let list_document = Html::parse_document(&list_body);

    // ステップ2: 一覧ページからリンクを抽出（<img class="new">要素が含まれているもののみ）
    // 出動の配信のみで終了報は配信されないため、配信から約24時間で外れる<img class="new">で絞り込む
    let item_selector = Selector::parse("li.list-group-item").unwrap();
    let link_selector = Selector::parse("a").unwrap();
    let img_selector = Selector::parse("img.new").unwrap();
    let message_selector =
        Selector::parse("div.app-container.page.page-article div.envelope p.message").unwrap();
    let mut disaster_data = vec![];

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

        // ステップ3: 詳細ページを取得
        let detail_config = HttpRequestConfig::new(HOST, &detail_url);
        let detail_body = match get_source_with_config(&detail_config) {
            Ok(body) => body,
            Err(e) => {
                eprintln!("  [詳細ページ取得失敗] {}: {}", detail_url, e);
                continue;
            }
        };
        let detail_document = Html::parse_document(&detail_body);

        let Some(message_element) = detail_document.select(&message_selector).next() else {
            continue;
        };

        // 全角数字を半角数字に変換し、改行を除去
        let text = to_half_width(&message_element.text().collect::<String>());
        let text: String = text.chars().filter(|c| !c.is_whitespace()).collect();

        // 「誤報」または「鎮火」が含まれている場合はスキップ（現在進行中の災害ではないため）
        if text.contains("誤報") || text.contains("鎮火") {
            continue;
        }

        // ステップ4: 詳細ページから情報を抽出
        // 例: "こちらは池田市消防本部です。09月26日15時22分頃、池田市畑3丁目付近で、救助事案が発生しております。"
        let Some((date_time_part, rest)) = text.split_once("頃、") else {
            continue;
        };

        // "...09月26日15時22分" → "15:22"
        let Some((_, time_str)) = date_time_part.rsplit_once('日') else {
            continue;
        };
        let time = time_str.replace("時", ":").replace("分", "");

        let Some((address, type_part)) = rest.split_once("付近で、") else {
            continue;
        };
        let Some((disaster_type, _)) = type_part.split_once("が発生") else {
            continue;
        };
        // "救助事案" → "救助"、"火災" → そのまま
        let disaster_type = disaster_type.strip_suffix("事案").unwrap_or(disaster_type);

        // 住所以外（例: "滑走路上緊急着陸【池田用】"）の場合は、住所を「大阪府池田市」とし、
        // その文言を災害種別の括弧内にそのまま入れる（例: "警戒（滑走路上緊急着陸【池田用】）"）
        let (address, disaster_type) = if address.starts_with("池田市") {
            (format!("大阪府{}", address), disaster_type.to_string())
        } else {
            ("大阪府池田市".to_string(), format!("{}（{}）", disaster_type, address))
        };

        if !time.is_empty() && !address.is_empty() && !disaster_type.is_empty() {
            disaster_data.push(json!({
                "type": disaster_type,
                "address": address,
                "time": time
            }));
        }
    }

    let output = json!({
        "jisx0402": "272043",
        "source": [
            {
                "url": LIST_URL,
                "name": "池田市消防本部"
            }
        ],
        "disasters": disaster_data
    });

    let mut file = File::create("dist/272043.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 272043.json （池田市消防本部）");
    Ok(())
}
