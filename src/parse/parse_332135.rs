use serde_json::json;
use std::fs::File;
use std::io::Write;
use scraper::{Html, Selector};
use chrono::{Datelike, Duration, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use crate::to_half_width;

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "www.ikkr.jp";
const LIST_URL: &str = "https://www.ikkr.jp/app/user/33213/history/showHistory.do?om=1";

pub fn return_332135() -> Result<(), Box<dyn std::error::Error>> {
    println!("332135, 赤磐市消防本部");

    // ステップ1: 一覧ページを取得
    let config = HttpRequestConfig::new(HOST, LIST_URL).with_shift_jis(true);
    let list_body = get_source_with_config(&config)?;
    let list_document = Html::parse_document(&list_body);

    // 出動（「火災発生連絡」「救助発生連絡」）と終了報が別々の記事として新しい順に並ぶ配信履歴のため、
    // 直近24時間以内の出動のみを対象とする。一覧には日付しかないため、今日・昨日の出動のみ詳細ページを取得する
    let now = Local::now();
    let today = now.date_naive();
    let time_threshold = now - Duration::hours(24);

    let link_selector = Selector::parse("ul.level1col1 li.page a").unwrap();
    let td_selector = Selector::parse("td").unwrap();
    let mut disaster_data = vec![];

    for link_element in list_document.select(&link_selector) {
        // 例: "2026年10月11日：火災発生連絡 <1260000067>"
        let title = link_element.text().collect::<String>();
        let Some((date_part, category_part)) = title.split_once('：') else {
            continue;
        };

        // 終了報（「鎮火連絡」「救出完了連絡」「誤報連絡」「救助活動連絡」）はスキップ
        // （「救助活動連絡」は「救助活動はありませんでした」という内容の終了報）
        let Some((category, _)) = category_part.split_once('<') else {
            continue;
        };
        if !category.trim().ends_with("発生連絡") {
            continue;
        }

        let Ok(list_date) = NaiveDate::parse_from_str(date_part.trim(), "%Y年%m月%d日") else {
            continue;
        };
        // 今日・昨日以外は24時間以内になり得ないため、詳細ページを取得しない
        if list_date < today - Duration::days(1) || list_date > today {
            continue;
        }

        let Some(href) = link_element.value().attr("href") else {
            continue;
        };
        // リンクに含まれるセッションID（";jsessionid=..."）を除去する
        // 例: "/app/user/33213/history/showDetailHistory.do;jsessionid=XXX?pageId=0&mailHistoryId=54002&..."
        let href = match href.split_once(";jsessionid=") {
            Some((path, rest)) => match rest.split_once('?') {
                Some((_, query)) => format!("{}?{}", path, query),
                None => path.to_string(),
            },
            None => href.to_string(),
        };
        let detail_url = if href.starts_with("http") {
            href
        } else {
            format!("https://{}{}", HOST, href)
        };

        // ステップ2: 詳細ページを取得
        let detail_config = HttpRequestConfig::new(HOST, &detail_url).with_shift_jis(true);
        let detail_body = match get_source_with_config(&detail_config) {
            Ok(body) => body,
            Err(e) => {
                eprintln!("  [詳細ページ取得失敗] {}: {}", detail_url, e);
                continue;
            }
        };
        let detail_document = Html::parse_document(&detail_body);

        // 本文は<BR>区切りで1行ずつテキストノードになっている
        // 例: ["10月11日 19時07分", "赤磐市上仁保地内で", "その他の火災", "が発生しました", "−−", "赤磐市消防本部"]
        //     ["10月08日 08時11分", "赤磐市沼田地内で", "一般救助", "通常出動が発生しました", ...]
        let Some(td_element) = detail_document
            .select(&td_selector)
            .find(|td| td.text().any(|text| text.contains("発生しました")))
        else {
            continue;
        };
        let lines: Vec<String> = td_element
            .text()
            .map(|text| to_half_width(text).trim().to_string())
            .filter(|text| !text.is_empty())
            .collect();

        // "10月11日 19時07分" → 月・日・時・分
        let Some(time_line) = lines
            .iter()
            .find(|line| line.contains('月') && line.contains('時') && line.ends_with('分'))
        else {
            continue;
        };
        let Some((month_str, rest)) = time_line.split_once('月') else {
            continue;
        };
        let Some((day_str, rest)) = rest.split_once('日') else {
            continue;
        };
        let Some((hour_str, minute_str)) = rest.split_once('時') else {
            continue;
        };
        let (Ok(month), Ok(day), Ok(hour), Ok(minute)) = (
            month_str.trim().parse::<u32>(),
            day_str.trim().parse::<u32>(),
            hour_str.trim().parse::<u32>(),
            minute_str.trim().trim_end_matches('分').parse::<u32>(),
        ) else {
            continue;
        };

        // 本文には年が無いため一覧の日付の年で補完し、未来日になる場合は前年とみなす
        let mut year = list_date.year();
        if NaiveDate::from_ymd_opt(year, month, day).is_some_and(|date| date > today) {
            year -= 1;
        }
        let (Some(disaster_date), Some(disaster_time)) = (
            NaiveDate::from_ymd_opt(year, month, day),
            NaiveTime::from_hms_opt(hour, minute, 0),
        ) else {
            continue;
        };
        let Some(disaster_datetime) = Local
            .from_local_datetime(&NaiveDateTime::new(disaster_date, disaster_time))
            .single()
        else {
            continue;
        };
        if disaster_datetime < time_threshold || disaster_datetime > now {
            continue;
        }

        // 住所（「…地内で」の行）と、その次の行の災害種別
        let Some(address_index) = lines.iter().position(|line| line.ends_with('で')) else {
            continue;
        };
        let Some(disaster_type) = lines.get(address_index + 1) else {
            continue;
        };
        let location = lines[address_index].trim_end_matches('で').replace("地内", "");
        let address = if location.starts_with("赤磐市") {
            format!("岡山県{}", location)
        } else {
            location
        };

        let time = format!("{:02}:{:02}", hour, minute);

        if !address.is_empty() && !disaster_type.is_empty() {
            disaster_data.push(json!({
                "type": disaster_type,
                "address": address,
                "time": time
            }));
        }
    }

    let output = json!({
        "jisx0402": "332135",
        "source": [
            {
                "url": LIST_URL,
                "name": "赤磐市消防本部"
            }
        ],
        "disasters": disaster_data
    });

    let mut file = File::create("dist/332135.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 332135.json （赤磐市消防本部）");
    Ok(())
}
