use serde_json::json;
use std::fs::File;
use std::io::Write;
use scraper::{Html, Selector};
use chrono::{Datelike, Duration, Local, NaiveDate, NaiveDateTime, TimeZone};

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "www.city.fukushima.fukushima.jp";
// 福島市消防本部のページ（https://www.city.fukushima.fukushima.jp/bosai-anzen/shobo/1/index.html）の
// 「出動情報」欄は空のままで、JavaScript（list_e_1001-2.js）がこのURLをAjaxで取得し、
// 返ってきた<li>要素を流し込んで表示している。そのため直接こちらを取得する。
const GET_SOURCE: &str = "https://www.city.fukushima.fukushima.jp/section/syoubou-info/history.html";

pub fn return_072010() -> Result<(), Box<dyn std::error::Error>> {
    println!("072010, 福島市消防本部");

    let config = HttpRequestConfig::new(HOST, GET_SOURCE);
    let body = get_source_with_config(&config)?;
    let document = Html::parse_document(&body);

    // 「現在」の状態を示す仕組みが無く、日付を跨いだ配信履歴がそのまま並ぶ一覧のため、
    // 土浦市・名古屋市と同様に直近24時間以内のもののみを対象とする
    let now = Local::now();
    let today = now.date_naive();
    let time_threshold = now - Duration::hours(24);

    let li_selector = Selector::parse("li").unwrap();
    let mut disaster_data = vec![];

    for li_element in document.select(&li_selector) {
        let text = li_element.text().collect::<String>();
        let text = text.trim();

        // 例: "08月14日 19時43分　【火災】福島市泉字先達地内で中高層火災が発生しました。"
        let Some((prefix, after_bracket_open)) = text.split_once('【') else {
            continue;
        };
        let Some((category, message)) = after_bracket_open.split_once('】') else {
            continue;
        };

        let mut prefix_tokens = prefix.split_whitespace();
        let Some(date_str) = prefix_tokens.next() else {
            continue;
        };
        let Some(time_str) = prefix_tokens.next() else {
            continue;
        };

        // 日付を解析（年の記載が無いため、現在の年を基準に補完する）
        let Some((month_str, day_str)) = date_str.split_once('月') else {
            continue;
        };
        let (Ok(month), Ok(day)) = (
            month_str.trim().parse::<u32>(),
            day_str.trim().trim_end_matches('日').parse::<u32>(),
        ) else {
            continue;
        };

        // 時刻を解析
        let Some((hour_str, minute_str)) = time_str.split_once('時') else {
            continue;
        };
        let (Ok(hour), Ok(minute)) = (
            hour_str.trim().parse::<u32>(),
            minute_str.trim().trim_end_matches('分').parse::<u32>(),
        ) else {
            continue;
        };

        // 年またぎ（現在の年で組み立てると未来日になる場合は前年とみなす）を考慮して日付を確定
        let mut year = now.year();
        let mut disaster_date = NaiveDate::from_ymd_opt(year, month, day);
        if let Some(d) = disaster_date {
            if d > today {
                year -= 1;
                disaster_date = NaiveDate::from_ymd_opt(year, month, day);
            }
        }
        let Some(disaster_date) = disaster_date else {
            continue;
        };
        let Some(naive_time) = chrono::NaiveTime::from_hms_opt(hour, minute, 0) else {
            continue;
        };
        let naive_datetime = NaiveDateTime::new(disaster_date, naive_time);
        let Some(disaster_datetime) = Local.from_local_datetime(&naive_datetime).single() else {
            continue;
        };

        // 直近24時間以内でない場合はスキップ（既に対応済みの古い情報を誤って取得しないため）
        if disaster_datetime < time_threshold || disaster_datetime > now {
            continue;
        }

        // 住所と災害種別を抽出
        // 「地内で」または「地内へ」の前が住所
        let (address_part, rest) = if let Some((a, r)) = message.split_once("地内で") {
            (a, r)
        } else if let Some((a, r)) = message.split_once("地内へ") {
            (a, r)
        } else {
            continue;
        };

        let address = address_part.trim();
        let address = if address.starts_with("福島市") {
            format!("福島県{}", address)
        } else {
            address.to_string()
        };

        // 災害種別は「に伴う」「が発生しました」「のため消防車」のいずれかの直前を抽出し、
        // 末尾の「活動」を削る（例: "救助活動" → "救助"、"中高層火災" → そのまま）
        let type_raw = if let Some((t, _)) = rest.split_once("に伴う") {
            Some(t)
        } else if let Some((t, _)) = rest.split_once("が発生しました") {
            Some(t)
        } else if let Some((t, _)) = rest.split_once("のため消防車") {
            Some(t)
        } else {
            None
        };

        let disaster_type = match type_raw {
            Some(t) => t.strip_suffix("活動").unwrap_or(t).trim().to_string(),
            // 上記のパターンに当てはまらない場合（例: 救急支援）は冒頭の【】内の分類を使用
            None => category.trim().to_string(),
        };

        let time = format!("{:02}:{:02}", hour, minute);

        if !time.is_empty() && !address.is_empty() && !disaster_type.is_empty() {
            disaster_data.push(json!({
                "type": disaster_type,
                "address": address,
                "time": time
            }));
        }
    }

    let output = json!({
        "jisx0402": "072010",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "福島市消防本部"
            }
        ],
        "disasters": disaster_data
    });

    let mut file = File::create("dist/072010.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 072010.json （福島市消防本部）");
    Ok(())
}
