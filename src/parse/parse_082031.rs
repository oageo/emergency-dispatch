use serde_json::json;
use std::fs::File;
use std::io::Write;
use chrono::{Local, NaiveDateTime, Duration, TimeZone};

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "www.city.tsuchiura.lg.jp";
const GET_SOURCE: &str = "https://www.city.tsuchiura.lg.jp/mm_pro/backnumber.php";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    let config = HttpRequestConfig::new(HOST, GET_SOURCE);
    get_source_with_config(&config)
}

pub fn return_082031() -> Result<(), Box<dyn std::error::Error>> {
    println!("082031, 土浦市消防本部");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);

    // 現在時刻を取得（JST）
    let now = Local::now();
    // 24時間前の時刻を計算
    let time_threshold = now - Duration::hours(24);

    // 各tableを処理（配信情報のテーブル）
    let table_selector = scraper::Selector::parse("table[width='450']").unwrap();
    let mut disaster_data = vec![];

    for table in document.select(&table_selector) {
        let table_text = table.text().collect::<String>();

        // 配信日時を抽出して24時間以内かチェック
        // 形式: "2025年10月18日（土） 02時40分 配信"
        let datetime_selector = scraper::Selector::parse("td[bgcolor='#0033FF']").unwrap();
        if let Some(datetime_element) = table.select(&datetime_selector).next() {
            let datetime_text = datetime_element.text().collect::<String>();

            // 日時文字列をパース
            // "2025年10月18日（土） 02時40分 配信" から日時部分を抽出
            if let Some(datetime_str) = datetime_text.split(" 配信").next() {
                // "2025年10月18日（土） 02時40分" を解析
                let cleaned = datetime_str
                    .split("（").next().unwrap_or("")  // "2025年10月18日"
                    .to_string() + " " +
                    datetime_str.split("） ").nth(1).unwrap_or("");  // "02時40分"

                // "2025年10月18日 02時40分" をパース
                let parsed_datetime = NaiveDateTime::parse_from_str(
                    &cleaned.replace("年", "-").replace("月", "-").replace("日", "").replace("時", ":").replace("分", ""),
                    "%Y-%m-%d %H:%M"
                ).ok().map(|dt| Local.from_local_datetime(&dt).unwrap());

                // 24時間以内でない場合はスキップ
                if let Some(post_time) = parsed_datetime {
                    if post_time < time_threshold {
                        continue;
                    }
                } else {
                    // パースできない場合もスキップ
                    continue;
                }
            }
        }

        // 本文の行のみを処理（「消防車が出動しました」を含む行）
        if table_text.contains("消防車が出動しました") {
            let text = table_text;
            // 例: "2025年10月18日（土）、2時36分頃、下高津二丁目で救急支援（救急隊活動の補助）が発生し消防車が出動しました。"

            // 時刻を抽出（「時」と「分頃」の間）
            let time_raw = text
                .split("、")
                .nth(1)
                .unwrap_or("")
                .split("頃")
                .next()
                .unwrap_or("")
                .trim();

            // 時と分を分離して2桁にフォーマット
            let time = if let Some((hour_str, minute_str)) = time_raw.split_once("時") {
                let hour = hour_str.trim().parse::<u32>().unwrap_or(0);
                let minute = minute_str.replace("分", "").trim().parse::<u32>().unwrap_or(0);
                format!("{:02}:{:02}", hour, minute)
            } else {
                time_raw.to_string()
            };

            // 住所と災害種別を抽出
            // 「頃、」の後から「で」までが住所
            // 「で」の後から「が発生し」までが災害種別
            if let Some(after_time) = text.split("頃、").nth(1) {
                if let Some(address_part) = after_time.split("で").next() {
                    let address = format!("茨城県土浦市{}", address_part.trim());

                    // 災害種別を抽出
                    if let Some(disaster_part) = after_time.split("で").nth(1) {
                        let disaster_type = disaster_part
                            .split("が発生し")
                            .next()
                            .unwrap_or("")
                            .trim()
                            .to_string();

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
        "jisx0402": "082031",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "土浦市消防本部"
            }
        ],
        "disasters": disaster_data
    });

    // JSONファイルに書き出し
    let mut file = File::create("dist/082031.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 082031.json （土浦市消防本部）");
    Ok(())
}
