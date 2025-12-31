use serde_json::json;
use std::fs::File;
use std::io::Write;
use crate::to_half_width;
use chrono::{DateTime, Local, Duration};

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "nagoya-fd.site2.ktaiwork.jp";
const GET_SOURCE: &str = "https://nagoya-fd.site2.ktaiwork.jp/";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    let config = HttpRequestConfig::new(HOST, GET_SOURCE);
    get_source_with_config(&config)
}

pub fn return_231002() -> Result<(), Box<dyn std::error::Error>> {
    println!("231002, 名古屋市消防局");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);
    let article_selector = scraper::Selector::parse("article").unwrap();
    let mut disaster_data = vec![];

    // 現在時刻を取得（JST）
    let now = Local::now();
    // 24時間前の時刻を計算
    let time_threshold = now - Duration::hours(24);

    // 各article要素を処理
    for article in document.select(&article_selector) {
        // 投稿日時を取得してフィルタリング
        let time_selector = scraper::Selector::parse("time.entry-date").unwrap();
        let post_datetime = article
            .select(&time_selector)
            .next()
            .and_then(|element| element.value().attr("datetime"))
            .and_then(|dt_str| DateTime::parse_from_rfc3339(dt_str).ok())
            .map(|dt| dt.with_timezone(&Local));

        // 投稿日時が24時間以内でない場合はスキップ
        if let Some(post_time) = post_datetime {
            if post_time < time_threshold {
                continue;
            }
        } else {
            // 投稿日時が取得できない場合もスキップ
            continue;
        }

        // タイトルを取得して「火災発生」かどうかを確認
        let title_selector = scraper::Selector::parse("h1.entry-title").unwrap();
        let title = article
            .select(&title_selector)
            .next()
            .map(|element| element.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        // 「火災鎮火」「火災誤報」の場合はスキップ
        if title.contains("鎮火") || title.contains("誤報") {
            continue;
        }

        // entry-content内のテキストを取得
        let content_selector = scraper::Selector::parse("div.entry-content p").unwrap();
        let content = article
            .select(&content_selector)
            .next()
            .map(|element| to_half_width(&element.text().collect::<String>().trim().to_string()))
            .unwrap_or_default();

        // 「鎮火」という文字列が本文に含まれている場合もスキップ
        if content.contains("鎮火") {
            continue;
        }

        // 内容が空の場合はスキップ
        if content.is_empty() {
            continue;
        }

        // パース処理
        // 形式: "2025年09月30日12時38分頃、南区呼続４丁目　地内から火災の通報があり消防車が出動しています。"

        // 時刻を抽出（例: "12時38分" → "12:38"）
        let time = if let Some(time_str) = content.split("日").nth(1) {
            time_str
                .split("頃").next()
                .unwrap_or("")
                .replace("時", ":")
                .replace("分", "")
                .trim()
                .to_string()
        } else {
            continue;
        };

        // 住所を抽出（例: "南区呼続４丁目"）
        let address = if let Some(addr_part) = content.split("頃、").nth(1) {
            let addr = addr_part
                .split("地内").next()
                .unwrap_or("")
                .trim()
                .replace("　", "");

            // 愛知県名古屋市を追加
            format!("愛知県名古屋市{}", addr)
        } else {
            continue;
        };

        // 災害種別を抽出（例: "から火災の通報" → "火災"）
        let disaster_type = if content.contains("から") && content.contains("の通報") {
            content
                .split("から").nth(1)
                .unwrap_or("")
                .split("の通報").next()
                .unwrap_or("火災")
                .trim()
                .to_string()
        } else {
            // 将来の拡張のため、デフォルトはタイトルから推測
            title.clone()
        };

        disaster_data.push(json!({
            "type": disaster_type,
            "address": address,
            "time": time
        }));
    }

    let output = json!({
        "jisx0402": "231002",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "名古屋市消防局"
            }
        ],
        "disasters": disaster_data
    });

    // JSONファイルに書き出し
    let mut file = File::create("dist/231002.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 231002.json （名古屋市消防局）");
    Ok(())
}
