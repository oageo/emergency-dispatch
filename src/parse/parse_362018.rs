use serde_json::json;
use std::fs::File;
use std::io::Write;
use scraper::{Html, Selector};
use crate::to_half_width;

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "tokushima-fd.mailio.jp";
const LIST_URL: &str = "https://tokushima-fd.mailio.jp/public/backnumber/d2d6d3fb307f4a91a996d2f975ed97f8";

pub fn return_362018() -> Result<(), Box<dyn std::error::Error>> {
    println!("362018, 徳島市消防局");

    // ステップ1: 一覧ページを取得
    let config = HttpRequestConfig::new(HOST, LIST_URL);
    let list_body = get_source_with_config(&config)?;
    let list_document = Html::parse_document(&list_body);

    // ステップ2: 一覧ページからリンクを抽出（<img class="new">要素が含まれているもののみ）
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

        // 終了報（誤報情報・鎮火情報・活動完了情報・救出完了情報・救出済情報）はスキップ
        // （現在進行中の災害ではないため）
        if text.contains("誤報") || text.contains("鎮火") || text.contains("完了") {
            continue;
        }

        // ステップ4: 詳細ページから情報を抽出
        // 例: "07時05分徳島市八万町大野付近で一般救助事案が発生し、消防車が出動しました。"
        let Some((time_part, rest)) = text.split_once('分') else {
            continue;
        };
        let Some((location_and_type, _)) = rest.split_once("事案が発生し") else {
            continue;
        };
        let Some((address, disaster_type)) = location_and_type.split_once("付近で") else {
            continue;
        };

        // "07時05" → "07:05"
        let time = time_part.replace("時", ":");

        let address = if address.starts_with("徳島市") {
            format!("徳島県{}", address)
        } else {
            address.to_string()
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
        "jisx0402": "362018",
        "source": [
            {
                "url": LIST_URL,
                "name": "徳島市消防局"
            }
        ],
        "disasters": disaster_data
    });

    let mut file = File::create("dist/362018.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 362018.json （徳島市消防局）");
    Ok(())
}
