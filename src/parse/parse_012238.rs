use serde_json::json;
use std::fs::File;
use std::io::Write;
use scraper::{ElementRef, Html, Selector};
use crate::to_half_width;

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "www.city.nemuro.hokkaido.jp";
const GET_SOURCE: &str = "https://www.city.nemuro.hokkaido.jp/lifeinfo/kakuka/syoubou/shoubousho/kyuukyuutushin/oshirase/605.html";

// 災害種別の書き出しに使われる語。本文では住所と災害種別が区切りなく続く
// （例: "槍昔林野火災"、"光洋町4丁目救急救助"）ため、これらのうち最も手前に現れる位置で分ける
const TYPE_HEADS: &[&str] = &[
    "ドクターヘリ", "救急", "救助", "火災", "火事", "警戒", "調査", "誤認", "誤報",
    "その他", "交通", "水難", "林野", "建物", "車両", "船舶", "野火",
];

// 根室市に隣接し、応援出動で住所に現れる町（例: "浜中町恵茶人"）
const NEIGHBOR_TOWNS: &[&str] = &["浜中町", "別海町"];

// "11月01日午後4時07" → "16:07"（午前・午後の12時間表記を24時間表記にする）
fn to_24h_time(text: &str) -> Option<String> {
    let (is_pm, hour_minute) = match text.split_once("午後") {
        Some((_, hour_minute)) => (true, hour_minute),
        None => (false, text.split_once("午前")?.1),
    };
    let (hour, minute) = hour_minute.split_once('時')?;
    let (hour, minute) = (hour.parse::<u32>().ok()?, minute.parse::<u32>().ok()?);
    let hour = match (is_pm, hour) {
        (true, 0..=11) => hour + 12,
        (false, 12) => 0,
        _ => hour,
    };
    Some(format!("{:02}:{:02}", hour, minute))
}

// "光洋町4丁目救急救助" → ("光洋町4丁目", "救急救助")。種別の語が無い場合は全体を災害種別とする
fn split_location_and_type(text: &str) -> (&str, String) {
    TYPE_HEADS
        .iter()
        .filter_map(|head| {
            text.split_once(head)
                .map(|(location, rest)| (location, format!("{}{}", head, rest)))
        })
        .min_by_key(|(location, _)| location.len())
        .unwrap_or(("", text.to_string()))
}

pub fn return_012238() -> Result<(), Box<dyn std::error::Error>> {
    println!("012238, 根室市消防本部");

    let config = HttpRequestConfig::new(HOST, GET_SOURCE);
    let body = get_source_with_config(&config)?;
    let document = Html::parse_document(&body);

    // 「現在発生している災害」の見出しの直後の<div>に、現在の出動が1件ずつ<p>で入っている
    // （その次の<div>は注記で、次の見出しからは「過去の災害出動情報」）
    let h2_selector = Selector::parse("h2").unwrap();
    let p_selector = Selector::parse("p").unwrap();
    let Some(current_list) = document
        .select(&h2_selector)
        .find(|h2| h2.text().collect::<String>().contains("現在発生している災害"))
        .and_then(|h2| h2.next_siblings().find_map(ElementRef::wrap))
    else {
        return Err(format!("「現在発生している災害」の欄が見つかりません: {}", GET_SOURCE).into());
    };

    let mut disaster_data = vec![];

    for p_element in current_list.select(&p_selector) {
        // 全角数字を半角数字に変換し、空白を除去
        let text: String = to_half_width(&p_element.text().collect::<String>())
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();

        // 災害がない場合
        if text.contains("出動情報はありません") {
            continue;
        }

        // 例: "11月01日午後4時07分槍昔林野火災のため消防車が出動しました。"
        let Some((date_time_part, rest)) = text.split_once('分') else {
            continue;
        };
        let Some(time) = to_24h_time(date_time_part) else {
            continue;
        };
        let Some((location_and_type, _)) = rest.split_once("のため") else {
            continue;
        };

        let (location, disaster_type) = split_location_and_type(location_and_type);
        let location = location.replace("付近", "");

        // 住所が無い場合（例: "ドクターヘリ支援"）は根室市とし、隣接する町の場合は市名を付けない
        let address = if location.is_empty() {
            "北海道根室市".to_string()
        } else if NEIGHBOR_TOWNS.iter().any(|town| location.starts_with(town)) {
            format!("北海道{}", location)
        } else {
            format!("北海道根室市{}", location)
        };

        disaster_data.push(json!({
            "type": disaster_type,
            "address": address,
            "time": time
        }));
    }

    let output = json!({
        "jisx0402": "012238",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "根室市消防本部"
            }
        ],
        "disasters": disaster_data
    });

    let mut file = File::create("dist/012238.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 012238.json （根室市消防本部）");
    Ok(())
}
