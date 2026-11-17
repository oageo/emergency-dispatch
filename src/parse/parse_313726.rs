use serde_json::json;
use std::fs::File;
use std::io::Write;
use scraper::{Html, Selector};
use chrono::{Duration, Local, NaiveDateTime, TimeZone};
use crate::to_half_width;

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "www.chubu-furusato-tottori.jp";
const GET_SOURCE: &str = "https://www.chubu-furusato-tottori.jp/category/dis-info";

// 本文から（覚知時刻, 災害種別, 場所）を抽出する。
// 場所の後ろに続く「鳥取中部ふるさと広域連合消防局」が見つからない場合
// （一覧の抜粋が場所の途中で切れている場合）は None を返す
// 例: "260900378 09月26日 14時56分覚知 救助発生 交通事故 場所:琴浦町森藤地内 鳥取中部ふるさと広域連合消防局 …"
fn parse_dispatch_text(text: &str) -> Option<(String, String, String)> {
    // 全角数字を半角数字に変換
    let text = to_half_width(text);

    let (date_time_part, rest) = text.split_once("分覚知")?;

    // "260900378 09月26日 14時56" → "14:56"
    let (_, time_str) = date_time_part.rsplit_once('日')?;
    let time = time_str.trim().replace("時", ":");

    let (kind_part, location_part) = rest.split_once("場所:")?;
    let (location, _) = location_part.split_once("鳥取中部ふるさと広域連合消防局")?;

    // "救助発生 交通事故" → "救助（交通事故）"
    let mut kind_tokens = kind_part.split_whitespace();
    let category = kind_tokens.next()?;
    let category = category.strip_suffix("発生").unwrap_or(category);
    let disaster_type = match kind_tokens.next() {
        Some(detail) => format!("{}（{}）", category, detail),
        None => category.to_string(),
    };

    // "琴浦町森藤地内" → "琴浦町森藤"
    let location: String = location.chars().filter(|c| !c.is_whitespace()).collect();
    let location = location.replace("地内", "");

    Some((time, disaster_type, location))
}

pub fn return_313726() -> Result<(), Box<dyn std::error::Error>> {
    println!("313726, 北栄町（鳥取中部ふるさと広域連合消防局）");

    let config = HttpRequestConfig::new(HOST, GET_SOURCE);
    let body = get_source_with_config(&config)?;
    let document = Html::parse_document(&body);

    // 出動（「緊急連絡」）と終了報が別々の記事として新しい順に並ぶ配信履歴のため、
    // 直近24時間以内の「緊急連絡」のみを対象とする。
    // 一覧の抜粋（空白込み約100文字で省略される）に覚知時刻・種別・場所が含まれるため、
    // 抜粋が場所の途中で切れている場合を除いて詳細ページは取得しない
    let now = Local::now();
    let time_threshold = now - Duration::hours(24);

    let item_selector = Selector::parse("div.sub-archive").unwrap();
    let title_selector = Selector::parse("h2.sub-entry-title").unwrap();
    let link_selector = Selector::parse("a").unwrap();
    let date_selector = Selector::parse("div.sub-entry-meta span").unwrap();
    let summary_selector = Selector::parse("div.sub-entry-summary p").unwrap();
    let content_selector = Selector::parse("section.entry-content").unwrap();
    let mut disaster_data = vec![];

    for item_element in document.select(&item_selector) {
        let Some(title_element) = item_element.select(&title_selector).next() else {
            continue;
        };
        let title = title_element.text().collect::<String>();

        // 終了報（「鎮火のお知らせ」「救助必要なし」「誤報のお知らせ」「救助完了」「本日の活動終了」）はスキップ。
        // 「本日の活動終了」の本文には「救助発生」が含まれるため、本文ではなくタイトルで判定する
        if title.trim() != "緊急連絡" {
            continue;
        }

        // 例: "2026年9月26日 14:58"
        let Some(date_element) = item_element.select(&date_selector).next() else {
            continue;
        };
        let date_text = date_element.text().collect::<String>();
        let Ok(naive_datetime) = NaiveDateTime::parse_from_str(date_text.trim(), "%Y年%m月%d日 %H:%M") else {
            continue;
        };
        let Some(published) = Local.from_local_datetime(&naive_datetime).single() else {
            continue;
        };
        if published < time_threshold || published > now {
            continue;
        }

        let Some(summary_element) = item_element.select(&summary_selector).next() else {
            continue;
        };
        let summary = summary_element.text().collect::<Vec<_>>().join(" ");

        let parsed = match parse_dispatch_text(&summary) {
            Some(parsed) => Some(parsed),
            None => {
                // 一覧の抜粋が場所の途中で切れている場合は詳細ページを取得する
                let Some(href) = title_element
                    .select(&link_selector)
                    .next()
                    .and_then(|link_element| link_element.value().attr("href"))
                else {
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

                // 詳細ページは項目ごとに<p>や<br>で区切られているため、空白で連結して一覧の抜粋と同じ形にする
                detail_document
                    .select(&content_selector)
                    .next()
                    .and_then(|content_element| {
                        parse_dispatch_text(&content_element.text().collect::<Vec<_>>().join(" "))
                    })
            }
        };
        let Some((time, disaster_type, location)) = parsed else {
            continue;
        };

        // 北栄町の事案のみ
        if !location.starts_with("北栄町") {
            continue;
        }
        let address = format!("鳥取県{}", location);

        if !time.is_empty() && !disaster_type.is_empty() {
            disaster_data.push(json!({
                "type": disaster_type,
                "address": address,
                "time": time
            }));
        }
    }

    let output = json!({
        "jisx0402": "313726",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "鳥取中部ふるさと広域連合消防局"
            }
        ],
        "disasters": disaster_data
    });

    let mut file = File::create("dist/313726.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 313726.json （北栄町・鳥取中部ふるさと広域連合消防局）");
    Ok(())
}
